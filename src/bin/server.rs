use colored::Colorize;
use futures::StreamExt;
use listen_signal::wait;
use log::LevelFilter;
use std::str::FromStr;

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

            let formatted = format!("[{:.3}] {}", (tokio::time::Instant::now() - start_time).as_secs_f32(), message);

            out.finish(format_args!("{}", formatted.color(level_color)));
        })
        .filter(|metadata| metadata.target().contains("dronoid"))
        .chain(std::io::stdout())
        .apply();
}

#[tokio::main(flavor = "multi_thread")]
async fn main() -> anyhow::Result<()> {
    setup_logger();

    let (stopper_tx, stopper_rx) = crossbeam_channel::bounded::<()>(1);

    tokio::spawn(async move {
        let mut signals = wait(&listen_signal::STOP);
        signals.next().await;
        stopper_tx.send(()).await;
    });

    dronoid::run(stopper_rx, 8080).await?;

    Ok(())
}
