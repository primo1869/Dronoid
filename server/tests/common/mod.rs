use anyhow::Ok;
use anyhow::Result;
use anyhow::bail;
use dronoid::protocol::ClientMessage;
use dronoid::protocol::Response;
use dronoid::protocol::ServerMessage;
use dronoid::protocol::State;
use dronoid::protocol::{Action, AuthenticationRequest, AuthenticationResponse};
use dronoid_server::new_commands;
use dronoid_server::persistence;
use dronoid_server::{Commands, Error, Rules};
use futures::SinkExt;
use futures::StreamExt;
use std::net::SocketAddr;
use tokio::{net::TcpStream, task::JoinHandle};
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream, tungstenite::Message};
use tracing::info;

pub struct TestContext {
    pub commands: Commands,
    pub server_addr: SocketAddr,
    #[allow(unused)]
    pub rules: Rules,
    pub hdl: JoinHandle<Result<()>>,
}

impl TestContext {
    pub async fn setup(rules: Rules) -> anyhow::Result<Self> {
        // init_logger();
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await?;
        let server_addr = listener.local_addr()?;
        let (commands, controls) = new_commands();
        let rules_cln = rules.clone();
        let hdl = tokio::spawn(async move {
            dronoid_server::run(
                rules_cln,
                persistence::Database::default(),
                listener,
                controls,
            )
            .await?;
            anyhow::Ok(())
        });
        Ok(Self {
            commands,
            server_addr,
            rules,
            hdl,
        })
    }

    pub async fn teardown(self) -> anyhow::Result<()> {
        self.commands.stop()?;
        self.hdl.await??;
        Ok(())
    }
}

#[allow(unused)]
pub fn default_rules() -> Rules {
    Rules::default()
}

pub struct Client {
    stream: WebSocketStream<MaybeTlsStream<TcpStream>>,
    pub spawn_point: (f32, f32),
}

impl Client {
    pub async fn connect(addr: SocketAddr) -> Result<Self> {
        info!(sender = "Client", "Connecting to {addr}");
        let (stream, _) = tokio_tungstenite::connect_async(
            format!("ws://{}:{}", addr.ip(), addr.port()).as_str(),
        )
        .await
        .map_err(|err| Error::ClientConnectError(err))?;
        Ok(Self {
            stream,
            spawn_point: (0., 0.),
        })
    }

    pub async fn authenticated(addr: SocketAddr, player_name: &'static str) -> Result<Self> {
        let mut client = Self::connect(addr).await?;
        let response = client.authenticate(player_name).await?;
        if !response.result {
            bail!(Error::ClientFailedAuthentication(response.text));
        }
        Ok(client)
    }

    pub async fn authenticate(
        &mut self,
        player_name: &'static str,
    ) -> Result<AuthenticationResponse> {
        info!(sender = "Client", "Authenticating");
        self.stream
            .send(Message::Binary(
                bson::serialize_to_vec(&ClientMessage::AuthenticationRequest(
                    AuthenticationRequest {
                        player_name: player_name.to_string(),
                    },
                ))
                .unwrap()
                .into(),
            ))
            .await
            .map_err(|err| Error::ClientSendError(err))?;
        if let Some(maybe_binary) = self.stream.next().await {
            if let std::result::Result::Ok(Message::Binary(auth_resp_text)) = maybe_binary {
                if let std::result::Result::Ok(response) =
                    bson::deserialize_from_slice::<ServerMessage>(auth_resp_text.iter().as_slice())
                {
                    match response {
                        ServerMessage::Response(Response::AuthenticationResponse(response)) => {
                            self.spawn_point = response.spawn_point;
                            return Ok(response);
                        }
                        _ => {
                            bail!("Test client: read: Not a response or not auth response");
                        }
                    }
                } else {
                    bail!("Test client: read: deserialization error");
                }
            } else {
                bail!(
                    "Test client: read: not okay: {}",
                    maybe_binary.err().unwrap()
                );
            }
        } else {
            bail!("Test client: read: none");
        }
    }

    #[allow(unused)]
    pub async fn send_action(&mut self, action: Action) -> Result<()> {
        info!(sender = "Client", "Sending action");
        self.stream
            .send(Message::Binary(
                bson::serialize_to_vec(&action).unwrap().into(),
            ))
            .await
            .map_err(|err| Error::ClientSendError(err))?;
        Ok(())
    }
    #[allow(unused)]
    pub async fn action(&mut self, action: Action) -> Result<Response> {
        self.send_action(action).await?;
        self.until_response().await
    }

    pub async fn next_message(&mut self) -> Result<ServerMessage> {
        let maybe_maybe_message = self.stream.next().await;
        if maybe_maybe_message.is_none() {
            bail!(Error::ClientReadError);
        }
        let message = maybe_maybe_message
            .unwrap()
            .map_err(|err| Error::RecvError(err))?;
        if let Message::Binary(text) = message {
            let server_message =
                bson::deserialize_from_slice::<ServerMessage>(text.iter().as_slice())
                    .map_err(|_| Error::ClientReadError)?;
            return Ok(server_message);
        } else {
            bail!(Error::ClientReadError);
        }
    }
    #[allow(unused)]
    pub async fn until_response(&mut self) -> Result<Response> {
        loop {
            match self.next_message().await? {
                ServerMessage::State(_) => continue,
                ServerMessage::Response(response) => {
                    return Ok(response);
                }
            }
        }
    }
    #[allow(unused)]
    pub async fn until_state(&mut self) -> Result<State> {
        loop {
            match self.next_message().await? {
                ServerMessage::State(state) => {
                    return Ok(state);
                }
                ServerMessage::Response(_) => continue,
            }
        }
    }
}
