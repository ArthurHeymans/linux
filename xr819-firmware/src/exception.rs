use core::cell::UnsafeCell;

pub const EXCEPTION_MAGIC: u32 = 0x5843_4651; // "XCFQ"
pub const EXCEPTION_KIND_PANIC: u32 = 0x100;

#[repr(C)]
#[derive(Clone, Copy)]
pub struct ExceptionRecord {
    pub valid: u32,
    pub kind: u32,
    pub registers: [u32; 13],
    pub exception_sp: u32,
    pub exception_lr: u32,
    pub spsr: u32,
    pub cpsr: u32,
    pub fault_pc: u32,
    pub dfsr: u32,
    pub dfar: u32,
}

impl ExceptionRecord {
    pub const fn empty() -> Self {
        Self {
            valid: 0,
            kind: 0,
            registers: [0; 13],
            exception_sp: 0,
            exception_lr: 0,
            spsr: 0,
            cpsr: 0,
            fault_pc: 0,
            dfsr: 0,
            dfar: 0,
        }
    }
}

#[repr(transparent)]
pub struct ExceptionStorage(UnsafeCell<ExceptionRecord>);

// Exception handlers permanently quiesce the core before publishing this
// record. The section is deliberately excluded from the normal BSS clear so a
// software restart can still inspect the previous terminal fault.
unsafe impl Sync for ExceptionStorage {}

#[unsafe(no_mangle)]
#[unsafe(link_section = ".noinit.exception")]
pub static XR819_EXCEPTION_RECORD: ExceptionStorage =
    ExceptionStorage(UnsafeCell::new(ExceptionRecord::empty()));

#[cfg(target_arch = "arm")]
fn publish_record(label: &[u8]) -> ! {
    let record = unsafe { &*XR819_EXCEPTION_RECORD.0.get() };
    let registers = [
        record.kind,
        record.fault_pc,
        record.exception_lr,
        record.exception_sp,
        record.spsr,
        record.cpsr,
        record.dfsr,
        record.dfar,
        record.registers[0],
        record.registers[1],
        record.registers[2],
        record.registers[3],
        record.registers[4],
        record.registers[5],
        record.registers[6],
        record.registers[7],
        record.registers[8],
        record.registers[9],
    ];
    unsafe { crate::hif::publish_terminal_exception(registers, label) };
    loop {
        core::hint::spin_loop();
    }
}

/// Terminal continuation entered by the ARM exception veneers after the core
/// register record has been committed. It must never attempt to resume the
/// faulting context.
#[cfg(target_arch = "arm")]
#[unsafe(no_mangle)]
pub extern "C" fn xr819_exception_terminal() -> ! {
    publish_record(b"xr819-cpu-exception")
}

/// Records a Rust panic distinctly from an architected CPU exception.
#[cfg(target_arch = "arm")]
pub fn panic_terminal(line: u32, column: u32, file_hash: u32) -> ! {
    let record = XR819_EXCEPTION_RECORD.0.get();
    unsafe {
        core::ptr::addr_of_mut!((*record).valid).write_volatile(0);
        core::ptr::addr_of_mut!((*record).kind).write_volatile(EXCEPTION_KIND_PANIC);
        core::ptr::addr_of_mut!((*record).fault_pc).write_volatile(line);
        core::ptr::addr_of_mut!((*record).exception_lr).write_volatile(column);
        core::ptr::addr_of_mut!((*record).exception_sp).write_volatile(file_hash);
        core::ptr::addr_of_mut!((*record).valid).write_volatile(EXCEPTION_MAGIC);
    }
    publish_record(b"xr819-rust-panic")
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::mem::{offset_of, size_of};

    #[test]
    fn assembly_record_layout_is_stable() {
        assert_eq!(offset_of!(ExceptionRecord, valid), 0);
        assert_eq!(offset_of!(ExceptionRecord, kind), 4);
        assert_eq!(offset_of!(ExceptionRecord, registers), 8);
        assert_eq!(offset_of!(ExceptionRecord, exception_sp), 60);
        assert_eq!(offset_of!(ExceptionRecord, exception_lr), 64);
        assert_eq!(offset_of!(ExceptionRecord, spsr), 68);
        assert_eq!(offset_of!(ExceptionRecord, cpsr), 72);
        assert_eq!(offset_of!(ExceptionRecord, fault_pc), 76);
        assert_eq!(offset_of!(ExceptionRecord, dfsr), 80);
        assert_eq!(offset_of!(ExceptionRecord, dfar), 84);
        assert_eq!(size_of::<ExceptionRecord>(), 88);
    }
}
