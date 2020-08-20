use anyhow::{Context as _, Result};
use args::{Args, ArgsKey};
use handler::Handler;
use serenity::client::Client;
use std::env;

mod args;
mod handler;

#[tokio::main]
async fn main() -> Result<()> {
    let args: Args = argh::from_env();

    let token = env::var("DISCORD_TOKEN")
        .context("You must provide a DISCORD_TOKEN environment variable")?;

    let mut client = Client::new(token)
        .event_handler(Handler)
        .await
        .context("Failed to create client")?;

    {
        let mut data = client.data.write().await;

        data.insert::<ArgsKey>(args);
    }

    client.start().await.context("Client failed to run")?;

    Ok(())
}
