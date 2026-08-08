//! XR819 platform setup performed before the vendor HIF initializer.
//!
//! This is an intentionally literal translation of the startup routines at
//! 0x000009e0, 0x000056d8, 0x0000572a, 0x000058b6, and 0x00000bb0. Interrupt
//! callback registration is kept separate because the Rust handlers do not
//! exist yet.

use tock_registers::interfaces::{Readable, Writeable};
use tock_registers::register_structs;
use tock_registers::registers::ReadWrite;

register_structs! {
    ClockReset {
        (0x00 => _reserved0),
        (0x04 => control_04: ReadWrite<u32>),
        (0x08 => window_selector: ReadWrite<u32>),
        (0x0c => _reserved1),
        (0x18 => feature_control: ReadWrite<u32>),
        (0x1c => _reserved2),
        (0x2c => window_value: ReadWrite<u32>),
        (0x30 => _reserved3),
        (0x40 => mask_40: ReadWrite<u32>),
        (0x44 => mask_44: ReadWrite<u32>),
        (0x48 => mask_48: ReadWrite<u32>),
        (0x4c => @END),
    },

    SystemControl {
        (0x00 => control: ReadWrite<u32>),
        (0x04 => status: ReadWrite<u32>),
        (0x08 => secondary_control: ReadWrite<u32>),
        (0x0c => @END),
    },

    InterruptController {
        (0x00 => control: ReadWrite<u32>),
        (0x04 => clear_04: ReadWrite<u32>),
        (0x08 => vector_base: ReadWrite<u32>),
        (0x0c => enable: ReadWrite<u32>),
        (0x10 => _reserved0),
        (0x14 => clear_14: ReadWrite<u32>),
        (0x18 => configuration: ReadWrite<u32>),
        (0x1c => @END),
    },

    PeripheralControl {
        (0x00 => clear: ReadWrite<u32>),
        (0x04 => _reserved0),
        (0x08 => mode_08: ReadWrite<u32>),
        (0x0c => _reserved1),
        (0x10 => mode_10: ReadWrite<u32>),
        (0x14 => _reserved2),
        (0x20 => mode_20: ReadWrite<u32>),
        (0x24 => mode_24: ReadWrite<u32>),
        (0x28 => @END),
    },

    PlatformControl {
        (0x00 => _reserved0),
        (0x58 => shared_pointer: ReadWrite<u32>),
        (0x5c => _reserved1),
        (0x64 => power_control: ReadWrite<u32>),
        (0x68 => _reserved2),
        (0x80 => ready_status: ReadWrite<u32>),
        (0x84 => clock_flags: ReadWrite<u32>),
        (0x88 => _reserved3),
        (0x94 => clock_status: ReadWrite<u32>),
        (0x98 => _reserved4),
        (0x9c => parameter_9c: ReadWrite<u32>),
        (0xa0 => parameter_a0: ReadWrite<u32>),
        (0xa4 => _reserved5),
        (0xbc => configuration: ReadWrite<u32>),
        (0xc0 => @END),
    },

    BootState {
        (0x00 => _reserved0),
        (0x04 => platform_ready: ReadWrite<u32>),
        (0x08 => flags_08: ReadWrite<u16>),
        (0x0a => _reserved1),
        (0x12 => startup_mode: ReadWrite<u16>),
        (0x14 => _reserved2),
        (0x1c => remap_present: ReadWrite<u16>),
        (0x1e => _reserved3),
        (0x20 => remap_windows: [ReadWrite<u32>; 8]),
        (0x40 => @END),
    },

    InterruptRouting {
        (0x00 => _reserved0),
        (0x10 => route_10: ReadWrite<u32>),
        (0x14 => route_14: ReadWrite<u32>),
        (0x18 => route_18: ReadWrite<u32>),
        (0x1c => route_1c: ReadWrite<u32>),
        (0x20 => route_20: ReadWrite<u32>),
        (0x24 => route_24: ReadWrite<u32>),
        (0x28 => route_28: ReadWrite<u32>),
        (0x2c => route_2c: ReadWrite<u32>),
        (0x30 => route_30: ReadWrite<u32>),
        (0x34 => @END),
    },

    ClockParameters {
        (0x00 => _reserved0),
        (0x0c => reload: ReadWrite<u32>),
        (0x10 => flags: ReadWrite<u32>),
        (0x14 => _reserved1),
        (0x18 => quantum: ReadWrite<u32>),
        (0x1c => alternate_clock: ReadWrite<u8>),
        (0x1d => _reserved2),
        (0x20 => frequency: ReadWrite<u32>),
        (0x24 => divisor: ReadWrite<u32>),
        (0x28 => @END),
    }
}

