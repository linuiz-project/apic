use bit_field::BitField;
use core::{arch::asm, fmt};
use local_vector::*;

pub mod local_vector;
pub mod x1;
pub mod x2;

mod interrupt_command;
pub use interrupt_command::*;

/// Gets the value of the `IA32_APIC_BASE` model-specific register.
fn get_ia32_apic_base() -> u64 {
    let value_low: u64;
    let value_high: u64;

    unsafe {
        asm!(
            "rdmsr",
            in("ecx") 0x1B,
            out("edx") value_high,
            out("eax") value_low,
            options(nostack, nomem, preserves_flags)
        );
    }

    (value_high << 32) | value_low
}

/// Sets the value of the `IA32_APIC_BASE` model-specific register.
unsafe fn set_ia32_apic_base(value: u64) {
    let value_low = value & 0xFFFF;
    let value_high = value >> 32;

    unsafe {
        asm!(
            "wrmsr",
            in("ecx") 0x1B,
            in("edx") value_high,
            in("eax") value_low,
            options(nostack, nomem, preserves_flags)
        );
    }
}

/// Specifies the version of an APIC device, the number of local vector
/// table entries, and whether software can suppress end-of-interrupt broadcasts.
pub struct Version(u32);

impl Version {
    /// Version of the APIC device.
    ///
    /// Possible values:
    /// - 0x0_: 82489DX discrete APIC
    /// - 0x10 to 0x15: Integrated APIC
    pub fn version(&self) -> u8 {
        u8::try_from(self.0.get_bits(..8)).unwrap()
    }

    /// Indicates whether software can inhibit the broadcast of an end of interrupt
    /// message by setting bit 12 of the spurious interrupt vector register.
    pub fn can_suppress_eoi_broadcast(&self) -> bool {
        self.0.get_bit(24)
    }

    /// The number of local vector table entries, less 1.
    ///
    /// Possible values:
    /// - For processors based on the Nehalem microarchitecture (which has 7 LVT entries) and onward: 6
    /// - For the Pentium 4 and Intel Xeon processors (which have 6 LVT entries): 5
    /// - For the P6 family processors (which have 5 LVT entries): 4
    /// - For the Pentium processor (which has 4 LVT entries): 3
    pub fn max_lvt_entry(&self) -> u8 {
        u8::try_from(self.0.get_bits(16..24)).unwrap()
    }
}

impl fmt::Debug for Version {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Version")
            .field("Version", &self.version())
            .field(
                "Can Suppress EOI Broadcast",
                &self.can_suppress_eoi_broadcast(),
            )
            .field("Maximum LVT Entry", &self.max_lvt_entry())
            .finish()
    }
}

bitflags! {
    #[repr(transparent)]
    #[derive(Debug, Clone, Copy)]
    pub struct ErrorStatus: u32 {
        const SEND_CHECKSUM_ERROR = 1 << 0;
        const RECEIVE_CHECKSUM_ERROR = 1 << 1;
        const SEND_ACCEPT_ERROR = 1 << 2;
        const RECEIVE_ACCEPT_ERROR = 1 << 3;
        const REDIRECTABLE_IPI = 1 << 4;
        const SENT_ILLEGAL_VECTOR = 1 << 5;
        const RECEIVED_ILLEGAL_VECTOR = 1 << 6;
        const ILLEGAL_REGISTER_ADDRESS = 1 << 7;
    }
}

bitflags! {
    #[repr(transparent)]
    #[derive(Debug, Clone, Copy)]
    pub struct TimerDivideConfiguration: u32 {
        const DIVIDE_1      = 0b1011;
        const DIVIDE_2      = 0b0000;
        const DIVIDE_4      = 0b0001;
        const DIVIDE_8      = 0b0010;
        const DIVIDE_16     = 0b0011;
        const DIVIDE_32     = 0b1000;
        const DIVIDE_64     = 0b1001;
        const DIVIDE_128    = 0b1010;
    }
}

