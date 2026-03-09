//! Core library for measuring time spent in a process

pub mod error;
pub mod history;
pub mod phase;
pub mod runner;
pub mod summary;
pub mod timestamp;

pub use error::{Error, Result};