const CLOCK_RESET_BASE: usize = 0x0aa8_0000;
const SYSTEM_CONTROL_BASE: usize = 0x0a98_0000;
const INTERRUPT_CONTROLLER_BASE: usize = 0x0a88_0000;
const PERIPHERAL_CONTROL_BASE: usize = 0x0ac0_0000;
const PLATFORM_CONTROL_BASE: usize = 0x0ac8_0000;
const BOOT_STATE_BASE: usize = 0x0400_1fd4;
const CLOCK_PARAMETERS_BASE: usize = 0x0400_218c;
const INTERRUPT_ROUTING_BASE: usize = 0x0abb_0000;
const HOST_DOWNLOAD_STATE: usize = 0x0400_1428;
const HIF_SOFTWARE_STATE_BASE: usize = 0x0400_9754;
const HIF_SHARED_BASE: usize = 0x0ab0_0100;
const IRQ_CALLBACK_TABLE: usize = 0x0400_11bc;
const VENDOR_BSS_START: usize = 0x0400_2078;
const VENDOR_BSS_END: usize = 0x0400_9c44;

#[unsafe(no_mangle)]
#[used]
pub static REMAP_DEBUG_INDEX: u32 = 0;

fn clock_reset() -> &'static ClockReset {
    unsafe { &*(CLOCK_RESET_BASE as *const ClockReset) }
}

fn system_control() -> &'static SystemControl {
    unsafe { &*(SYSTEM_CONTROL_BASE as *const SystemControl) }
}

fn interrupt_controller() -> &'static InterruptController {
    unsafe { &*(INTERRUPT_CONTROLLER_BASE as *const InterruptController) }
}

fn peripheral_control() -> &'static PeripheralControl {
    unsafe { &*(PERIPHERAL_CONTROL_BASE as *const PeripheralControl) }
}

fn platform_control() -> &'static PlatformControl {
    unsafe { &*(PLATFORM_CONTROL_BASE as *const PlatformControl) }
}

fn boot_state() -> &'static BootState {
    unsafe { &*(BOOT_STATE_BASE as *const BootState) }
}

fn clock_parameters() -> &'static ClockParameters {
    unsafe { &*(CLOCK_PARAMETERS_BASE as *const ClockParameters) }
}

fn interrupt_routing() -> &'static InterruptRouting {
    unsafe { &*(INTERRUPT_ROUTING_BASE as *const InterruptRouting) }
}

fn register32(address: usize) -> &'static ReadWrite<u32> {
    unsafe { &*(address as *const ReadWrite<u32>) }
}

fn post_code(value: u32) {
    register32(0x0900_ff98).set(value);
}

extern "C" fn diagnostic_irq_stub() {}

/// Vendor IRQ 6 callback at `0x0000f1fe`: atomically set bit 27 in the
/// platform event word. The original masks IRQ/FIQ around this read-modify-write;
/// the interrupt dispatcher already invokes Rust callbacks with IRQs masked.
extern "C" fn scheduler_event_irq() {
    let events = register32(0x0400_1fd4);
    events.set(events.get() | (1 << 27));
}

/// Literal side effects of vendor `0x00016148`: install the callback in the
/// reverse-ordered 32-entry table and enable the interrupt source.
fn register_interrupt_source_with(irq: u32, callback: extern "C" fn()) {
    let callback = callback as *const () as usize as u32 | 1;
    register32(IRQ_CALLBACK_TABLE + ((31 - irq) as usize * 4)).set(callback);
    let interrupts = interrupt_controller();
    interrupts.enable.set(interrupts.enable.get() | (1 << irq));
}