pub const xAPIC_BASE_ADDR: usize = 0xFEE00000;
pub const x2APIC_BASE_MSR_ADDR: u32 = 0x800;

pub trait Mode {
    type Inner;

    fn get_id(inner: Self::Inner) -> u8;
    fn get_version(inner: Self::Inner) -> Version;

    fn get_task_priority(inner: Self::Inner) -> TaskPriority;
    fn set_task_priority(inner: Self::Inner, value: TaskPriority);

    fn get_arbitration_priority(inner: Self::Inner) -> ArbitrationPriority;
    fn get_processor_priority(inner: Self::Inner) -> ProcessorPriority;

    fn get_remote_read(inner: Self::Inner) -> RemoteRead;
    fn get_local_destination(inner: Self::Inner) -> LocalDestination;

    fn get_error_status(inner: Self::Inner) -> ErrorStatus;
    fn clear_error_status(inner: Self::Inner);

    fn get_timer_initial_count(inner: Self::Inner) -> u32;
    fn set_timer_initial_count(inner: Self::Inner, value: u32);

    fn get_timer_current_count(inner: Self::Inner) -> u32;

    fn get_timer_divide_configuration(inner: Self::Inner) -> TimerDivideConfiguration;
    fn set_timer_divide_configuration(inner: Self::Inner, value: TimerDivideConfiguration);

    fn send_interrupt_command(inner: Self::Inner, interrupt_command: InterruptCommand);

    fn get_spurious_vector(inner: Self::Inner) -> SpuriousInterruptVector;
    fn set_spurious_vector(inner: Self::Inner, value: SpuriousInterruptVector);

    fn get_timer_vector(inner: Self::Inner) -> LocalVector<Timer>;
    fn set_timer_vector(inner: Self::Inner, value: LocalVector<Timer>);

    fn get_cmci_vector(inner: Self::Inner) -> LocalVector<CMCI>;
    fn set_cmci_vector(inner: Self::Inner, value: LocalVector<CMCI>);

    fn get_lint0_vector(inner: Self::Inner) -> LocalVector<LINT0>;
    fn set_lint0_vector(inner: Self::Inner, value: LocalVector<LINT0>);

    fn get_lint1_vector(inner: Self::Inner) -> LocalVector<LINT1>;
    fn set_lint1_vector(inner: Self::Inner, value: LocalVector<LINT1>);

    fn get_error_vector(inner: Self::Inner) -> LocalVector<Error>;
    fn set_error_vector(inner: Self::Inner, value: LocalVector<Error>);

    fn get_performance_monitors_vector(inner: Self::Inner) -> LocalVector<PerformanceMonitors>;
    fn set_performance_monitors_vector(inner: Self::Inner, value: LocalVector<PerformanceMonitors>);

    fn get_thermal_sensor_vector(inner: Self::Inner) -> LocalVector<ThermalSensor>;
    fn set_thermal_sensor_vector(inner: Self::Inner, value: LocalVector<ThermalSensor>);

    fn end_of_interrrupt(inner: Self::Inner);
}

pub struct xApic<M: Mode>(M::Inner);

// impl Apic {
//     pub fn new(map_xapic_fn: Option<impl FnOnce(usize) -> *mut u8>) -> Option<Self> {
//         let ia32_apic_base = get_ia32_apic_base();
//         let is_hw_enabled = ia32_apic_base.get_bit(11);
//         let is_x2_mode = ia32_apic_base.get_bit(10);

//         let is_xapic = is_hw_enabled && !is_x2_mode;
//         let is_x2apic = is_hw_enabled && is_x2_mode;

//         if is_x2apic {
//             Some(Self(Type::x2APIC))
//         } else if is_xapic {
//             let map_xapic_fn = map_xapic_fn.expect("no mapping function provided for xAPIC");
//             Some(Self(Type::xAPIC(map_xapic_fn(
//                 IA32_APIC_BASE::get_base_address().try_into().unwrap(),
//             ))))
//         } else {
//             None
//         }
//     }

