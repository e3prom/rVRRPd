//! debugging module
//! This module provides debugging related functions.
use super::*;

// chrono
use chrono::{DateTime, Local, Utc};

// Verbose Structure
#[derive(Clone, Copy)]
pub struct Verbose {
    level: u8,
    time_zone: u8,
    time_format: u8,
}

// Debug type implementation
impl Verbose {
    // new() method
    pub fn new(level: u8, time_zone: u8, time_format: u8) -> Verbose {
        Verbose {
            level,
            time_zone,
            time_format,
        }
    }
}

// print_debug() function
/// This function simply print debugging information according to the specified level
pub fn print_debug(debug: &Verbose, msg_level: u8, msg: String) {
    // print debugging information with date and time
    if debug.level >= msg_level {
        match debug.time_zone {
            1 => {
                // UTC
                let now: DateTime<Utc> = Utc::now();
                match debug.time_format {
                    1 => println!("[{}] {}", now.to_rfc2822(), msg),
                    _ => println!("[{}] {}", now.format(RVRRPD_DFLT_DATE_FORMAT), msg),
                }
            } // local
            _ => {
                let now: DateTime<Local> = Local::now();
                match debug.time_format {
                    1 => println!("[{}] {}", now.to_rfc2822(), msg),
                    _ => println!("[{}] {}", now.format(RVRRPD_DFLT_DATE_FORMAT), msg),
                }
            }
        }
    }
}
