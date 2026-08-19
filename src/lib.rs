#![forbid(unsafe_code)]

pub(crate) mod error;
pub(crate) mod game;
pub(crate) mod network;
pub(crate) mod player;
pub(crate) mod protocol;
pub mod run;
pub(crate) mod utils;

use crate::error::Error;

pub type Result<T> = std::result::Result<T, Error>;
