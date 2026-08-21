//! XR819 platform setup performed before the vendor HIF initializer.
//!
//! This is an intentionally literal translation of the startup routines at
//! 0x000009e0, 0x000056d8, 0x0000572a, 0x000058b6, and 0x00000bb0. Interrupt
//! callback registration is kept separate because the Rust handlers do not
//! exist yet.

use crate::packet_ram;
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
    }
}

const CLOCK_RESET_BASE: usize = 0x0aa8_0000;
const SYSTEM_CONTROL_BASE: usize = 0x0a98_0000;
const INTERRUPT_CONTROLLER_BASE: usize = 0x0a88_0000;
const PERIPHERAL_CONTROL_BASE: usize = 0x0ac0_0000;
const PLATFORM_CONTROL_BASE: usize = 0x0ac8_0000;
const BOOT_STATE_BASE: usize = crate::dtcm::SCHEDULER_EVENT_ROOT.get();
const CLOCK_PARAMETERS_BASE: usize = crate::dtcm::CLOCK_PARAMETERS.get();
const INTERRUPT_ROUTING_BASE: usize = 0x0abb_0000;
const HOST_DOWNLOAD_STATE: usize = crate::dtcm::initialized_tsf_resync_state().get();
const HIF_SHARED_BASE: usize = 0x0ab0_0100;
const IRQ_CALLBACK_TABLE: usize = crate::dtcm::IRQ_CALLBACK_TABLE.get();
const VENDOR_BSS_START: usize = crate::dtcm::VENDOR_BSS_START.get();
const VENDOR_BSS_END: usize = crate::dtcm::VENDOR_BSS_END.get();

const MAC_REGISTER_BASE: usize = 0x09c0_0000;
const PACKET_CONTROLLER_REGISTER_BASE: usize = 0x09c1_0000;
const MIC_REGISTER_BASE: usize = 0x09c4_0000;
const AES_REGISTER_BASE: usize = 0x09c5_0000;
const TX_RING_REGISTER_BASE: usize = 0x09c6_0000;
const TX_RING_STRIDE: usize = 0x80;

/// Address one register in the sparse MAC/packet-DMA register aperture.
pub const fn mac_register(offset: usize) -> usize {
    MAC_REGISTER_BASE + offset
}

/// Address the unresolved single-word packet-controller aperture.
pub const fn packet_controller_register(offset: usize) -> usize {
    PACKET_CONTROLLER_REGISTER_BASE + offset
}

/// Address the quarantined MIC register aperture without inferring its extent.
pub const fn mic_register(offset: usize) -> usize {
    MIC_REGISTER_BASE + offset
}

/// Base of the decoded AES register block.
pub const fn aes_register_base() -> usize {
    AES_REGISTER_BASE
}

/// Address a register in one of the four hardware TX rings.
pub const fn tx_ring_register(pipe: usize, offset: usize) -> usize {
    debug_assert!(pipe < 4);
    TX_RING_REGISTER_BASE + pipe * TX_RING_STRIDE + offset
}

/// Address a register by its offset in the complete hardware TX-ring aperture.
pub const fn tx_ring_register_offset(offset: usize) -> usize {
    TX_RING_REGISTER_BASE + offset
}

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

#[inline(always)]
fn boot_word(offset: usize) -> &'static ReadWrite<u32> {
    register32(BOOT_STATE_BASE + offset)
}

#[inline(always)]
fn boot_read_u16(offset: usize) -> u16 {
    unsafe { (BOOT_STATE_BASE.wrapping_add(offset) as *const u16).read_volatile() }
}

#[inline(always)]
fn boot_write_u16(offset: usize, value: u16) {
    unsafe { (BOOT_STATE_BASE.wrapping_add(offset) as *mut u16).write_volatile(value) }
}

#[inline(always)]
fn clock_parameter_word(offset: usize) -> &'static ReadWrite<u32> {
    register32(CLOCK_PARAMETERS_BASE + offset)
}

