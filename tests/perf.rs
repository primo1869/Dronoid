use tokio::time::Instant;

use crate::common::{TestContext, default_rules};

mod common;

#[tokio::test(flavor = "multi_thread")]
async fn ping_01_twenty_messages() -> anyhow::Result<()> {
    let ctx = TestContext::setup(default_rules()).await?;
    let mut client = dronoid::Client::authenticated(ctx.server_addr, "Player1").await?;
    let mut now = Instant::now();
    let mut times = Vec::<f32>::new();

    for _ in 1..20 {
        client.next_message().await?;
        let new_now = Instant::now();
        times.push((new_now - now).as_secs_f32());
        now = new_now;
    }

    let average_ping: f32 = times.iter().sum::<f32>() / times.len() as f32;
    assert!(average_ping > ctx.rules.tick_duration - 0.05);
    assert!(average_ping < ctx.rules.tick_duration + 0.05);

    ctx.teardown().await
}