fn register_interrupt_source(irq: u32) {
    register_interrupt_source_with(irq, diagnostic_irq_stub);
}

/// Reproduces the startup-image data and BSS initialization needed by the
/// translated platform and HIF routines.
///
/// The vendor image loader fills `0x04002078..0x04009c44` with zero before
/// entering main firmware. The custom raw downloader does not process those
/// section records, so Rust must perform the equivalent initialization.
pub fn initialize_runtime_state() {
    let mut address = VENDOR_BSS_START;
    while address < VENDOR_BSS_END {
        unsafe { (address as *mut u32).write_volatile(0) };
        address += 4;
    }

    let state = boot_state();
    state.platform_ready.set(0);
    state.flags_08.set(7);
    unsafe {
        (BOOT_STATE_BASE.wrapping_add(0x0a) as *mut u16).write_volatile(9);
        (BOOT_STATE_BASE.wrapping_add(0x0c) as *mut u32).write_volatile(0x14a);
        (BOOT_STATE_BASE.wrapping_add(0x10) as *mut u32).write_volatile(0x0001_9600);
        (BOOT_STATE_BASE.wrapping_add(0x14) as *mut u32).write_volatile(0);
        (BOOT_STATE_BASE.wrapping_add(0x18) as *mut u32).write_volatile(0x200);
        (BOOT_STATE_BASE.wrapping_add(0x1c) as *mut u32).write_volatile(0);

        // The vendor container's initialized-SRAM segment supplies the zero
        // observed by 0x164bc at 0x04001428. Our flat custom image omits that
        // segment, so reproduce its loader effect before entering startup.
        (0x0400_1428 as *mut u32).write_volatile(0);
        (0x0400_142c as *mut u32).write_volatile(0x1234_5678);
        (0x0400_1430 as *mut u32).write_volatile(0);

        (0x0400_11b4 as *mut u32).write_volatile(0x0204_0a01);
        (0x0400_11b8 as *mut u32).write_volatile(0x0000_1f40);
    }
}

/// Waits at the first instruction of `firmware_main_initialize` until the
/// downloader/host releases main-firmware startup.
pub fn wait_for_host_download_completion(max_polls: u32) -> bool {
    for _ in 0..max_polls {
        if unsafe { (HOST_DOWNLOAD_STATE as *const u32).read_volatile() } == 0 {
            return true;
        }
        core::hint::spin_loop();
    }
    false
}

/// Samples and records the eight hardware remap windows (`FUN_00015944`).
fn sample_remap_windows() {
    let clock = clock_reset();
    let state = boot_state();
    let selector = clock.window_selector.get() & !0x1c;
    let mut any = 0;

    for (index, saved) in state.remap_windows.iter().enumerate() {
        clock.window_selector.set(selector | ((index as u32) << 2));
        let value = clock.window_value.get();
        any |= value;
        saved.set(value);
    }

    state.remap_present.set(u16::from(any != 0));
    if any == 0 {
        return;
    }

    let window_3 = state.remap_windows[3].get();
    let window_4 = state.remap_windows[4].get();
    let window_5 = state.remap_windows[5].get();
    let relocated_3 = ((window_3 & 0x0fff_ffff) >> 12).wrapping_add(0x166);
    let relocated_4 = ((window_4 & 0x0fff_ffff) >> 12).wrapping_add(0x166);
    let relocated_high = (window_4 >> 28 | window_5 << 4).wrapping_add(0x166);

    state.remap_windows[3].set((window_3 & 0xf000_0fff) | (relocated_3 << 12));
    state.remap_windows[4]
        .set((window_4 & 0x0000_0fff) | (relocated_4 << 12) | (relocated_high << 28));
    state.remap_windows[5].set((window_5 & 0xffff_f000) | ((relocated_high & 0xffff) >> 4));
}

