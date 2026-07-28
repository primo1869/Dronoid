use anyhow::Ok;
use std::net::SocketAddr;
use tokio::task::JoinHandle;

pub struct TestContext {
    pub commands: dronoid::Commands,
    pub server_addr: SocketAddr,
    #[allow(unused)]
    pub rules: dronoid::Rules,
    pub hdl: JoinHandle<dronoid::Result<()>>,
}

impl TestContext {
    pub async fn setup(rules: dronoid::Rules) -> anyhow::Result<Self> {
        dronoid::init_logger();
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await?;
        let server_addr = listener.local_addr()?;
        let (commands, controls) = dronoid::new_commands();
        let hdl = tokio::spawn(dronoid::run(
            rules.clone(),
            dronoid::persistence::Database::default(),
            listener,
            controls,
        ));
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
pub fn default_rules() -> dronoid::Rules {
    dronoid::Rules::default()
}