#[inline(always)]
fn clock_parameter_read_u8(offset: usize) -> u8 {
    unsafe { (CLOCK_PARAMETERS_BASE.wrapping_add(offset) as *const u8).read_volatile() }
}

#[inline(always)]
fn clock_parameter_write_u8(offset: usize, value: u8) {
    unsafe { (CLOCK_PARAMETERS_BASE.wrapping_add(offset) as *mut u8).write_volatile(value) }
}

fn interrupt_routing() -> &'static InterruptRouting {
    unsafe { &*(INTERRUPT_ROUTING_BASE as *const InterruptRouting) }
}

fn register32(address: usize) -> &'static ReadWrite<u32> {
    unsafe { &*(address as *const ReadWrite<u32>) }
}

fn post_code(_value: u32) {}

extern "C" fn diagnostic_irq_stub() {}

/// Vendor IRQ 6 callback at `0x0000f1fe`: atomically set bit 27 in the
/// platform event word. The original masks IRQ/FIQ around this read-modify-write;
/// the interrupt dispatcher already invokes Rust callbacks with IRQs masked.
extern "C" fn scheduler_event_irq() {
    let events = register32(crate::dtcm::SCHEDULER_EVENT_ROOT.get());
    events.set(events.get() | (1 << 27));
}

/// Exact vendor IRQ 26 callback at `0x00007e3c`.
extern "C" fn packet_dma_irq26() {
    let status = register32(0x09c0_0e34);
    status.set(status.get() & 0x1f00_0003);
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

    // Initialized vendor SRAM supplies the scheduler/radio exclusion word
    // immediately before the event structure. Sectioned custom images do not
    // carry that data segment, so stale bootloader bits (notably 0x20) would
    // make `task_b88e` leave every class-0 frame pending.
    register32(crate::dtcm::scheduler_exclusion_mask().get()).set(0);
    register32(crate::dtcm::scheduler_secondary_exclusion().get()).set(0);

    boot_word(0x04).set(0);
    boot_write_u16(0x08, 7);
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
        (crate::dtcm::initialized_tsf_resync_state().get() as *mut u32).write_volatile(0);
        crate::tx::initialize_retry_random_state();
        (crate::dtcm::initialized_tsf_accumulator_low().get() as *mut u32).write_volatile(0);
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
    let selector = clock.window_selector.get() & !0x1c;
    let mut any = 0;

    for index in 0..8 {
        clock.window_selector.set(selector | ((index as u32) << 2));
        let value = clock.window_value.get();
        any |= value;
        boot_word(0x20 + index * 4).set(value);
    }

    boot_write_u16(0x1c, u16::from(any != 0));
    if any == 0 {
        return;
    }

    let window_3 = boot_word(0x2c).get();
    let window_4 = boot_word(0x30).get();
    let window_5 = boot_word(0x34).get();
    let relocated_3 = ((window_3 & 0x0fff_ffff) >> 12).wrapping_add(0x166);
    let relocated_4 = ((window_4 & 0x0fff_ffff) >> 12).wrapping_add(0x166);
    let relocated_high = (window_4 >> 28 | window_5 << 4).wrapping_add(0x166);

    boot_word(0x2c).set((window_3 & 0xf000_0fff) | (relocated_3 << 12));
    boot_word(0x30)
        .set((window_4 & 0x0000_0fff) | (relocated_4 << 12) | (relocated_high << 28));
    boot_word(0x34).set((window_5 & 0xffff_f000) | ((relocated_high & 0xffff) >> 4));
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
    let _ = clock.window_value.get();
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
    let remap_control = if boot_read_u16(0x1c) != 0 {
        let value = boot_word(0x20).get();
        let remap_flag = ((value & 0x0800_0000) >> 19) as u16;
        boot_write_u16(0x08, boot_read_u16(0x08) | remap_flag);
        value
    } else {
        0
    };

    if remap_control & 0x0800_0000 == 0 {
        boot_write_u16(0x12, 3);
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
                boot_write_u16(0x12, 1);
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
        .set(unsafe { (crate::dtcm::initialized_tsf_accumulator_low().get() as *const u32).read_volatile() });
    interrupts.control.set(1);

    clock.mask_44.set(clock.mask_44.get() & 0x7fff_f777);
    clock.mask_48.set(clock.mask_48.get() & 0x777f_ffff);
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
    if boot_read_u16(0x1c) == 0 {
        platform
            .configuration
            .set(platform.configuration.get() | 0x0012_0000);
    }
    post_code(0x444d_4303);

    // Preserve the exact 0x00000bb0 transition. Loss of the 0x0900ff98 alias
    // after this write is not evidence that the CPU stopped executing.
    post_code(0x444d_4331);
    clock.control_04.set(0x200);
    boot_word(0x04).set(1);
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

    if platform.clock_status.get() & 8 == 0 {
        clock_parameter_write_u8(0x1c, 1);
    }
    post_code(0x444d_4306);

    let alternate = clock_parameter_read_u8(0x1c) != 0;
    let flags = if alternate { 0x0c02 } else { 0x0042 };
    if alternate {
        clock_parameter_word(0x24).set(0x30c);
    }
    clock_parameter_word(0x10).set(flags);
    platform.clock_flags.set(flags);
    clock_parameter_word(0x0c).set(if alternate { 0x0001_e000 } else { 0x0001_3130 });
    clock_parameter_word(0x20).set(if alternate { 0x0098_9680 } else { 0x0131_2d00 });
    clock_parameter_word(0x18).set(0x5640);
    post_code(0x444d_4307);
}

/// Translates the fixed packet-memory/DMA register setup reached through
/// `FUN_000000f6`, `FUN_000000bc`, `FUN_0000f608`, `FUN_0000f4f4`, and
/// `FUN_0000fcfe` before the vendor startup indication is queued.
/// Vendor `0x00000c38`, `0x00000a88`, and `0x00000b88`, called in this
/// order after HIF activation and before `0x000009ac`.
pub fn register_post_activation_interrupts() {
    register_interrupt_source_with(6, scheduler_event_irq);
    #[cfg(target_arch = "arm")]
    {
        register_interrupt_source_with(18, crate::crypto::hardware_crypto_irq18);
        register_interrupt_source_with(20, crate::crypto::hardware_crypto_irq20);
    }
    #[cfg(not(target_arch = "arm"))]
    {
        register_interrupt_source(18);
        register_interrupt_source(20);
    }
    register_interrupt_source(21);
}

/// Register translated sources from the tail of vendor `fw_subsystem_init`
/// after packet-DMA and MAC/RX setup is complete.
pub fn register_packet_dma_interrupts() {
    register_interrupt_source_with(26, packet_dma_irq26);
    // Vendor IRQ 27 calls `event_send_error_0x34`. Do not enable it with a
    // no-op callback: an uncleared level source could livelock once CPU IRQ
    // delivery is unmasked. Register it only after the event allocator and
    // source-specific acknowledgement are translated.
}

/// Services vendor IRQ 26 while this firmware intentionally keeps CPU IRQ/FIQ
/// masked. This reproduces the generic demultiplexer's acknowledge-before-
/// callback order instead of merely registering a callback that cannot run.
pub fn service_masked_packet_dma_interrupt() -> bool {
    const IRQ26: u32 = 1 << 26;
    let pending = register32(INTERRUPT_CONTROLLER_BASE + 0x20).get() & IRQ26;
    if pending == 0 {
        return false;
    }
    interrupt_controller().clear_04.set(pending);
    packet_dma_irq26();
    true
}

pub fn prepare_mac_receive_hardware() {
    let control = 0x09c0_0040;
    for (offset, value) in [
        (0x000, 0x8000_ffd4),
        (0x004, 0x8000_ffc4),
        (0x008, 0x8000_ffb4),
        (0x00c, 0x8000_efe4),
        (0x010, 0x8000_ff84),
        (0x014, 0x8000_ff94),
        (0x018, 0x0000_ff80),
        (0x01c, 0x0000_8f88),
        (0x020, 0x0000_2020),
        (0x024, 0x0000_4040),
        (0x028, 0x0000_4000),
        (0x02c, 0x0004_0101),
    ] {
        register32(0x09c0_0000 + offset).set(value);
    }
    for (offset, value) in [
        (0x060, 0x0010_0101),
        (0x064, 0x8018_6000),
        (0x068, 0x0002_ff00),
        (0x06c, 0x0003_ff00),
        (0x070, 0x0000_ff74),
        (0x074, 0x0084_0604),
        (0x078, 0x0001_8080),
        (0x07c, 0x801a_0202),
        (0x080, 0x801d_0100),
        (0x084, 0x000c_0202),
        (0x088, 0x000f_0100),
        (0x08c, 0x0018_0202),
        (0x090, 0x8600_e3e2),
        (0x094, 0x860a_3f1d),
        (0x0b4, 0x001b_0100),
        (0x0b8, 0x0000_0f00),
        (0x0bc, 0x0084_2624),
        (0x0c0, 0x8000_0000),
        (0x0c4, 0x0002_0000),
        (0x0c8, 0x0016_0000),
        (0x0cc, 0xc018_0000),
        (0x0d0, 0xc010_0000),
        (0x0d4, 0xc012_0000),
        (0x0e0, 0x0004_0000),
        (0x0e8, 0x800a_0000),
        (0x0f0, 0x0010_0000),
    ] {
        register32(control + offset).set(value);
    }

    let timing = 0x09c0_0200;
    for (offset, value) in [
        (0x010, 0x1900_0000),
        (0x014, 0x1c00_0000),
        (0x018, 0x1000_0000),
        (0x01c, 0x4000_0000),
        (0x020, 0x8000_0000),
        (0x028, 0x2000_0002),
        (0x02c, 0x0100_0004),
        (0x030, 0x0400_0004),
        (0x034, 0x0180_0010),
        (0x038, 0x0480_0010),
        (0x03c, 0x2000_1010),
        (0x040, 0x2000_0020),
        (0x044, 0x2000_1020),
        (0x048, 0x0200_0040),
        (0x04c, 0x0000_0800),
        (0x050, 0x2000_2080),
        (0x054, 0x2000_0001),
        (0x05c, 0x0100_0000),
        (0x060, 0x0400_0000),
        (0x064, 0x0000_c380),
        (0x068, 8),
        (0x06c, 0),
        (0x074, 0x2080_0010),
        (0x078, 0x2082_0080),
        (0x07c, 0x0000_2000),
        (0x098, 0),
        (0x0c0, 0),
        (0x0e4, 0),
        (0x0e8, 0x80),
        (0x0f0, 0x0002_0004),
        (0x0f4, 0x001e_0080),
        (0x0f8, 0x0006_0000),
        (0x0fc, 0x0067_0000),
    ] {
        register32(timing + offset).set(value);
    }
    register32(0x09c0_0a14).set(0);
    register32(0x09c0_0a18).set(0x8000);
    register32(0x09c0_0308).set(0x70);
    register32(0x09c0_02ec).set(4);
}

#[cfg(test)]
trait StationAddressIo {
    fn write_low_mac_byte(&mut self, address: crate::dtcm::DtcmAddress, value: u8);
    fn write_vif_bytes(&mut self, index: usize, value: u8);
}

#[cfg(test)]
fn program_station_address_bytes_with_io<I: StationAddressIo>(address: [u8; 6], io: &mut I) {
    for (index, value) in address.into_iter().enumerate() {
        io.write_low_mac_byte(crate::dtcm::low_mac_own_mac_byte_unchecked(0, index), value);
        io.write_low_mac_byte(crate::dtcm::low_mac_own_mac_byte_unchecked(1, index), value);
        io.write_vif_bytes(index, value);
    }
}

pub fn program_station_address(address: [u8; 6]) {
    for (index, value) in address.into_iter().enumerate() {
        unsafe {
            (crate::dtcm::low_mac_own_mac_byte_unchecked(0, index).get() as *mut u8)
                .write_volatile(value);
            (crate::dtcm::low_mac_own_mac_byte_unchecked(1, index).get() as *mut u8)
                .write_volatile(value);
        }
        crate::vif::set_all_own_mac_byte(index, value);
    }
    let low = u32::from_le_bytes([address[0], address[1], address[2], address[3]]);
    let high = u32::from(u16::from_le_bytes([address[4], address[5]]));
    register32(0x09c0_0030).set(low);
    register32(0x09c0_0034).set(high);
    register32(0x09c0_0038).set(0x101);
    register32(0x09c0_0048).set(low);
    register32(0x09c0_004c).set(high);
    register32(0x09c0_0050).set(0x101);
}

pub fn prepare_packet_dma() {
    register32(0x0ab9_8040).set(0);
    register32(0x09c0_080c).set(0);

    // Vendor order, from `mac_hw_reset_regs` (`annotated-main.c:469-477`):
    // the auxiliary configuration at +0x0c goes in BEFORE the main control word
    // at +0x00, and the producer/consumer cursors are cleared only afterwards.
    // Our previous order wrote the control word first, exposing the controller
    // to stale +0x0c and cursor values across that transition. Final register
    // image is identical either way, so this only matters if +0x00 latches
    // internal routing state -- which is exactly what we are testing, since
    // vendor never leaks TX command-fetch bytes into the RX FIFO and we do.
    register32(0x09c0_060c).set(0x80);
    register32(0x09c0_0600).set(0x0102_0418);
    register32(0x09c0_061c).set(7);
    register32(0x09c0_0604).set(0);
    register32(0x09c0_0608).set(0);
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
    // Vendor `tsf_hw_init` (`annotated-main.c:18643-18659`) clears +0xe34 to 0
    // and only then writes 3, producing a 0 -> 3 edge. We wrote 3 directly.
    // Not equivalent if the register has sticky or write-one-to-set semantics.
    register32(0x09c0_0e34).set(0);
    register32(0x09c0_0e38).set(0);
    register32(0x09c0_0e3c).set(0);
    register32(0x09c0_0e34).set(3);
    post_code(0x5044_4d02);

    register32(0x09c0_0e60).set(2);
    register32(0x09c0_0e80).set(0x100);
    register32(0x09c0_0e88).set(0xff);
    post_code(0x5044_4d21);

    for (first_pipe, second_pipe) in [(0, 1), (2, 3)] {
        register32(tx_ring_register(first_pipe, 0x0c))
            .set(packet_ram::tx_command(first_pipe, 0) as u32 & 0x007f_ffff);
        register32(tx_ring_register(first_pipe, 0x10)).set(packet_ram::TX_COMMAND_SIZE as u32);
        register32(tx_ring_register(first_pipe, 0x14)).set(1);
        register32(tx_ring_register(second_pipe, 0x0c))
            .set(packet_ram::tx_command(second_pipe, 0) as u32 & 0x007f_ffff);
        register32(tx_ring_register(second_pipe, 0x10)).set(packet_ram::TX_COMMAND_SIZE as u32);
        register32(tx_ring_register(second_pipe, 0x14)).set(1);
    }
    post_code(0x5044_4d22);

    // Tail of FUN_0000f4f4. The earlier failure here was caused by keeping the
    // Rust stack in the 0xfffxxxxx window across the 0x0aa80004 transition;
    // with the vendor low SRAM stack, this packet-memory window is accessible.
    // Vendor `mac_hw_init_pipes` calls `phy_set_band_reg(1)` before it
    // publishes the automatic-response command list. Keep that ordering: the
    // final register/RAM image is identical, but +0xe8c may latch response-list
    // interpretation across the transition.
    register32(0x09c0_0e8c).set(0xbf);
    post_code(0x5044_4d03);
    let list_base = packet_ram::automatic_response_list().start;
    register32(list_base).set(0x4e14_0000);
    for index in 0..33 {
        register32(list_base + 4 + index * 4).set(0x2200_0000 | (list_base as u32 & 0x007f_ffff));
    }
    register32(list_base + 4 + 33 * 4).set(0xf000_0000);
    post_code(0x5044_4d23);

    // Hardware-visible head of FUN_00000044, called here by FUN_000000f6.
    register32(0x09c0_1300).set(0);
    register32(0x09c0_1300).set(0x0100_0000);
    for index in 0..4 {
        let slot = 0x09c0_0060 + index * 0x0c;
        register32(slot).set(u32::MAX);
        register32(slot + 4).set(0xff);
        register32(slot + 8).set(0xff);
    }
    post_code(0x5044_4d04);

    // Vendor `hif_dbg_ctx_init` (`annotated-main.c:19298-19313`) writes the
    // base/control register at +0x00 LAST, after its parameters. We wrote it
    // first, which transiently publishes whatever +0x04..+0x10 held.
    register32(0x09c0_1404).set(0);
    register32(0x09c0_1408).set(0);
    register32(0x09c0_140c).set(0x7ff);
    register32(0x09c0_1410).set(0);
    register32(0x09c0_1400).set(0);
    post_code(0x5044_4d05);

    // Vendor `0xf6 -> 0x4c6`: four software-owned packet-RAM records.
    // The final record terminates the free list rather than wrapping.
    register32(crate::dtcm::MAC_SOFTWARE_RECORDS.get()).set(crate::dtcm::MAC_SOFTWARE_RECORDS.get() as u32 + 4);
    for index in 0..4 {
        let record = crate::dtcm::MAC_SOFTWARE_RECORDS.get() + index * 8;
        register32(record + 4).set(if index == 3 {
            0
        } else {
            (record + 0x0c) as u32
        });
        register32(record + 8).set(packet_ram::software_record(index) as u32);
    }
    post_code(0x5044_4d06);
}

/// Vendor `0x9ac -> 0xc80 -> 0x1a2b0`, immediately before startup
/// indication construction.
pub fn enable_packet_controller() {
    register32(0x09c0_1000).set(1);
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

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    struct ByteWrite {
        address: usize,
        width: u8,
        value: u8,
    }

    #[derive(Default)]
    struct StationAddressRecorder {
        writes: std::vec::Vec<ByteWrite>,
    }

    impl StationAddressIo for StationAddressRecorder {
        fn write_low_mac_byte(&mut self, address: crate::dtcm::DtcmAddress, value: u8) {
            self.writes.push(ByteWrite { address: address.get(), width: 1, value });
        }

        fn write_vif_bytes(&mut self, index: usize, value: u8) {
            for interface in 0..3 {
                let record = crate::dtcm::vif_record(interface).unwrap();
                self.writes.push(ByteWrite {
                    address: record.own_mac_byte(index).unwrap().get(),
                    width: 1,
                    value,
                });
            }
        }
    }

    #[test]
    fn station_address_bytes_preserve_parent_address_width_and_order() {
        let address = [0x10, 0x21, 0x32, 0x43, 0x54, 0x65];
        let mut recorder = StationAddressRecorder::default();
        program_station_address_bytes_with_io(address, &mut recorder);

        let mut expected = std::vec::Vec::new();
        for (index, value) in address.into_iter().enumerate() {
            expected.push(ByteWrite {
                address: crate::dtcm::low_mac_own_mac_byte_unchecked(0, index).get(),
                width: 1,
                value,
            });
            expected.push(ByteWrite {
                address: crate::dtcm::low_mac_own_mac_byte_unchecked(1, index).get(),
                width: 1,
                value,
            });
            for interface in 0..3 {
                expected.push(ByteWrite {
                    address: crate::dtcm::vif_record(interface)
                        .unwrap()
                        .own_mac_byte(index)
                        .unwrap()
                        .get(),
                    width: 1,
                    value,
                });
            }
        }
        assert_eq!(recorder.writes, expected);
    }
}