/// Captures the cold hardware remap-window state without applying the later
/// clock/reset transition.
pub fn capture_remap_debug() {
    sample_remap_windows();
    let clock = clock_reset();
    let selected = unsafe { (&raw const REMAP_DEBUG_INDEX).read_volatile() } as usize & 7;
    let selector = clock.window_selector.get() & !0x1c;
    clock
        .window_selector
        .set(selector | ((selected as u32) << 2));
    let output = register32(0x0900_ff98);
    output.set(clock.window_value.get());
    let _ = output.get();
}

/// Applies the control update at `firmware_main_initialize + 0x0c` before
/// `FUN_000009e0`: clear bit 0 and set bit 23 at `0x0ac800bc`.
pub fn prepare_main_control() {
    let platform = platform_control();
    platform
        .configuration
        .set((platform.configuration.get() & !1) | 0x0080_0000);
}

/// Translates `FUN_000009e0`, including interrupt-controller initialization.
///
/// This leaves CPU IRQ/FIQ masked and does not install any firmware handlers.
pub fn prepare_memory_and_interrupts() {
    sample_remap_windows();

    let clock = clock_reset();
    let system = system_control();
    let state = boot_state();
    let remap_control = if state.remap_present.get() != 0 {
        let value = state.remap_windows[0].get();
        let remap_flag = ((value & 0x0800_0000) >> 19) as u16;
        state.flags_08.set(state.flags_08.get() | remap_flag);
        value
    } else {
        0
    };

    if remap_control & 0x0800_0000 == 0 {
        state.startup_mode.set(3);
        let saved_features = clock.feature_control.get();
        clock.feature_control.set(saved_features | 0x0018_0200);
        system.control.set((system.control.get() & !0x20) | 0x40);
        system
            .secondary_control
            .set(system.secondary_control.get() | 0x40);

        if system.status.get() & 0x20 != 0 {
            system
                .secondary_control
                .set(system.secondary_control.get() & !0x40);
            if system.status.get() & 0x20 == 0 {
                state.startup_mode.set(1);
            }
        }
        clock.feature_control.set(saved_features);
    }

    let peripheral = peripheral_control();
    peripheral.clear.set(u32::MAX);
    peripheral.mode_10.set(0x4f);
    peripheral.mode_08.set(0x80);
    peripheral.mode_24.set(0x4f);
    peripheral.mode_20.set(0);

    let interrupts = interrupt_controller();
    interrupts.configuration.set(0x1600_a037);
    interrupts.clear_14.set(u32::MAX);
    interrupts.clear_04.set(u32::MAX);
    interrupts
        .vector_base
        .set(((INTERRUPT_CONTROLLER_BASE as u32) << 9).wrapping_add(0));
    interrupts
        .enable
        .set(unsafe { (0x0400_1430 as *const u32).read_volatile() });
    interrupts.control.set(1);

    clock.mask_44.set(clock.mask_44.get() & 0x7fff_f777);
    clock.mask_48.set(clock.mask_48.get() & 0x777f_ffff);

    register32(0x0900_ff9c).set(state.remap_present.get() as u32);
    for (index, window) in state.remap_windows.iter().take(8).enumerate() {
        register32(0x0900_ffa0 + index * 4).set(window.get());
    }
}

/// Reproduces the hardware-visible part of `0x00000a74 -> 0xfff019aa`.
/// The vendor performs this between platform preparation and DMA/clock setup,
/// before registering IRQs 0 and 2.
pub fn prepare_high_platform_support() {
    // 0x00000a74 followed by the hardware/callback effects of 0xfff019aa.
    register32(0xfff0_32ec).set(1);
    register32(0x0a90_0004).set(0x2b);
    register_interrupt_source(0);
    register_interrupt_source(2);
}

