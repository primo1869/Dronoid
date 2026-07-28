#[tokio::main(flavor = "multi_thread")]
async fn main() -> anyhow::Result<()> {
    let _ = dronoid::run(8080).await?;
    Ok(())
}
