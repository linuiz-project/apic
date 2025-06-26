use crate::InterruptDeliveryMode;
use bit_field::BitField;
use core::marker::PhantomData;

pub trait Kind {}
pub trait Deliverable: Kind {}

pub struct Timer;
impl Kind for Timer {}

pub struct CMCI;
impl Kind for CMCI {}
impl Deliverable for CMCI {}

pub struct LINT0;
impl Kind for LINT0 {}

pub struct LINT1;
impl Kind for LINT1 {}

pub struct Error;
impl Kind for Error {}

pub struct PerformanceMonitors;
impl Kind for PerformanceMonitors {}
impl Deliverable for PerformanceMonitors {}

pub struct ThermalSensor;
impl Kind for ThermalSensor {}
impl Deliverable for ThermalSensor {}

#[derive(Debug, Clone)]
pub struct LocalVector<K: Kind>(u32, PhantomData<K>);

impl<K: Kind> LocalVector<K> {
    /// # Safety
    ///
    /// - `raw` must be a valid `u32` read directly from the respective
    ///   entry in the APIC's local vector table.
    pub(crate) unsafe fn new(raw: u32) -> Self {
        Self(raw, PhantomData)
    }

    /// Gets the delivery status of the interrupt.
    ///
    /// - `true` indicates that an interrupt from this source has been delivered to the
    ///   processor core but has not yet been accepted.
    /// - `false` indicates there is currently no activity for this interrupt source, or
    ///   the previous interrupt from this source was delivered to the processor core and
    ///   accepted.
    pub fn get_delivery_status(&self) -> bool {
        self.0.get_bit(12)
    }

    /// Whether the interrupt is masked (ignored upon reception to the APIC).
    ///
    /// Note: When the local APIC handles a performance-monitoring counters interrupt, it
    ///       automatically sets the mask flag in the LVT performance counter register. This
    ///       flag is set to 1 on reset. It can only be cleared by software.
    pub fn get_masked(&self) -> bool {
        self.0.get_bit(16)
    }

    /// Masks or unmasks the interrupt based on `masked`.
    ///
    /// Note: When the local APIC handles a performance-monitoring counters interrupt, it
    ///       automatically sets the mask flag in the LVT performance counter register. This
    ///       flag is set to 1 on reset. It can only be cleared by software.
    pub fn set_masked(&mut self, masked: bool) {
        self.0.set_bit(16, masked);
    }

    /// Gets the interrupt vector number.
    pub fn get_vector(&self) -> u8 {
        let vector = self.0.get_bits(0..8);

        debug_assert!(vector > 15, "interrupts vectors 0..=15 are reserved");

        u8::try_from(vector).unwrap()
    }

    /// Sets the interrupt vector number.
    pub fn set_vector(&mut self, vector: u8) {
        assert!(vector > 15, "interrupts vectors 0..=15 are reserved");

        self.0.set_bits(0..8, u32::from(vector));
    }
}

impl<K: Kind> From<LocalVector<K>> for u32 {
    fn from(value: LocalVector<K>) -> Self {
        value.0
    }
}

impl<K: Kind> From<LocalVector<K>> for u64 {
    fn from(value: LocalVector<K>) -> Self {
        u64::from(value.0)
    }
}

impl<K: Deliverable> LocalVector<K> {
    /// Specifies the type of interrupt to be sent to the processor. Some delivery modes will only
    /// operate as intended when used in conjunction with a specific trigger mode.
    pub fn set_delivery_mode(&mut self, mode: InterruptDeliveryMode) {
        self.0.set_bits(8..11, u32::from(mode));
    }
}

/// Specifies the polarity of an interrupt pin.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PinPolarity {
    ActiveHigh,
    ActiveLow,
}

/// Various valid modes for APIC timer to operate.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TimerMode {
    /// Timer will operate in a one-shot mode using a count-down value.
    OneShot,

    /// Timer will operate in a periodic mode by reloading a count-down value.
    Periodic,

    /// Uses the `IA32_TSC_DEADLINE` model-specific register as a deadline value, which will
    /// trigger when the hardware thread's timestamp counter reaches or passes the deadline.
    TscDeadline,
}

impl From<TimerMode> for u32 {
    fn from(value: TimerMode) -> Self {
        match value {
            TimerMode::OneShot => 0b00,
            TimerMode::Periodic => 0b01,
            TimerMode::TscDeadline => 0b10,
        }
    }
}

impl TryFrom<u32> for TimerMode {
    type Error = u32;

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        match value {
            0b00 => Ok(Self::OneShot),
            0b01 => Ok(Self::Periodic),
            0b10 => Ok(Self::TscDeadline),
            value => Err(value),
        }
    }
}

impl LocalVector<Timer> {
    /// Gets the mode that the timer is currently operating in.
    pub fn get_mode(&self) -> TimerMode {
        TimerMode::try_from(self.0.get_bits(17..19)).unwrap()
    }

    /// Sets the mode for the timer to operate in.
    pub fn set_mode(&mut self, mode: TimerMode) {
        // Safety: `cpuid` instruction is almost definitely supported.
        let is_tsc_deadline_supported = unsafe { core::arch::x86_64::__cpuid(0x1).ecx.get_bit(24) };

        assert!(
            mode != TimerMode::TscDeadline || is_tsc_deadline_supported,
            "TSC deadline mode is not supported by this APIC"
        );

        self.0.set_bits(17..19, u32::from(mode));
    }
}
