use crate::{
    ErrorStatus, Mode, TimerDivideConfiguration, Version,
    local_vector::{
        CMCI, Error, LINT0, LINT1, LocalVector, PerformanceMonitors, ThermalSensor, Timer,
        TimerMode,
    },
};
use bit_field::BitField;

#[repr(u32)]
#[derive(Debug, Clone, Copy)]
#[allow(non_camel_case_types)]
enum Register {
    ID = 0x802,
    VERSION = 0x803,
    TASK_PRIORITY = 0x808,
    PROCESSOR_PRIORITY = 0x80A,
    END_OF_INTERRUPT = 0x80B,
    LOCAL_DESTINATION = 0x80D,
    SPURIOUS_VECTOR = 0x80F,
    ERROR_STATUS = 0x828,
    CMCI_VECTOR = 0x802F,
    INTERRUPT_COMMAND = 0x830,
    TIMER_VECTOR = 0x832,
    THERMAL_SENSOR_VECTOR = 0x833,
    PERFORMANCE_MONITORS_VECTOR = 0x834,
    LINT0_VECTOR = 0x835,
    LINT1_VECTOR = 0x836,
    ERROR_VECTOR = 0x837,
    TIMER_INITIAL_COUNT = 0x838,
    TIMER_CURRENT_COUNT = 0x839,
    TIMER_DIVIDE_CONFIGURATION = 0x83E,
}

/// Reads from the model-specific register at the provided `address`.
///
/// # Safety
///
///
#[inline(always)]
fn read_register(register: Register) -> u64 {
    let value_low: u64;
    let value_high: u64;

    // Safety: Reading from a model-specific register cannot create undefined behaviour.
    unsafe {
        core::arch::asm!(
            "rdmsr",
            in("ecx") register as u32,
            out("edx") value_high,
            out("eax") value_low,
            options(nostack, nomem, preserves_flags)
        );
    }

    (value_high << 32) | value_low
}

/// Writes `value` to the model-specific register at the provided `address`.
#[inline(always)]
fn write_register(register: Register, value: u64) {
    let value_low = value & 0xFFFF;
    let value_high = value >> 32;

    // Safety: Writing to x2 APIC model-specific registers cannot create undefined behaviour.
    unsafe {
        core::arch::asm!(
            "wrmsr",
            in("ecx") register as u32,
            in("edx") value_high,
            in("eax") value_low,
            options(nostack, nomem, preserves_flags)
        );
    }
}

struct x2;

impl Mode for x2 {
    type Inner = ();

    fn get_id() -> u8 {
        let raw = read_register(Register::ID);
        u8::try_from(raw.get_bits(24..)).unwrap()
    }

    fn get_version() -> Version {
        let raw = u32::try_from(read_register(Register::VERSION)).unwrap();
        Version(raw)
    }

    fn get_task_priority() -> TaskPriority {
        todo!()
    }

    fn set_task_priority(value: TaskPriority) {
        todo!()
    }

    fn get_arbitration_priority() -> ArbitrationPriority {
        todo!()
    }

    fn get_processor_priority() -> ProcessorPriority {
        todo!()
    }

    fn get_remote_read() -> RemoteRead {
        todo!()
    }

    fn get_local_destination() -> LocalDestination {
        todo!()
    }

    fn get_error_status() -> ErrorStatus {
        let raw = u32::try_from(read_register(Register::ERROR_STATUS)).unwrap();
        ErrorStatus::from_bits_truncate(raw)
    }

    fn clear_error_status() {
        write_register(Register::ERROR_STATUS, 0x0);
    }

    fn get_timer_initial_count() -> u32 {
        u32::try_from(read_register(Register::TIMER_INITIAL_COUNT)).unwrap()
    }

    fn set_timer_initial_count(value: u32) {
        write_register(Register::TIMER_INITIAL_COUNT, u64::from(value));
    }

    fn get_timer_current_count() -> u32 {
        u32::try_from(read_register(Register::TIMER_CURRENT_COUNT)).unwrap()
    }

    fn get_timer_divide_configuration() -> TimerDivideConfiguration {
        let raw = u32::try_from(read_register(Register::TIMER_DIVIDE_CONFIGURATION)).unwrap();
        TimerDivideConfiguration::from_bits_truncate(raw)
    }

    fn set_timer_divide_configuration(value: TimerDivideConfiguration) {
        write_register(
            Register::TIMER_DIVIDE_CONFIGURATION,
            u64::from(value.bits()),
        );
    }