//     /// Reads the given register from the local APIC.
//     fn read_register(&self, register: Register) -> u32 {
//         match self.0 {
//             // Safety: Address provided for xAPIC mapping is required to be valid.
//             Type::xAPIC(xapic_ptr) => unsafe {
//                 xapic_ptr
//                     .add(register.xapic_offset())
//                     .cast::<u32>()
//                     .read_volatile()
//             },

//             // Safety: MSR addresses are known-valid from IA32 SDM.
//             Type::x2APIC => unsafe { msr::rdmsr(register.x2apic_msr()).try_into().unwrap() },
//         }
//     }

//     /// ## Safety
//     ///
//     /// Writing an invalid value to a register is undefined behaviour.
//     unsafe fn write_register(&self, register: Register, value: u32) {
//         match self.0 {
//             Type::xAPIC(xapic_ptr) => xapic_ptr
//                 .add(register.xapic_offset())
//                 .cast::<u32>()
//                 .write_volatile(value),
//             Type::x2APIC => msr::wrmsr(register.x2apic_msr(), value.into()),
//         }
//     }

//     /// ## Safety
//     ///
//     /// Given the amount of external contexts that could potentially rely on the APIC, enabling it
//     /// has the oppurtunity to affect those contexts in undefined ways.
//     #[inline]
//     pub unsafe fn sw_enable(&self) {
//         self.write_register(
//             Register::SPR,
//             *self.read_register(Register::SPR).set_bit(8, true),
//         );
//     }

//     /// ## Safety
//     ///
//     /// Given the amount of external contexts that could potentially rely on the APIC, disabling it
//     /// has the oppurtunity to affect those contexts in undefined ways.
//     #[inline]
//     pub unsafe fn sw_disable(&self) {
//         self.write_register(
//             Register::SPR,
//             *self.read_register(Register::SPR).set_bit(8, false),
//         );
//     }

//     pub fn get_id(&self) -> u32 {
//         self.read_register(Register::ID).get_bits(24..32)
//     }

//     #[inline]
//     pub fn get_version(&self) -> u32 {
//         self.read_register(Register::VERSION)
//     }

//     // TODO maybe unsafe?
//     #[inline]
//     pub fn end_of_interrupt(&self) {
//         unsafe { self.write_register(Register::EOI, 0x0) };
//     }

//     #[inline]
//     pub fn get_error_status(&self) -> ErrorStatus {
//         ErrorStatus::from_bits_truncate(self.read_register(Register::ERR))
//     }

//     /// ## Safety
//     ///
//     /// An invalid or unexpcted interrupt command could potentially put the core in an unusable state.
//     #[inline]
//     pub unsafe fn send_int_cmd(&self, interrupt_command: InterruptCommand) {
//         self.write_register(Register::ICRL, interrupt_command.destination_id());
//         self.write_register(Register::ICRH, interrupt_command.raw_command());
//     }

//     /// ## Safety
//     ///
//     /// The timer divisor directly affects the tick rate and interrupt rate of the
//     /// internal local timer clock. Thus, changing the divisor has the potential to
//     /// cause the same sorts of UB that [`set_timer_initial_count`] can cause.
//     #[inline]
//     pub unsafe fn set_timer_divisor(&self, divisor: TimerDivisor) {
//         self.write_register(Register::TIMER_DIVISOR, divisor.as_divide_value().into());
//     }

//     /// ## Safety
//     ///
//     /// Setting the initial count of the timer resets its internal clock. This can lead
//     /// to a situation where another context is awaiting a specific clock duration, but
//     /// is instead interrupted later than expected.
//     #[inline]
//     pub unsafe fn set_timer_initial_count(&self, count: u32) {
//         self.write_register(Register::TIMER_INT_CNT, count);
//     }

//     #[inline]
//     pub fn get_timer_current_count(&self) -> u32 {
//         self.read_register(Register::TIMER_CUR_CNT)
//     }

