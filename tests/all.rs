#[cfg(test)]
mod tests {
    use anyhow::bail;
    use std::{net::SocketAddr, str::FromStr};
    use tokio::{io::AsyncWriteExt, net::TcpSocket};
    use tokio_tungstenite::tungstenite;

    async fn bootstrap() -> anyhow::Result<(
        tokio::sync::oneshot::Sender<()>,
        tokio::task::JoinHandle<Result<(), anyhow::Error>>,
        SocketAddr,
    )> {
        let (tx, rx) = tokio::sync::oneshot::channel::<()>();
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await?;
        let addr = listener.local_addr().unwrap();
        let hdl = tokio::spawn(async {
            dronoid::run_custom(rx, listener).await?;
            anyhow::Ok(())
        });
        Ok((tx, hdl, addr))
    }

    #[tokio::test]
    async fn test_01_run() -> anyhow::Result<()> {
        let (tx, hdl, addr) = bootstrap().await?;

        let socket = TcpSocket::new_v4()?;
        let _stream = socket.connect(addr).await?;
        if tx.send(()).is_err() {
            bail!("Tests: stop send error");
        }

        hdl.await??;

        anyhow::Ok(())
    }

    #[tokio::test]
    async fn test_02_connect_close() -> anyhow::Result<()> {
        let (tx, hdl, addr) = bootstrap().await?;
        let socket = TcpSocket::new_v4()?;
        let mut stream = socket.connect(addr).await?;

        if stream.shutdown().await.is_err() {
            bail!("shutdown error");
        }

        if tx.send(()).is_err() {
            bail!("Tests: stop send error");
        }

        hdl.await??;
        anyhow::Ok(())
    }

    #[tokio::test]
    async fn test_03_websocket_upgrade() -> anyhow::Result<()> {
        let (tx, hdl, addr) = bootstrap().await?;

        let websocket_request = tungstenite::ClientRequestBuilder::new(
            tungstenite::http::Uri::from_str(
                format!("ws://{}:{}", addr.ip(), addr.port()).as_str(),
            )
            .unwrap(),
        );

        tokio_tungstenite::connect_async(websocket_request).await?;

        if tx.send(()).is_err() {
            bail!("Tests: stop send error");
        }

        hdl.await??;
        anyhow::Ok(())
    }
}
