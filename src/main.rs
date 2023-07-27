mod commands;

use std::env;

use songbird::SerenityInit;

use serenity::async_trait;
use serenity::prelude::*;
use serenity::model::channel::Message;
use serenity::framework::standard::macros::{command, group};
use serenity::framework::standard::{StandardFramework, CommandResult};
use serenity::prelude::*;

use crate::commands::tetr::*;
use crate::commands::voice::*;
use crate::commands::misc::*;

#[group]
#[commands(join, leave, play, queue, current, skip, remove, pause, resume, cothenoi, help, pleasedont, replace, ping, repeat, test, tts, profile, profile40l, profileblitz, leaguerecent)]
pub struct General;

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    // async fn ready(&self, _: Context, ready: Ready) {
    //     info!("Connected as {}", ready.user.name);
    // }
}


#[tokio::main]
async fn main() {
    // let ytdl_args = [
    //     "ytsearch5:among us",
    //     "--get id",
    //     "--get-title",
    //     "--get-url"
    // ];

    // let mut youtube_dl = Command::new(YOUTUBE_DL_COMMAND)
    //     .args(&ytdl_args)
    //     .stdin(Stdio::null())
    //     .stderr(Stdio::piped())
    //     .stdout(Stdio::piped())
    //     .spawn().unwrap();    
    // let mut taken_stdout = youtube_dl.stdout.take().unwrap();

    // taken_stdout.read_to_string(&mut res);

    // print!("{}", res);

    env::set_var("RUST_BACKTRACE", "1");
    
    let framework = StandardFramework::new()
        .configure(|c| c.prefix("~"))
        .group(&GENERAL_GROUP);

    let token = "ODk3NjY1NTE2MTMzNTQ4MDQz.YWY-KA.KYoLYWH_VIUaQnYOPUrwjlpgZ5s";
    let intents = GatewayIntents::non_privileged() | GatewayIntents::MESSAGE_CONTENT;
    let mut client = Client::builder(token, intents)
        .event_handler(Handler)
        .framework(framework)
        .register_songbird()
        .await
        .expect("Error creating client");

    if let Err(why) = client.start().await {
        println!("An error occurred while running the client: {:?}", why);
    }
}
