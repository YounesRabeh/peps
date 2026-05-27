//! Binary entry point for the local Peps IDE server.

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    peps::ide::server::run().await
}
