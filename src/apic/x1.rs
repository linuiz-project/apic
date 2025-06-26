use crate::{LocalVector, Mode, Timer, TimerMode};

struct x1;
impl Mode for x1 {
    type Inner = usize;

    fn get_id() -> u8 {
        todo!()
    }

    fn get_version() -> crate::Version {
        todo!()
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

    fn get_error_status() -> crate::ErrorStatus {
        todo!()
    }

    fn clear_error_status() {
        todo!()
    }

    fn get_timer_initial_count() -> u32 {
        todo!()
    }

    fn set_timer_initial_count(value: u32) {
        todo!()
    }

    fn get_timer_current_count() -> u32 {
        todo!()
    }

    fn get_timer_divide_configuration() -> crate::TimerDivideConfiguration {
        todo!()
    }

    fn set_timer_divide_configuration(value: crate::TimerDivideConfiguration) {
        todo!()
    }

    fn send_interrupt_command(interrupt_command: crate::InterruptCommand) {
        todo!()
    }

    fn get_spurious_vector() -> SpuriousInterruptVector {
        todo!()
    }

    fn set_spurious_vector(value: SpuriousInterruptVector) {
        todo!()
    }

    fn get_timer_vector() -> GenericVector {
        todo!()
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
    }

    fn get_cmci_vector() -> GenericVector {
        todo!()
    }

    fn set_cmci_vector(value: GenericVector) {
        todo!()
    }

    fn get_lint0_vector() -> LocalInterruptVector {
        todo!()
    }

    fn set_lint0_vector(value: LocalInterruptVector) {
        todo!()
    }

    fn get_lint1_vector() -> LocalInterruptVector {
        todo!()
    }

    fn set_lint1_vector(value: LocalInterruptVector) {
        todo!()
    }

    fn get_error_vector() -> ErrorVector {
        todo!()
    }

    fn set_error_vector(value: ErrorVector) {
        todo!()
    }

    fn get_performance_monitors_vector() -> GenericVector {
        todo!()
    }

    fn set_performance_monitors_vector(value: GenericVector) {
        todo!()
    }

    fn get_thermal_sensor_vector() -> GenericVector {
        todo!()
    }

    fn set_thermal_sensor_vector(value: GenericVector) {
        todo!()
    }

    fn end_of_interrrupt() {
        todo!()
    }
}
