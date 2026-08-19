#![forbid(unsafe_code)]

pub(crate) mod entity;
pub(crate) mod error;
pub(crate) mod game;
pub(crate) mod network;
pub(crate) mod player;
pub(crate) mod utils;

pub mod protocol;
pub mod server;

use crate::error::Error;

pub type Result<T> = std::result::Result<T, Error>;
