#![forbid(unsafe_code)]

pub(crate) mod entity;
pub(crate) mod error;
pub(crate) mod game;
pub(crate) mod network;
pub(crate) mod player;
pub(crate) mod utils;

pub mod protocol;
pub mod run;

use crate::error::Error;

pub type Result<T> = std::result::Result<T, Error>;

pub async fn run(port: u16) -> Result<()> {
    let (_sd, rx) = tokio::sync::oneshot::channel::<()>();

    let tcp_listener = tokio::net::TcpListener::bind(format!("127.0.0.1:{}", port))
        .await
        .map_err(|_err| Error::Error)?;

    run::run_custom(rx, tcp_listener).await?;
    Ok(())
}
