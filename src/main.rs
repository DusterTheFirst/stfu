use args::{Args, ArgsKey};
use handler::Handler;
use notify_rust::Notification;
use panic::PanicInfo;
use serenity::client::Client;
use std::panic;

mod args;
mod handler;

#[tokio::main]
async fn main() {
    panic::set_hook(Box::new(|info: &PanicInfo| {
        Notification::new()
            .summary("PANIC")
            .body(&format!("{}", info))
            .show()
            .unwrap();
        eprint!("{}", info);
    }));

    let args: Args = argh::from_env();

    let mut client = Client::new(&args.token)
        .event_handler(Handler)
        .await
        .expect("Failed to create client");

    {
        let mut data = client.data.write().await;

        data.insert::<ArgsKey>(args);
    }

    client.start().await.expect("Client failed to run");
}
