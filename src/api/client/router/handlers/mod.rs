//! Client API router handlers module
use super::*;

// configuration scope handlers
pub mod config;

// running config scope handlers
pub mod run;

// gotham
use gotham::state::State;

const HELLO: &str = "Hello and welcome";

pub fn index(state: State) -> (State, &'static str) {
    (state, HELLO)
}
