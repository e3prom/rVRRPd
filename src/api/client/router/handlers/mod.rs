//! Client API router handlers module
use super::*;

// gotham
use gotham::state::State;

// configuration scope handlers
pub mod config;

// running config scope handlers
pub mod run;

const HELLO: &str = "Hello and welcome";

// index() function
pub fn index(state: State) -> (State, &'static str) {
    (state, HELLO)
}
