#![feature(path_try_exists)]

pub mod bootstrap;

#[cfg(target_family = "unix")]
mod daemonize;

pub mod dirs;
pub mod logging;

#[cfg(feature = "profile")]
pub mod profile;

pub mod signal;
