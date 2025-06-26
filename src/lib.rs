#![no_std]
#![allow(non_camel_case_types, non_upper_case_globals)]

#[macro_use]
extern crate bitflags;

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
mod apic;
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
pub use apic::*;
