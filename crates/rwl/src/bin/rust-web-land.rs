//! `rust-web-land` binary alias entrypoint.

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    rwl::run().await
}
