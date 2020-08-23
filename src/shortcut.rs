use args::Args;
use messages::{Acknowledgement, Action};
use notify_rust::Notification;
use panic::PanicInfo;
use std::{panic, process::Command, time::Duration};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
    time::delay_for,
};

mod args;
mod consts;
mod messages;

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

    let Args {
        channel,
        command,
        token,
    } = argh::from_env();

    let action = match command {
        args::Command::Mute(_) => Action::Mute(channel),
        args::Command::Unmute(_) => Action::Unmute(channel),
    };

    for i in 0..=1 {
        println!("Connecting to running daemon.");
        let connection = TcpStream::connect(consts::ADDRESS).await;

        match connection {
            Ok(mut stream) => {
                println!("Connected to running daemon.");

                let data =
                    bincode::serialize(&action).expect("Failed to serialize the action to send");

                stream
                    .write(&data)
                    .await
                    .expect("Failed to send the action the the daemon");

                let mut buf = vec![0; Acknowledgement::max_size()];
                stream
                    .read_exact(&mut buf)
                    .await
                    .expect("Failed to read in Ack");

                let ack: Acknowledgement =
                    bincode::deserialize(&buf).expect("Failed to deserialize Ack");

                match ack {
                    Acknowledgement::Success(raction) if raction == action => {
                        println!("Successfully dispatched and executed command, exiting");
                    }
                    _ => panic!("Failed to dispatch command"),
                }

                break;
            }
            Err(_) if i == 1 => {
                eprintln!("Failed to connect to daemon after starting it. Aborting.");

                Notification::new()
                    .summary("Daemon connection error")
                    .body("Failed to connect to daemon after starting it. Aborting.")
                    .show()
                    .unwrap();

                return;
            }
            Err(_) => {
                println!("Daemon is not running. Starting now.");

                if cfg!(debug_assertions) {
                    Command::new("cargo")
                        .args(&["run", "--bin", "stfu-daemon", "--quiet", &token])
                        .output()
                } else {
                    Command::new("stfu-daemon").arg(&token).output()
                }
                .expect("Failed to start daemon");

                println!("Daemon started, waiting 5 seconds and attempting connection");
                delay_for(Duration::from_secs(5)).await;
            }
        }
    }
}
