//! `rwl` binary entrypoint.

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    rwl::run().await
}