/// Translates the hardware portion of `FUN_00000bb0` and `FUN_000058b6`.
///
/// IRQ 4 registration is deliberately omitted until its Rust handler exists.
pub fn prepare_dma_and_clocks() {
    post_code(0x444d_4300);
    let system = system_control();
    let clock = clock_reset();
    let platform = platform_control();
    let state = boot_state();

    system.control.set((system.control.get() & !0x80) | 0x10);
    system
        .secondary_control
        .set(system.secondary_control.get() & !0x10);
    post_code(0x444d_4301);
    clock.mask_48.set(clock.mask_48.get() | 0xf0);
    clock.mask_40.set(clock.mask_40.get() | 0x00f0_0000);
    post_code(0x444d_4302);

    platform.shared_pointer.set(0x0004_0090);
    platform.power_control.set(0x10);
    if state.remap_present.get() == 0 {
        platform
            .configuration
            .set(platform.configuration.get() | 0x0012_0000);
    }
    post_code(0x444d_4303);

    // Preserve the exact 0x00000bb0 transition. Loss of the 0x0900ff98 alias
    // after this write is not evidence that the CPU stopped executing.
    post_code(0x444d_4331);
    clock.control_04.set(0x200);
    state.platform_ready.set(1);
    post_code(0x444d_4332);
    platform.parameter_a0.set(31);
    post_code(0x444d_4333);
    platform.parameter_9c.set(6);
    register_interrupt_source(4);
    post_code(0x444d_4304);

    let mut attempts = 0;
    while platform.ready_status.get() & 1 == 0 && attempts < 0x200 {
        attempts += 1;
    }
    post_code(0x444d_4305);

    let parameters = clock_parameters();
    if platform.clock_status.get() & 8 == 0 {
        parameters.alternate_clock.set(1);
    }
    post_code(0x444d_4306);

    let alternate = parameters.alternate_clock.get() != 0;
    let flags = if alternate { 0x0c02 } else { 0x0042 };
    if alternate {
        parameters.divisor.set(0x30c);
    }
    parameters.flags.set(flags);
    platform.clock_flags.set(flags);
    parameters
        .reload
        .set(if alternate { 0x0001_e000 } else { 0x0001_3130 });
    parameters
        .frequency
        .set(if alternate { 0x0098_9680 } else { 0x0131_2d00 });
    parameters.quantum.set(0x5640);
    post_code(0x444d_4307);
}

/// Translates the fixed packet-memory/DMA register setup reached through
/// `FUN_000000f6`, `FUN_000000bc`, `FUN_0000f608`, `FUN_0000f4f4`, and
/// `FUN_0000fcfe` before the vendor startup indication is queued.
/// Vendor `0x00000c38`, `0x00000a88`, and `0x00000b88`, called in this
/// order after HIF activation and before `0x000009ac`.
pub fn register_post_activation_interrupts() {
    register_interrupt_source_with(6, scheduler_event_irq);
    register_interrupt_source(18);
    register_interrupt_source(20);
    register_interrupt_source(21);
}

