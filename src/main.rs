mod commands;
mod db;

use std::env;

use songbird::SerenityInit;

use serenity::async_trait;
use serenity::prelude::*;
use serenity::model::channel::Message;
use serenity::framework::standard::macros::{command, group};
use serenity::framework::standard::{StandardFramework, CommandResult};
use serenity::prelude::*;

use sqlx::{sqlite::SqlitePoolOptions, SqlitePool, query, query_as};


use crate::commands::tetr::*;
use crate::commands::voice::*;
use crate::commands::misc::*;

use crate::db::tetr_user::*;

#[group]
#[commands(join, leave, play, queue, current, skip, remove, pause, resume, cothenoi, help, pleasedont, replace, ping, repeat, test, tts, profile, profile40l, profileblitz, leaguerecent, save, lb)]
pub struct General;

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    // async fn ready(&self, _: Context, ready: Ready) {
    //     info!("Connected as {}", ready.user.name);
    // }
}

struct SQLClient;

impl TypeMapKey for SQLClient {
    type Value = SqlitePool;
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


    let db_path = "arab.sqlite";

    let db = SqlitePoolOptions::new()
        .max_connections(5)
        .connect(db_path)
        .await
        .expect("Failed to connect to database");

    sqlx::migrate!("./migrations")
        .run(&db)
        .await
        .expect("Failed to run migrations");

    {
        let mut data = client.data.write().await;
        data.insert::<SQLClient>(db.clone());
        data.insert::<TetrSavedUsers>(TetrSavedUsers::new(db.clone()));
    }

    // let u = query!(
    //     "SELECT
    //         *
    //     FROM tetr_users")
    //     .fetch_all(&db)
    //     .await;

    if let Err(why) = client.start().await {
        println!("An error occurred while running the client: {:?}", why);
    }
}
