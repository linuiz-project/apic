use core::marker::PhantomData;

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

    fn get_id(_: Self::Inner) -> u32 {
        u32::try_from(read_register(Register::ID)).unwrap()
    }

    fn get_version(_: Self::Inner) -> Version {
        let raw = u32::try_from(read_register(Register::VERSION)).unwrap();
        Version(raw)
    }

    fn get_task_priority(_: Self::Inner) -> TaskPriority {
        todo!()
    }

    fn set_task_priority(_: Self::Inner, value: TaskPriority) {
        todo!()
    }

    fn get_arbitration_priority(_: Self::Inner) -> ArbitrationPriority {
        todo!()
    }

    fn get_processor_priority(_: Self::Inner) -> ProcessorPriority {
        todo!()
    }

    fn get_remote_read(_: Self::Inner) -> RemoteRead {
        todo!()
    }

    fn get_local_destination(_: Self::Inner) -> LocalDestination {
        todo!()
    }

    fn get_error_status(_: Self::Inner) -> ErrorStatus {
        let raw = u32::try_from(read_register(Register::ERROR_STATUS)).unwrap();
        ErrorStatus::from_bits_truncate(raw)
    }

    fn clear_error_status(_: Self::Inner) {
        write_register(Register::ERROR_STATUS, 0x0);
    }

    fn get_timer_initial_count(_: Self::Inner) -> u32 {
        u32::try_from(read_register(Register::TIMER_INITIAL_COUNT)).unwrap()
    }

    fn set_timer_initial_count(_: Self::Inner, value: u32) {
        write_register(Register::TIMER_INITIAL_COUNT, u64::from(value));
    }

    fn get_timer_current_count(_: Self::Inner) -> u32 {
        u32::try_from(read_register(Register::TIMER_CURRENT_COUNT)).unwrap()
    }

    fn get_timer_divide_configuration(_: Self::Inner) -> TimerDivideConfiguration {
        let raw = u32::try_from(read_register(Register::TIMER_DIVIDE_CONFIGURATION)).unwrap();
        TimerDivideConfiguration::from_bits_truncate(raw)
    }

    fn set_timer_divide_configuration(_: Self::Inner, value: TimerDivideConfiguration) {
        write_register(
            Register::TIMER_DIVIDE_CONFIGURATION,
            u64::from(value.bits()),
        );
    }

    fn send_interrupt_command(_: Self::Inner, interrupt_command: crate::InterruptCommand) {
        let high = u64::from(interrupt_command.high());
        let low = u64::from(interrupt_command.low());

        assert!(
            low.get_bits(8..11) != 0b001,
            "x2 APIC does not support low priority delivery mode"
        );

        write_register(Register::INTERRUPT_COMMAND, (high << 32) | low);
    }

    fn get_spurious_vector(_: Self::Inner) -> u8 {
        u8::try_from(read_register(Register::SPURIOUS_VECTOR).get_bits(..8)).unwrap()
    }

    fn get_spurious_apic_software_enabled(_: Self::Inner) -> bool {
        read_register(Register::SPURIOUS_VECTOR).get_bit(8)
    }

    fn get_spurious_focus_processor_checking(_: Self::Inner) -> bool {
        read_register(Register::SPURIOUS_VECTOR).get_bit(9)
    }

    fn get_spurious_eoi_broadcast_suppression(_: Self::Inner) -> bool {
        read_register(Register::SPURIOUS_VECTOR).get_bit(12)
    }

    fn set_spurious_vector(_: Self::Inner, vector: u8) {
        write_register(
            Register::SPURIOUS_VECTOR,
            *read_register(Register::SPURIOUS_VECTOR).set_bits(..8, u64::from(vector)),
        );
    }

    fn set_spurious_apic_software_enabled(_: Self::Inner, value: bool) {
        write_register(
            Register::SPURIOUS_VECTOR,
            *read_register(Register::SPURIOUS_VECTOR).set_bit(8, value),
        );
    }

    fn set_spurious_focus_processor_checking(_: Self::Inner, value: bool) {
        write_register(
            Register::SPURIOUS_VECTOR,
            *read_register(Register::SPURIOUS_VECTOR).set_bit(9, value),
        );
    }

    fn set_spurious_eoi_broadcast_suppression(_: Self::Inner, value: bool) {
        write_register(
            Register::SPURIOUS_VECTOR,
            *read_register(Register::SPURIOUS_VECTOR).set_bit(12, value),
        );
    }

    fn get_timer_vector(_: Self::Inner) -> LocalVector<Timer> {
        let raw = u32::try_from(read_register(Register::TIMER_VECTOR)).unwrap();
        LocalVector::<Timer>(raw, PhantomData)
    }

    fn set_timer_vector(_: Self::Inner, value: LocalVector<Timer>) {
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

    fn get_cmci_vector(_: Self::Inner) -> LocalVector<CMCI> {
        let raw = u32::try_from(read_register(Register::CMCI_VECTOR)).unwrap();
        LocalVector::<CMCI>(raw, PhantomData)
    }

    fn set_cmci_vector(_: Self::Inner, value: LocalVector<CMCI>) {
        write_register(Register::CMCI_VECTOR, u64::from(value));
    }

    fn get_lint0_vector(_: Self::Inner) -> LocalVector<LINT0> {
        let raw = u32::try_from(read_register(Register::LINT0_VECTOR)).unwrap();
        LocalVector::<LINT0>(raw, PhantomData)
    }

    fn set_lint0_vector(_: Self::Inner, value: LocalVector<LINT0>) {
        write_register(Register::LINT0_VECTOR, u64::from(value));
    }

    fn get_lint1_vector(_: Self::Inner) -> LocalVector<LINT1> {
        let raw = u32::try_from(read_register(Register::LINT1_VECTOR)).unwrap();
        LocalVector::<LINT1>(raw, PhantomData)
    }

    fn set_lint1_vector(_: Self::Inner, value: LocalVector<LINT1>) {
        write_register(Register::LINT1_VECTOR, u64::from(value));
    }

    fn get_error_vector(_: Self::Inner) -> LocalVector<Error> {
        let raw = u32::try_from(read_register(Register::ERROR_VECTOR)).unwrap();
        LocalVector::<Error>(raw, PhantomData)
    }

    fn set_error_vector(_: Self::Inner, value: LocalVector<Error>) {
        write_register(Register::ERROR_VECTOR, u64::from(value));
    }

    fn get_performance_monitors_vector(_: Self::Inner) -> LocalVector<PerformanceMonitors> {
        let raw = u32::try_from(read_register(Register::PERFORMANCE_MONITORS_VECTOR)).unwrap();
        LocalVector::<PerformanceMonitors>(raw, PhantomData)
    }

    fn set_performance_monitors_vector(_: Self::Inner, value: LocalVector<PerformanceMonitors>) {
        write_register(Register::PERFORMANCE_MONITORS_VECTOR, u64::from(value));
    }

    fn get_thermal_sensor_vector(_: Self::Inner) -> LocalVector<ThermalSensor> {
        let raw = u32::try_from(read_register(Register::THERMAL_SENSOR_VECTOR)).unwrap();
        LocalVector::<ThermalSensor>(raw, PhantomData)
    }

    fn set_thermal_sensor_vector(_: Self::Inner, value: LocalVector<ThermalSensor>) {
        write_register(Register::THERMAL_SENSOR_VECTOR, u64::from(value));
    }

    fn end_of_interrrupt(_: Self::Inner) {
        write_register(Register::END_OF_INTERRUPT, 0x0);
    }
}
