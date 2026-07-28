use clap::Parser;
use std::{net::SocketAddr, str::FromStr};
use tokio::{net::TcpListener, signal};

#[derive(Parser, Debug)]
struct Args {
    #[arg(default_value_t = dronoid::defaults::PORT)]
    port: u16,
    #[arg(long, default_value_t = dronoid::defaults::TERRAIN_SCALE)]
    terrain_scale: f32,
    #[arg(long, default_value_t = dronoid::defaults::MINERAL_THRESHOLD)]
    mineral_threshold: f32,
    #[arg(long, default_value_t = dronoid::defaults::TICK_DURATION)]
    tick_duration: f32,
    #[arg(long, default_value_t = dronoid::defaults::STARTING_MINERALS)]
    starting_minerals: u32,
    #[arg(long, default_value_t = dronoid::defaults::TERRAIN_SEED)]
    terrain_seed: u32,
}

#[tokio::main(flavor = "multi_thread")]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    dronoid::init_logger();
    let tcp_listener = TcpListener::bind(SocketAddr::from_str(
        format!("127.0.0.1:{}", args.port).as_str(),
    )?)
    .await?;
    let rules = dronoid::Rules {
        terrain_scale: args.terrain_scale,
        mineral_threshold: args.mineral_threshold,
        tick_duration: args.tick_duration,
        starting_minerals: args.starting_minerals,
        terrain_seed: args.terrain_seed,
        ..Default::default()
    };
    let database = dronoid::persistence::Database::default();
    let (commands, controls) = dronoid::new_commands();
    tokio::spawn(async move {
        #[cfg(target_os = "windows")]
        let mut ctrl_close = signal::windows::ctrl_close().unwrap();
        #[cfg(target_os = "windows")]
        let mut ctrl_logoff = signal::windows::ctrl_logoff().unwrap();
        #[cfg(target_os = "windows")]
        let mut ctrl_shutdown = signal::windows::ctrl_shutdown().unwrap();
        #[cfg(target_os = "windows")]
        tokio::select! {
            _ = signal::ctrl_c() => {}
            _ = ctrl_close.recv() => {}
            _ = ctrl_logoff.recv() => {}
            _ = ctrl_shutdown.recv() => {}
        }
        #[cfg(target_os = "linux")]
        let _ = signal::ctrl_c().await;
        let _ = commands.stop();
    });

    dronoid::run(rules, database, tcp_listener, controls).await?;

    anyhow::Ok(())
}
