//! operating systems support module

// drivers
pub mod drivers;

// Linux Operating System support
#[cfg(target_os = "linux")]
pub mod linux;

// FreeBSD Operating System support
#[cfg(target_os = "freebsd")]
pub mod freebsd;

// Multi-operating System Support
pub mod multi;