//     #[inline]
//     pub fn get_timer(&self) -> LocalVector<Timer> {
//         LocalVector(self, PhantomData)
//     }

//     #[inline]
//     pub fn get_lint0(&self) -> LocalVector<LINT0> {
//         LocalVector(self, PhantomData)
//     }

//     #[inline]
//     pub fn get_lint1(&self) -> LocalVector<LINT1> {
//         LocalVector(self, PhantomData)
//     }

//     #[inline]
//     pub fn get_performance(&self) -> LocalVector<Performance> {
//         LocalVector(self, PhantomData)
//     }

//     #[inline]
//     pub fn get_thermal_sensor(&self) -> LocalVector<Thermal> {
//         LocalVector(self, PhantomData)
//     }

//     #[inline]
//     pub fn get_error(&self) -> LocalVector<Error> {
//         LocalVector(self, PhantomData)
//     }

//     /// Resets the APIC module. The APIC module state is configured as follows:
//     ///     - Module is software disabled, then enabled at function end.
//     ///     - TPR and TIMER_INT_CNT are zeroed.
//     ///     - Timer, Performance, Thermal, and Error local vectors are masked.
//     ///     - LINT0 & LINT1 are unmasked and assigned to the `LINT0_VECTOR` (253) and `LINT1_VECTOR` (254), respectively.
//     ///     - The spurious register is configured with the `SPURIOUS_VECTOR` (255).
//     ///
//     /// ## Safety
//     ///
//     /// The caller must guarantee that software is in a state that is ready to accept the APIC performing a software reset.
//     pub unsafe fn software_reset(&self, spr_vector: u8, lint0_vector: u8, lint1_vector: u8) {
//         self.sw_disable();

//         self.write_register(Register::TPR, 0x0);
//         let modified_spr = *self
//             .read_register(Register::SPR)
//             .set_bits(0..8, spr_vector.into());
//         self.write_register(Register::SPR, modified_spr);

//         self.sw_enable();

//         // IA32 SDM specifies that after a software disable, all local vectors
//         // are masked, so we need to re-enable the LINTx vectors.
//         self.get_lint0().set_masked(false).set_vector(lint0_vector);
//         self.get_lint1().set_masked(false).set_vector(lint1_vector);
//     }
// }

// pub trait LocalVectorVariant {
//     const REGISTER: Register;
// }

// pub trait GenericVectorVariant: LocalVectorVariant {}

// pub struct Timer;
// impl LocalVectorVariant for Timer {
//     const REGISTER: Register = Register::LVT_TIMER;
// }

// pub struct LINT0;
// impl LocalVectorVariant for LINT0 {
//     const REGISTER: Register = Register::LVT_LINT0;
// }
// impl GenericVectorVariant for LINT0 {}

// pub struct LINT1;
// impl LocalVectorVariant for LINT1 {
//     const REGISTER: Register = Register::LVT_LINT1;
// }
// impl GenericVectorVariant for LINT1 {}

// pub struct Performance;
// impl LocalVectorVariant for Performance {
//     const REGISTER: Register = Register::LVT_PERF;
// }
// impl GenericVectorVariant for Performance {}

// pub struct Thermal;
// impl LocalVectorVariant for Thermal {
//     const REGISTER: Register = Register::LVT_THERMAL;
// }
// impl GenericVectorVariant for Thermal {}

// pub struct Error;
// impl LocalVectorVariant for Error {
//     const REGISTER: Register = Register::LVT_ERR;
// }

// #[repr(transparent)]
// pub struct LocalVector<'a, T: LocalVectorVariant>(&'a Apic, PhantomData<T>);

// impl<T: LocalVectorVariant> LocalVector<'_, T> {
//     const INTERRUPTED_OFFSET: usize = 12;
//     const MASKED_OFFSET: usize = 16;

//     #[inline]
//     pub fn get_interrupted(&self) -> bool {
//         self.0
//             .read_register(T::REGISTER)
//             .get_bit(Self::INTERRUPTED_OFFSET)
//     }