pub fn prepare_packet_dma() {
    register32(0x0ab9_8040).set(0);
    register32(0x09c0_080c).set(0);

    register32(0x09c0_0600).set(0x0102_0418);
    register32(0x09c0_0604).set(0);
    register32(0x09c0_0608).set(0);
    register32(0x09c0_060c).set(0x80);
    register32(0x09c0_061c).set(7);
    register32(0x09c0_0620).set(0x3000_0000);

    register32(0x09c0_0200).set(0);
    register32(0x09c0_0204).set(0);
    register32(0x09c0_020c).set(3);
    register32(0x09c1_0000).set(0);
    register32(0x09c0_0a00).set(0x1030_0000);
    register32(0x09c0_0a04).set(0);
    register32(0x09c0_0a1c).set(0);
    post_code(0x5044_4d01);

    register32(0x09c0_0e00).set(0x4f00_004f);
    register32(0x09c0_0e04).set(0x0909_0000);
    for offset in (0x08..=0x2c).step_by(4) {
        register32(0x09c0_0e00 + offset).set(0);
    }
    register32(0x09c0_0e34).set(3);
    register32(0x09c0_0e38).set(0);
    register32(0x09c0_0e3c).set(0);
    post_code(0x5044_4d02);

    register32(0x09c0_0e60).set(2);
    register32(0x09c0_0e80).set(0x100);
    register32(0x09c0_0e88).set(0xff);
    post_code(0x5044_4d21);

    for (base, first, second) in [
        (0x09c6_0000, 0x0000_7080, 0x0000_71d0),
        (0x09c6_0100, 0x0000_7320, 0x0000_7470),
    ] {
        register32(base + 0x0c).set(first);
        register32(base + 0x10).set(0x54);
        register32(base + 0x14).set(1);
        register32(base + 0x8c).set(second);
        register32(base + 0x90).set(0x54);
        register32(base + 0x94).set(1);
    }
    post_code(0x5044_4d22);

    // Tail of FUN_0000f4f4. The earlier failure here was caused by keeping the
    // Rust stack in the 0xfffxxxxx window across the 0x0aa80004 transition;
    // with the vendor low SRAM stack, this packet-memory window is accessible.
    let list_base = 0x0901_6a28;
    register32(list_base).set(0x4e14_0000);
    for index in 0..33 {
        register32(list_base + 4 + index * 4).set(0x2201_6a28);
    }
    register32(list_base + 4 + 33 * 4).set(0xf000_0000);
    post_code(0x5044_4d23);
    register32(0x09c0_0e8c).set(0xbf);
    post_code(0x5044_4d03);

    // Hardware-visible head of FUN_00000044, called here by FUN_000000f6.
    register32(0x09c0_1300).set(0);
    register32(0x09c0_1300).set(0x0100_0000);
    post_code(0x5044_4d04);

    register32(0x09c0_1400).set(0);
    register32(0x09c0_1404).set(0);
    register32(0x09c0_1408).set(0);
    register32(0x09c0_140c).set(0x7ff);
    register32(0x09c0_1410).set(0);
    post_code(0x5044_4d05);
}

/// Applies the final hardware enable from `FUN_00016d24` after MAC/PHY state
/// initialization. Kept separate so its effect on HIF bring-up is measurable.
pub fn enable_mac_core() {
    let control = register32(0x0ab8_0c00);
    control.set(control.get() | (1 << 11));
}

/// Writes the fixed routing table immediately before the reference firmware
/// drains the ARM write buffer and enables CPU interrupts.
pub fn finalize_interrupt_routing() {
    let routing = interrupt_routing();
    routing.route_10.set(0);
    post_code(0x5254_0000);
    routing.route_14.set(0x3fd);
    post_code(0x5254_0001);
    routing.route_18.set(0x3fa);
    post_code(0x5254_0002);
    routing.route_1c.set(0x3ff);
    post_code(0x5254_0003);
    routing.route_20.set(0x1b);
    post_code(0x5254_0004);
    routing.route_24.set(0x5b);
    post_code(0x5254_0005);
    routing.route_28.set(0xb6);
    post_code(0x5254_0006);
    routing.route_2c.set(0x10a);
    post_code(0x5254_0007);
    routing.route_30.set(0x12c);
    post_code(0x5254_0008);
}

/// Performs the side-effect read and ready check from `FUN_00000ab4`.
///
/// This must run after [`prepare_dma_and_clocks`]. A bounded form is used for
/// bring-up so a bad prerequisite cannot trap the experiment indefinitely.
pub fn try_activate_hif(max_polls: u32) -> bool {
    unsafe {
        let _ = (HIF_SHARED_BASE.wrapping_add(0x40) as *const u32).read_volatile();
        (HIF_SOFTWARE_STATE_BASE as *mut u32).write_volatile(1);
        for _ in 0..max_polls {
            if (HIF_SHARED_BASE.wrapping_add(0x34) as *const u32).read_volatile() & (1 << 10) == 0 {
                return true;
            }
            core::hint::spin_loop();
        }
    }
    false
}

/// Exact unbounded ready wait used by the reference startup routine.
pub fn activate_hif_and_wait_ready() {
    while !try_activate_hif(u32::MAX) {}
}
