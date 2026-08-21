#[cfg(test)]
mod tests {
    use anyhow::bail;
    use colored::*;
    use dronoid::protocol::{AuthenticationResponse, PlayerAction, ServerMessage};
    use futures::{SinkExt, StreamExt};
    use log::LevelFilter;
    use std::{net::SocketAddr, str::FromStr};
    use tokio::{
        io::AsyncWriteExt,
        net::{TcpSocket, TcpStream},
    };
    use tokio_tungstenite::{
        MaybeTlsStream, WebSocketStream,
        tungstenite::{self},
    };

    fn setup_logger() {
        let start_time = tokio::time::Instant::now();

        let log_level_str = std::env::var("RUST_LOG").unwrap_or_else(|_| "ERROR".to_string());
        let log_level = LevelFilter::from_str(&log_level_str).unwrap_or(LevelFilter::Error);

        let _ = fern::Dispatch::new()
            .level(log_level)
            .format(move |out, message, record| {
                let level_color = match record.level() {
                    log::Level::Error => colored::Color::Red,
                    log::Level::Warn => colored::Color::Yellow,
                    log::Level::Info => colored::Color::White,
                    log::Level::Debug => colored::Color::Green,
                    log::Level::Trace => colored::Color::BrightBlack,
                };

                let formatted = format!(
                    "[{:.3}] {}",
                    (tokio::time::Instant::now() - start_time).as_secs_f32(),
                    message
                );

                out.finish(format_args!("{}", formatted.color(level_color)));
            })
            .filter(|metadata| metadata.target().contains("dronoid"))
            .chain(std::io::stdout())
            .apply();
    }

    async fn bootstrap() -> anyhow::Result<(
        tokio::sync::oneshot::Sender<()>,
        tokio::task::JoinHandle<Result<(), anyhow::Error>>,
        SocketAddr,
    )> {
        let (tx, rx) = tokio::sync::oneshot::channel::<()>();
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await?;
        let addr = listener.local_addr().unwrap();
        let hdl = tokio::spawn(async {
            dronoid::server::run_custom(rx, listener).await?;
            anyhow::Ok(())
        });
        Ok((tx, hdl, addr))
    }

    async fn authenticated_client(
        addr: SocketAddr,
        player_name: String,
    ) -> anyhow::Result<(
        WebSocketStream<MaybeTlsStream<TcpStream>>,
        AuthenticationResponse,
    )> {
        let websocket_request = tungstenite::ClientRequestBuilder::new(
            tungstenite::http::Uri::from_str(
                format!("ws://{}:{}", addr.ip(), addr.port()).as_str(),
            )
            .unwrap(),
        );

        let (mut websocket, _response) =
            tokio_tungstenite::connect_async(websocket_request).await?;

        let auth_message = PlayerAction::Authentication { player_name };

        let message_str = serde_json::to_string(&auth_message).unwrap();

        websocket
            .send(tungstenite::Message::Text(message_str.into()))
            .await?;

        if let Some(Ok(tungstenite::Message::Text(auth_resp_text))) = websocket.next().await {
            if let Ok(ServerMessage::AuthenticationResponse(resp)) =
                serde_json::from_str(&auth_resp_text)
            {
                Ok((websocket, resp))
            } else {
                bail!("error")
            }
        } else {
            bail!("error")
        }
    }

    // fn before_all() {
    //     setup_logger();
    // }

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

    #[tokio::test]
    async fn test_04_auth_success() -> anyhow::Result<()> {
        let (tx, hdl, addr) = bootstrap().await?;

        let (_, resp) = authenticated_client(addr, String::from_str("Player").unwrap()).await?;

        assert_eq!(true, resp.result);
        assert_eq!("Welcome", resp.text);
        assert_eq!((0., 0.), resp.spawn_point);

        if tx.send(()).is_err() {
            bail!("Tests: stop send error");
        }

        hdl.await??;
        anyhow::Ok(())
    }

    #[tokio::test]
    async fn test_05_two_auths() -> anyhow::Result<()> {
        let (tx, hdl, addr) = bootstrap().await?;

        let (_, _) = authenticated_client(addr, String::from_str("Player1").unwrap()).await?;
        let (_, resp2) = authenticated_client(addr, String::from_str("Player2").unwrap()).await?;
        assert_eq!(true, resp2.result);
        assert_eq!("Welcome", resp2.text);
        assert_eq!((0., 0.), resp2.spawn_point);

        if tx.send(()).is_err() {
            bail!("Tests: stop send error");
        }

        hdl.await??;
        anyhow::Ok(())
    }

    #[tokio::test]
    async fn test_06_two_auths_same_name() -> anyhow::Result<()> {
        let (tx, hdl, addr) = bootstrap().await?;

        let (_, _) = authenticated_client(addr, String::from_str("Player1").unwrap()).await?;
        let (_, resp2) = authenticated_client(addr, String::from_str("Player1").unwrap()).await?;
        assert_eq!(false, resp2.result);
        assert_eq!("A player already has this name", resp2.text);
        assert_eq!((0., 0.), resp2.spawn_point);

        if tx.send(()).is_err() {
            bail!("Tests: stop send error");
        }

        hdl.await??;
        anyhow::Ok(())
    }

    #[tokio::test]
    async fn test_07_hundred_auths() -> anyhow::Result<()> {
        let (tx, hdl, addr) = bootstrap().await?;

        let mut hdls = Vec::new();
        for i in 0..100 {
            let hdl = tokio::spawn(async move {
                let (mut client, resp) = authenticated_client(addr, format!("Player{i}")).await?;
                client.close(None).await.unwrap();
                anyhow::Ok(resp)
            });
            hdls.push(hdl);
        }

        for hdl in hdls {
            let resp = hdl.await??;
            assert_eq!(true, resp.result);
            assert_eq!("Welcome", resp.text);
            assert_eq!((0., 0.), resp.spawn_point);
        }

        if tx.send(()).is_err() {
            bail!("Tests: stop send error");
        }

        hdl.await??;
        anyhow::Ok(())
    }
}
