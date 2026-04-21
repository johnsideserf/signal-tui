//! Library entrypoint for fuzz testing and external consumers.
//!
//! The main binary ([`main`](../main/index.html)) has its own module tree;
//! this surface re-exports only what fuzz harnesses need (`config`, `input`,
//! `keybindings`, `signal`).

pub mod config;
#[allow(dead_code)]
mod debug_log;
pub mod input;
pub mod keybindings;
pub mod signal;