    fn send_interrupt_command(interrupt_command: crate::InterruptCommand) {
        let high = u64::from(interrupt_command.high());
        let low = u64::from(interrupt_command.low());

        assert!(
            low.get_bits(8..11) != 0b001,
            "x2 APIC does not support low priority delivery mode"
        );

        write_register(Register::INTERRUPT_COMMAND, (high << 32) | low);
    }

    fn get_spurious_vector() -> SpuriousInterruptVector {
        todo!()
    }

    fn set_spurious_vector(value: SpuriousInterruptVector) {
        todo!()
    }

    fn get_timer_vector() -> LocalVector<Timer> {
        let raw = u32::try_from(read_register(Register::TIMER_VECTOR)).unwrap();
        // Safety: `raw` is a valid `u32` read directly from the respective entry in the local vector table.
        unsafe { LocalVector::<Timer>::new(raw) }
    }

    fn set_timer_vector(value: LocalVector<Timer>) {
        // IA32 SDM instructs utilizing the `mfence` instruction to ensure all writes to the IA32_TSC_DEADLINE
        // MSR are serialized *after* the APIC timer mode switch (`wrmsr` to `IA32_TSC_DEADLINE` is non-serializing).
        if value.get_mode() == TimerMode::TscDeadline {
            // Safety: `mfence` has no safety implications.
            unsafe {
                core::arch::x86_64::_mm_mfence();
            }
        }

        write_register(Register::TIMER_VECTOR, u64::from(value));
    }

    fn get_cmci_vector() -> LocalVector<CMCI> {
        let raw = u32::try_from(read_register(Register::CMCI_VECTOR)).unwrap();
        // Safety: `raw` is a valid `u32` read directly from the respective entry in the local vector table.
        unsafe { LocalVector::<CMCI>::new(raw) }
    }

    fn set_cmci_vector(value: LocalVector<CMCI>) {
        write_register(Register::CMCI_VECTOR, u64::from(value));
    }

    fn get_lint0_vector() -> LocalVector<LINT0> {
        let raw = u32::try_from(read_register(Register::LINT0_VECTOR)).unwrap();
        // Safety: `raw` is a valid `u32` read directly from the respective entry in the local vector table.
        unsafe { LocalVector::<LINT0>::new(raw) }
    }

    fn set_lint0_vector(value: LocalVector<LINT0>) {
        write_register(Register::LINT0_VECTOR, u64::from(value));
    }

    fn get_lint1_vector() -> LocalVector<LINT1> {
        let raw = u32::try_from(read_register(Register::LINT1_VECTOR)).unwrap();
        // Safety: `raw` is a valid `u32` read directly from the respective entry in the local vector table.
        unsafe { LocalVector::<LINT1>::new(raw) }
    }

    fn set_lint1_vector(value: LocalVector<LINT1>) {
        write_register(Register::LINT1_VECTOR, u64::from(value));
    }

    fn get_error_vector() -> LocalVector<Error> {
        let raw = u32::try_from(read_register(Register::ERROR_VECTOR)).unwrap();
        // Safety: `raw` is a valid `u32` read directly from the respective entry in the local vector table.
        unsafe { LocalVector::<Error>::new(raw) }
    }

    fn set_error_vector(value: LocalVector<Error>) {
        write_register(Register::ERROR_VECTOR, u64::from(value));
    }

    fn get_performance_monitors_vector() -> LocalVector<PerformanceMonitors> {
        let raw = u32::try_from(read_register(Register::PERFORMANCE_MONITORS_VECTOR)).unwrap();
        // Safety: `raw` is a valid `u32` read directly from the respective entry in the local vector table.
        unsafe { LocalVector::<PerformanceMonitors>::new(raw) }
    }

    fn set_performance_monitors_vector(value: LocalVector<PerformanceMonitors>) {
        write_register(Register::PERFORMANCE_MONITORS_VECTOR, u64::from(value));
    }

    fn get_thermal_sensor_vector() -> LocalVector<ThermalSensor> {
        let raw = u32::try_from(read_register(Register::THERMAL_SENSOR_VECTOR)).unwrap();
        // Safety: `raw` is a valid `u32` read directly from the respective entry in the local vector table.
        unsafe { LocalVector::<ThermalSensor>::new(raw) }
    }

    fn set_thermal_sensor_vector(value: LocalVector<ThermalSensor>) {
        write_register(Register::THERMAL_SENSOR_VECTOR, u64::from(value));
    }

    fn end_of_interrrupt() {
        write_register(Register::END_OF_INTERRUPT, 0x0);
    }
}
