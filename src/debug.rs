//! debugging module
//! This module provides debugging related functions.

// print_debug() function
/// This function simply print debugging information according to the specified level
pub fn print_debug(debug_level: u8, msg_level: u8, msg: String) {
    if debug_level >= msg_level {
        println!("{}", msg);
    }
}
