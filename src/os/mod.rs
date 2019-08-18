//! operating systems support module

// drivers
pub mod drivers;

// linux operating system support
#[cfg(target_os = "linux")]
pub mod linux;
