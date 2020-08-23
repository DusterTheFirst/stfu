use args::Args;
use messages::{Acknowledgement, Action};
use notify_rust::Notification;
use panic::PanicInfo;
use std::{
    panic,
    process::{Command, Stdio},
};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
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

    let connection = TcpStream::connect(consts::ADDRESS).await;
    let action = match command {
        args::Command::Mute(_) => Action::Mute(channel),
        args::Command::Unmute(_) => Action::Unmute(channel),
    };

    match connection {
        Ok(mut stream) => {
            println!("Connected to running daemon.");

            let data = bincode::serialize(&action).expect("Failed to serialize the action to send");

            println!("{:?}", data);

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
        }
        Err(_) => {
            println!("Daemon is not running. Starting now.");

            todo!()

            // let daemon_program = if cfg!(debug_assertions) {
            //     "./target/debug/stfu-daemon"
            // } else {
            //     "stfu-daemon"
            // };

            // let daemon = Command::new(daemon_program)
            //     .stdout(Stdio::null())
            //     .stderr(Stdio::null())
            //     .arg(token)
            //     .spawn()
            //     .expect("Failed to start daemon");
        }
    }
}
