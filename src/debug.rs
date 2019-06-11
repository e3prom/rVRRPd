//! debugging module
//! This module provides debugging related functions.
use chrono::{DateTime, Utc};

// print_debug() function
/// This function simply print debugging information according to the specified level
pub fn print_debug(debug_level: u8, msg_level: u8, msg: String) {
    let now: DateTime<Utc> = Utc::now();
    if debug_level >= msg_level {
        println!("[{}] {}", now.format("%e %b %Y %T"), msg);
    }
}
