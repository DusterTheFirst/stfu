use argh::FromArgs;
use serenity::prelude::TypeMapKey;

pub struct ArgsKey;
impl TypeMapKey for ArgsKey {
    type Value = Args;
}

#[derive(FromArgs, PartialEq, Debug)]
/// Program to make people shut the fuck up
pub struct Args {
    #[argh(subcommand)]
    pub command: Command,
    #[argh(option, default = "717435160378867772")]
    /// the channel id to mute
    pub channel: u64,
    #[argh(option)]
    /// the token to the bot
    pub token: String,
}

#[derive(FromArgs, PartialEq, Debug)]
#[argh(subcommand)]
pub enum Command {
    Mute(Mute),
    Unmute(Unmute),
}

#[derive(FromArgs, PartialEq, Debug)]
#[argh(subcommand, name = "mute")]
/// Mute all people in the channel
pub struct Mute {}

#[derive(FromArgs, PartialEq, Debug)]
#[argh(subcommand, name = "unmute")]
/// Unmute all people in the channel
pub struct Unmute {}