//     #[inline]
//     pub fn get_masked(&self) -> bool {
//         self.0
//             .read_register(T::REGISTER)
//             .get_bit(Self::MASKED_OFFSET)
//     }

//     /// ## Safety
//     ///
//     /// Masking an interrupt may result in contexts expecting that interrupt to fire to deadlock.
//     #[inline]
//     pub unsafe fn set_masked(&self, masked: bool) -> &Self {
//         self.0.write_register(
//             T::REGISTER,
//             *self
//                 .0
//                 .read_register(T::REGISTER)
//                 .set_bit(Self::MASKED_OFFSET, masked),
//         );

//         self
//     }

//     #[inline]
//     pub fn get_vector(&self) -> Option<u8> {
//         match self.0.read_register(T::REGISTER).get_bits(0..8) {
//             vector if (0..32).contains(&vector) => None,
//             vector => Some(vector as u8),
//         }
//     }

//     /// ## Safety
//     ///
//     /// Given the vector is an arbitrary >32 `u8`, all contexts must agree on what vectors
//     /// correspond to what local interrupts.
//     #[inline]
//     pub unsafe fn set_vector(&self, vector: u8) -> &Self {
//         assert!(vector >= 32, "interrupt vectors 0..32 are reserved");

//         self.0.write_register(
//             T::REGISTER,
//             *self
//                 .0
//                 .read_register(T::REGISTER)
//                 .set_bits(0..8, vector.into()),
//         );

//         self
//     }
// }

// impl<T: LocalVectorVariant> core::fmt::Debug for LocalVector<'_, T> {
//     fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
//         formatter
//             .debug_tuple("Local Vector")
//             .field(&self.0.read_register(T::REGISTER))
//             .finish()
//     }
// }

// impl<T: GenericVectorVariant> LocalVector<'_, T> {
//     /// ## Safety
//     ///
//     /// Setting the incorrect delivery mode may result in interrupts not being received
//     /// correctly, or being sent to all cores at once.
//     pub unsafe fn set_delivery_mode(&self, mode: InterruptDeliveryMode) -> &Self {
//         self.0.write_register(
//             T::REGISTER,
//             *self
//                 .0
//                 .read_register(T::REGISTER)
//                 .set_bits(8..11, mode as u32),
//         );

//         self
//     }
// }

// impl LocalVector<'_, Timer> {
//     #[inline]
//     pub fn get_mode(&self) -> TimerMode {
//         TimerMode::try_from(
//             self.0
//                 .read_register(<Timer as LocalVectorVariant>::REGISTER)
//                 .get_bits(17..19),
//         )
//         .unwrap()
//     }

//     /// ## Safety
//     ///
//     /// Setting the mode of the timer may result in undefined behaviour if switching modes while
//     /// the APIC is currently active and ticking (or otherwise expecting the timer to behave in
//     /// a particular, pre-defined fashion).
//     pub unsafe fn set_mode(&self, mode: TimerMode) -> &Self {
//         let tsc_dl_support = core::arch::x86_64::__cpuid(0x1).ecx.get_bit(24);

//         assert!(
//             mode != TimerMode::TscDeadline || tsc_dl_support,
//             "TSC deadline is not supported on this CPU."
//         );

//         self.0.write_register(
//             <Timer as LocalVectorVariant>::REGISTER,
//             *self
//                 .0
//                 .read_register(<Timer as LocalVectorVariant>::REGISTER)
//                 .set_bits(17..19, mode as u32),
//         );

//         if tsc_dl_support {
//             // IA32 SDM instructs utilizing the `mfence` instruction to ensure all writes to the IA32_TSC_DEADLINE
//             // MSR are serialized *after* the APIC timer mode switch (`wrmsr` to `IA32_TSC_DEADLINE` is non-serializing).
//             // Safety: `mfence` has no safety implications.
//             unsafe {
//                 core::arch::x86_64::_mm_mfence();
//             }
//         }

//         self
//     }
// }
