#![no_std]
#![no_main]

use core::panic::PanicInfo;

type StablePublish = unsafe extern "C" fn(*mut u8, u16);
type StableDivide = unsafe extern "C" fn(u32, u32) -> u32;

const LOW_SDD_DATA: usize = 0x0000_7500;
const LOW_SDD_LENGTH: usize = 0x0000_8464;
const LOW_CONFIGURATION_VALID: usize = 0x0000_847e;

unsafe extern "C" {
    static __diagnostic_output_base: u8;
    static __diagnostic_checkpoint_base: u8;
}

#[inline(always)]
fn diagnostic_output() -> *mut u32 {
    (&raw const __diagnostic_output_base).cast_mut().cast()
}

#[inline(always)]
fn diagnostic_checkpoint() -> *mut u32 {
    (&raw const __diagnostic_checkpoint_base).cast_mut().cast()
}

unsafe fn low_sdd_element(id: u8) -> Option<&'static [u8]> {
    unsafe {
        if (LOW_CONFIGURATION_VALID as *const u8).read_volatile() != 1 {
            return None;
        }
        let total = (LOW_SDD_LENGTH as *const u32).read_volatile() as usize;
        if total > xr819_firmware::configuration::MAX_DPD_DATA_LEN {
            return None;
        }
        let data = core::slice::from_raw_parts(LOW_SDD_DATA as *const u8, total);
        let mut offset = 0;
        while offset + 2 <= data.len() {
            let length = usize::from(data[offset + 1]);
            let end = offset + 2 + length;
            if end > data.len() {
                return None;
            }
            if data[offset] == id {
                return Some(&data[offset + 2..end]);
            }
            offset = end;
        }
        None
    }
}

unsafe fn write_sdd_u16(id: u8, destination: usize) {
    if let Some([a, b, ..]) = unsafe { low_sdd_element(id) } {
        unsafe { (destination as *mut u16).write_volatile(u16::from_le_bytes([*a, *b])) };
    }
}

unsafe fn write_sdd_pair(id: u8, destination: usize) {
    if let Some([a, b, c, d, ..]) = unsafe { low_sdd_element(id) } {
        unsafe {
            (destination as *mut u16).write_volatile(u16::from_le_bytes([*a, *b]));
            ((destination + 2) as *mut u16).write_volatile(u16::from_le_bytes([*c, *d]));
        }
    }
}

/// Fill the gain/RSSI coefficients omitted by the byte-identical stable low
/// image, sourcing every value from the Linux-provided SDD retained at its
/// validated low-image addresses.
unsafe fn populate_extension_gain_sdd_state() {
    unsafe {
        // XR819 has only the profile-zero 2.4 GHz band. Keep this extension
        // bounded to the five profile-zero records consumed by the active
        // gain/RSSI path; profile-one state remains detached.
        write_sdd_u16(0x20, 0x0400_35ac);
        write_sdd_u16(0x22, 0x0400_35ae);
        write_sdd_pair(0x40, 0x0400_3500);
        write_sdd_u16(0x42, 0x0400_34fa);
        write_sdd_pair(0x46, 0x0400_34fc);
    }
}

unsafe fn snapshot_gain_state() {
    unsafe {
        let output = diagnostic_output();
        output.add(0).write_volatile(0x4753_4e50);
        output
            .add(1)
            .write_volatile(u32::from((0x0400_994e as *const u8).read_volatile()));
        output
            .add(2)
            .write_volatile((0x0400_9994 as *const u32).read_volatile());
        output
            .add(3)
            .write_volatile((0x0400_9998 as *const u32).read_volatile());
        output
            .add(4)
            .write_volatile((0x0400_1ff0 as *const u32).read_volatile());
        output
            .add(5)
            .write_volatile((0x0400_2000 as *const u32).read_volatile());
        output
            .add(6)
            .write_volatile((0x0400_2004 as *const u32).read_volatile());
        output
            .add(7)
            .write_volatile((0x0400_99f4 as *const u32).read_volatile());
        output
            .add(8)
            .write_volatile((0x0400_99d4 as *const u32).read_volatile());
        output
            .add(9)
            .write_volatile((0x0400_34f8 as *const u32).read_volatile());
        output
            .add(10)
            .write_volatile((0x0400_34fc as *const u32).read_volatile());
        output
            .add(11)
            .write_volatile((0x0400_3500 as *const u32).read_volatile());
        output
            .add(12)
            .write_volatile((0x0400_35ac as *const u32).read_volatile());
        output
            .add(13)
            .write_volatile((0x0abb_801c as *const u32).read_volatile());
        output
            .add(14)
            .write_volatile((0x0abb_8024 as *const u32).read_volatile());
        output
            .add(15)
            .write_volatile((0x0abb_8044 as *const u32).read_volatile());
    }
}

/// High-SRAM proof that split-loaded extension code can execute while the
/// byte-identical low startup retains HIF ownership.
#[unsafe(no_mangle)]
#[unsafe(link_section = ".text.entry")]
pub unsafe extern "C" fn xr819_hif_extension_probe(transport: *mut u8, length: u16) {
    unsafe { diagnostic_checkpoint().write_volatile(0x4558_5431) };
    let stable_publish: StablePublish = unsafe { core::mem::transmute(0x0000_162d_usize) };
    unsafe { stable_publish(transport, length) };
    unsafe { diagnostic_checkpoint().add(1).write_volatile(0x4558_5432) };
}

/// Preserve the stable channel-frequency division call while inserting the
/// mandatory vendor-compatible 16-slot gain build after channel-power state is
/// published by the low image.
#[unsafe(no_mangle)]
#[unsafe(link_section = ".text.gain_entry")]
pub unsafe extern "C" fn xr819_gain_extension(dividend: u32, divisor: u32) -> u32 {
    let stable_divide: StableDivide = unsafe { core::mem::transmute(0x0000_7081_usize) };
    let quotient = unsafe { stable_divide(dividend, divisor) };
    unsafe { populate_extension_gain_sdd_state() };
    let status = match unsafe { xr819_firmware::phy::program_all_tx_gain_slots(200) } {
        Ok(()) => 0x4741_494e,
        Err(_) => 0x4741_4946,
    };
    unsafe {
        diagnostic_checkpoint().add(2).write_volatile(status);
        diagnostic_checkpoint()
            .add(3)
            .write_volatile((0x0abb_801c as *const u32).read_volatile());
        snapshot_gain_state();
    }
    quotient
}

#[panic_handler]
fn panic(_info: &PanicInfo<'_>) -> ! {
    loop {
        core::hint::spin_loop();
    }
}
