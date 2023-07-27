use std::{
    sync::Arc,
};

use json::{
    JsonValue,
    object
};

use lazy_static::lazy_static;

use youtube_dl::YoutubeDl;
use youtube_dl::SearchOptions;
use youtube_dl::YoutubeDlOutput::SingleVideo;
use youtube_dl::YoutubeDlOutput::Playlist;

use serenity::{
    async_trait,
    client::{Client, Context, EventHandler},
    framework::{
        standard::{
            macros::{command, group},
            Args,
            CommandResult,
        },
        StandardFramework,
    },
    http::Http,
    model::{channel::Message, guild::Guild, gateway::Ready, prelude::ChannelId, id::GuildId},
    prelude::{GatewayIntents, Mentionable, Mutex},
    Result as SerenityResult,
};

use songbird::{
    input::{
        self,
        restartable::Restartable,
        Input,
    },
    tracks::{
        TrackHandle,
        TrackState,  
    },
    Event,
    EventContext,
    EventHandler as VoiceEventHandler,
    SerenityInit,
    TrackEvent,
    Call,
};

use urlencoding;

use async_process::Command;

fn print_type_of<T>(_: &T) {
    println!("{}", std::any::type_name::<T>())
}

lazy_static! {
    static ref guild_data : Mutex<JsonValue> = Mutex::new(object!{});
}

#[command]
pub async fn repeat(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let mode = args.rest();
    if !["on", "off"].contains(&mode) {
        msg.reply_ping(ctx, format!("Repeat {} la cai deo gi", mode)).await?;
        return Ok(());
    }

    let guild = msg.guild(&ctx.cache).unwrap();
    guild_data.lock().await[guild.id.as_u64().to_string()] = mode.into();

    msg.reply_ping(ctx, format!("{}", guild_data.lock().await.pretty(2))).await?;
    
    Ok(())
}

pub async fn select_song(ctx: &Context, msg: &Message, mut url: std::string::String) -> Option<Restartable> {
    if !url.starts_with("http") {
        let out = Command::new("yt-dlp").arg(format!("ytsearch5:{}", url)).args(&["--print", "title"]).args(&["--print", "id"]).args(&["--print", "duration_string"]).arg("--no-warnings").arg("--flat-playlist").output().await;
        let ytdl_output = match out {
            Ok(s) => s,
            Err(_e) => {
                msg.reply_ping(ctx, "loi roi??").await;
                return None;
            }
        };

        let searches = std::str::from_utf8(&ytdl_output.stdout).unwrap();
        let lines : Vec<&str> = searches.split("\n").collect();
        
        let mut choose_msg = String::new();

        let mut urls : Vec<&str> = Vec::new();

        let mut count = 0;
        for i in (0..lines.len() - 1).step_by(3) {
            let title = lines[i];
            let _url = format!("https://www.youtube.com/watch?v={}", lines[i + 1]);
            let duration = lines[i + 2];

            println!("{} {} {}", title, _url, duration);

            count = count + 1;
            urls.push(lines[i + 1]);

            let s = format!("**`[{}]` [{}]({})** - {}\n", count, title, _url, duration);
            
            choose_msg = choose_msg + &s;
        }

        msg.channel_id.send_message(&ctx.http, |m| {
            m.content("").reference_message(msg).embed(
                |e| e.title("chon bai bang cach go so").description(choose_msg)
            )
        }).await;

        let channel_id = msg.channel_id;
        let user_reply = channel_id
                            .await_reply(&ctx)
                            .timeout(std::time::Duration::new(15, 0))
                            .channel_id(channel_id)
                            .author_id(msg.author.id)
                            .await;
        
        let index_str = match user_reply {
            Some(song) => song,
            None => {
                msg.reply_ping(ctx, "timed out ?? go nhanh len").await;
                return None;
            }
        }; 

        let index = match index_str.content.parse::<i32>() {
            Ok(i) => i,
            Err(_e) => {
                index_str.reply_ping(ctx, "ok??").await;
                return None;
            }
        };

        if index <= 0 || index > (url.len() as i32) {
            index_str.reply_ping(ctx, "hoc dem pls").await;
            return None;
        }

        url = format!("https://www.youtube.com/watch?v={}", urls[index as usize - 1]).to_string();
    }
    
    let source = match if url.starts_with("http") { Restartable::ytdl(url, true).await } else { Restartable::ytdl_search(url, true).await } {
        Ok(source) => source,
        Err(why) => {
            println!("Error: {:?}", why);
            msg.reply_ping(ctx, "loi roi").await;
            return None;
        }
    };
    return Some(source);
}

pub async fn join_channel(ctx: &Context, msg: &Message) -> Option<Arc<Mutex<Call>>> {
    let guild = msg.guild(&ctx.cache).unwrap();
    let guild_id = guild.id;

    let manager = songbird::serenity::get(ctx).await.expect("Placed at init").clone();
    if !manager.get(guild_id).is_some() {
        let channel_id = guild.voice_states.get(&msg.author.id).and_then(|voice_state| voice_state.channel_id);

        let to_connect = match channel_id {
            Some(channel) => channel,
            None => {
                msg.reply_ping(ctx, "chua vao voice?").await;

                return None;
            }
        };

        let (handler_lock, success) = manager.join(guild_id, to_connect).await; // who ask

        if let Ok(_channel) = success {
            msg.reply_ping(ctx, "da vao voice").await;

            return Some(handler_lock);
        }
        else {
            return None;
        }
    }
    else {
        let handler_lock = match manager.get(guild_id) {
            Some(handler_lock) => handler_lock,
            None => {
                msg.reply_ping(ctx, "ko vao duoc voice sob").await;
                return None;
            }
        };
        return Some(handler_lock);
    }
}

#[command]
pub async fn join(ctx: &Context, msg: &Message) -> CommandResult {
    match join_channel(ctx, msg).await {
        Some(handler) => handler,
        None => {
            return Ok(());
        }
    };

    Ok(())
}

#[command]
pub async fn leave(ctx: &Context, msg: &Message) -> CommandResult {
    let guild = msg.guild(&ctx.cache).unwrap();
    let guild_id = guild.id;

    let manager = songbird::serenity::get(ctx).await.expect("Placed at init").clone();

    // check if the guild has a songbird handler (is Some) or not (is None)
    if manager.get(guild_id).is_some() {
        if let Err(e) = manager.remove(guild_id).await {
            msg.reply_ping(ctx, format!("fail: {:?}", e)).await?;
        }
        msg.reply_ping(ctx, "noooooooo").await?;
    } else {
        msg.reply(ctx, "leave gi?").await?;
    }

    Ok(())
}

#[command]
pub async fn play(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let url = args.rest().to_string();

    let handler_lock = match join_channel(ctx, msg).await {
        Some(handler) => handler,
        None => {
            return Ok(());
        }
    };

    let mut handler = handler_lock.lock().await;
    println!("{}", url);
    
    if url.starts_with("https://www.youtube.com/playlist?list=") {
        let output = YoutubeDl::new(url)
        .socket_timeout("15")
        .run()
        .unwrap();

        if let Playlist(pl) = output {
            let lst_o = pl.entries;
            if let Some(lst) = lst_o {
                for i in lst {
                    println!("{}", i.title);
                }
            }
        }
        return Ok(());
    }

    let source = match select_song(ctx, msg, url.clone()).await {
        Some(song) => song,
        None => {
            return Ok(());
        }
    };

    let song = handler.enqueue_source(source.into());

    let queue = handler.queue().current_queue();
    let cur = &queue[queue.len() - 1];
    let title = match &cur.metadata().title {
        Some(a) => &a,
        None => ""
    };
    let url = match &cur.metadata().source_url {
        Some(a) => &a,
        None => ""
    };
    let channel = match &cur.metadata().channel {
        Some(a) => &a,
        None => ""
    };
    let duration = match cur.metadata().duration {
        Some(a) => a.as_secs(),
        None => 0
    };
    let thumb = match &cur.metadata().thumbnail {
        Some(a) => &a,
        None => ""
    };

    msg.channel_id.send_message(&ctx.http, |m| {
        m.content("").reference_message(msg).embed(
            |e| e.title("da them bai gi do ko biet ten").description(format!(
                "[{}]({})\n\n**Channel**\n{}\n**Duration**\n{:0>2}:{:0>2}\n**Position in queue**\n{}",
                title, url, channel, duration / 60, duration % 60, queue.len()
            )).thumbnail(thumb)
        )
    }).await?;

    let _ = song.add_event(
        Event::Track(TrackEvent::End),
        SongEndNotifier {
            ctx: ctx.clone(),
            msg: msg.clone(),
            url: url.to_string(),
        },
    );

    Ok(())
}



#[async_trait]
impl VoiceEventHandler for SongEndNotifier {
    async fn act(&self, _ctx: &EventContext<'_>) -> Option<Event> {
        let guild = self.msg.guild(&self.ctx.cache).unwrap();
        let data = &guild_data.lock().await[guild.id.as_u64().to_string()];

        if data.is_null() || data == "off" {
            return None;
        }

        if let EventContext::Track(&[(state, track)])  = _ctx {
            if state.volume == 0.0 {
                return None;
            }
            
            let source = match select_song(&self.ctx, &self.msg, self.url.clone()).await {
                Some(song) => song,
                None => {
                    return None;
                }
            };    

            let manager = songbird::serenity::get(&self.ctx).await.expect("Placed at init").clone();
            let guild_id = self.msg.guild(&self.ctx.cache).unwrap().id;
            let handler_lock = match manager.get(guild_id) {
                Some(handler_lock) => handler_lock,
                None => {
                    return None;
                }
            };    
            
            let mut handler = handler_lock.lock().await;
            let song = handler.enqueue_source(source.into());

            let _ = song.add_event(
                Event::Track(TrackEvent::End),
                SongEndNotifier {
                    ctx: self.ctx.clone(),
                    msg: self.msg.clone(),
                    url: self.url.clone(),
                },
            ); 
        }

        return None;
    }
}

struct SongEndNotifier {
    ctx: Context,
    msg: Message,
    url: String,
}


#[command]
pub async fn queue(ctx: &Context, msg: &Message) -> CommandResult {
    let guild = msg.guild(&ctx.cache).unwrap();
    let guild_id = guild.id;
    let data = &guild_data.lock().await[guild.id.as_u64().to_string()];

    let manager = songbird::serenity::get(ctx).await.expect("Placed at init").clone();

    // check if in a vc or not
    if !manager.get(guild_id).is_some() {
        msg.reply_ping(ctx, "co bat nhac gi dau").await?;
        return Ok(());
    }

    let handler_lock = match manager.get(guild_id) {
        Some(handler_lock) => handler_lock,
        None => {
            msg.reply_ping(ctx, "ko vao duoc voice sob").await?;
            return Ok(());
        }
    };
    let mut handler = handler_lock.lock().await;
    let queue = handler.queue().current_queue();

    if queue.is_empty() {
        msg.reply_ping(ctx, "co bat nhac gi dau").await?;
        return Ok(());
    }

    let mut thumb = "";

    let mut queue_msg = String::new();
    let mut total_time = 0;
    for (i, cur) in queue.iter().enumerate() {
        let title = match &cur.metadata().title {
            Some(a) => &a,
            None => ""
        };
        let url = match &cur.metadata().source_url {
            Some(a) => &a,
            None => ""
        };
        let duration = match cur.metadata().duration {
            Some(a) => a.as_secs(),
            None => 0
        };
        total_time += duration;

        if i == 0 {
            thumb = match &cur.metadata().thumbnail {
                Some(a) => &a,
                None => ""
            };
            let position = cur.get_info().await.unwrap().position.as_secs();

            queue_msg += "Now playing:\n";
            queue_msg = queue_msg + format!("{}. [{}]({}) - `{:0>2}:{:0>2}/{:0>2}:{:0>2}`\n", i + 1, title, url, position / 60, position % 60, duration / 60, duration % 60).as_str();
            queue_msg += "Next in queue:\n";
        } else {
            queue_msg = queue_msg + format!("{}. [{}]({}) - `{:0>2}:{:0>2}`\n", i + 1, title, url, duration / 60, duration % 60).as_str();
        }
        queue_msg += "\n";
    }
    if queue.len() == 1 {
        queue_msg += "khong con gi ca ngu di\n";
    }
    queue_msg += format!("\n**{} bai trong queue voi tong cong thoi gian la {:0>2}:{:0>2}** | **repeat**: {}", queue.len(), total_time / 60, total_time % 60, if data.is_null() {"off"} else {data.as_str().unwrap()}).as_str();
    msg.channel_id.send_message(&ctx.http, |m| {
        m.content("").reference_message(msg).embed(
            |e| e.title("queue cua cai gi do").description(queue_msg).thumbnail(thumb)
        )
    }).await?;

    Ok(())
}

#[command]
pub async fn current(ctx: &Context, msg: &Message) -> CommandResult {
    let guild = msg.guild(&ctx.cache).unwrap();
    let guild_id = guild.id;

    let manager = songbird::serenity::get(ctx).await.expect("Placed at init").clone();

    // check if in a vc or not
    if !manager.get(guild_id).is_some() {
        msg.reply_ping(ctx, "co bat nhac gi dau").await?;
        return Ok(());
    }

    let handler_lock = match manager.get(guild_id) {
        Some(handler_lock) => handler_lock,
        None => {
            msg.reply_ping(ctx, "ko vao duoc voice sob").await?;
            return Ok(());
        }
    };
    let mut handler = handler_lock.lock().await;
    let cur = match handler.queue().current() {
        Some(cur) => cur,
        None => {
            msg.reply_ping(ctx, "co bat nhac gi dau").await?;
            return Ok(());
        }
    };

    let title = match &cur.metadata().title {
        Some(a) => &a,
        None => ""
    };
    let url = match &cur.metadata().source_url {
        Some(a) => &a,
        None => ""
    };
    let thumb = match &cur.metadata().thumbnail {
        Some(a) => &a,
        None => ""
    };

    let position = cur.get_info().await.unwrap().position.as_secs();
    let duration = match cur.metadata().duration {
        Some(a) => a.as_secs(),
        None => 0
    };

    msg.channel_id.send_message(&ctx.http, |m| {
        m.content("").reference_message(msg).embed(
            |e| e.title("bai gi do quen ten roi").description(format!(
                "[{}]({})\n`{:0>2}:{:0>2}/{:0>2}:{:0>2}`", title, url, position / 60, position % 60, duration / 60, duration % 60
            )).thumbnail(thumb)
        )
    }).await?;

    Ok(())
}

#[command]
pub async fn test(ctx: &Context, msg: &Message) -> CommandResult {
    msg.channel_id.send_message(&ctx.http, |m| m.content("test").reference_message(msg).embed(|e| e.title("embed test").description("description"))).await?;
    Ok(())
}

#[command]
pub async fn skip(ctx: &Context, msg: &Message) -> CommandResult {
    let guild = msg.guild(&ctx.cache).unwrap();
    let guild_id = guild.id;

    let manager = songbird::serenity::get(ctx).await.expect("Placed at init").clone();

    // check if in a vc or not
    if !manager.get(guild_id).is_some() {
        msg.reply_ping(ctx, "co bat nhac gi dau").await?;
        return Ok(());
    }

    let handler_lock = match manager.get(guild_id) {
        Some(handler_lock) => handler_lock,
        None => {
            msg.reply_ping(ctx, "ko vao duoc voice sob").await?;
            return Ok(());
        }
    };
    let mut handler = handler_lock.lock().await;
    let cur = match handler.queue().current() {
        Some(cur) => cur,
        None => {
            msg.reply_ping(ctx, "co bat nhac gi dau").await?;
            return Ok(());
        }
    };
    let _ = handler.queue().skip();

    msg.reply_ping(ctx, format!("da skip bai **{}**", match &cur.metadata().title {
        Some(title) => &title,
        None => ""
    })).await?;

    Ok(())
}

#[command]
pub async fn remove(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let pos = match args.single::<i32>() {
        Ok(pos) => pos,
        Err(_) => {
            msg.reply_ping(ctx, "so pls").await?;
            return Ok(());
        }
    };

    let guild = msg.guild(&ctx.cache).unwrap();
    let guild_id = guild.id;

    let manager = songbird::serenity::get(ctx).await.expect("Placed at init").clone();

    // check if in a vc or not
    if !manager.get(guild_id).is_some() {
        msg.reply_ping(ctx, "co bat nhac gi dau").await?;
        return Ok(());
    }

    let handler_lock = match manager.get(guild_id) {
        Some(handler_lock) => handler_lock,
        None => {
            msg.reply_ping(ctx, "ko vao duoc voice sob").await?;
            return Ok(());
        }
    };
    let mut handler = handler_lock.lock().await;
    if handler.queue().len() == 0 {
        msg.reply_ping(ctx, "co bat nhac gi dau").await?;
        return Ok(());
    }

    if pos == 1 {
        let cur = match handler.queue().current() {
            Some(cur) => cur,
            None => {
                msg.reply_ping(ctx, "co bat nhac gi dau").await?;
                return Ok(());
            }
        };
        cur.set_volume(0.0).ok();
        let _ = handler.queue().skip();
        msg.reply_ping(ctx, format!("da remove bai thu **{}** ten la **{}**", pos, match &cur.metadata().title {
            Some(title) => &title,
            None => ""
        })).await?;
    } else if let Some(song) = handler.queue().dequeue((pos - 1) as usize) {
        msg.reply_ping(ctx, format!("da remove bai thu **{}** ten la **{}**", pos, match &song.metadata().title {
            Some(title) => &title,
            None => ""
        })).await?;
    } else {
        msg.reply_ping(ctx, "bai ko ton tai?").await?;
    }

    Ok(())
}

#[command]
pub async fn pause(ctx: &Context, msg: &Message) -> CommandResult {
    let guild = msg.guild(&ctx.cache).unwrap();
    let guild_id = guild.id;

    let manager = songbird::serenity::get(ctx).await.expect("Placed at init").clone();

    // check if in a vc or not
    if !manager.get(guild_id).is_some() {
        msg.reply_ping(ctx, "co bat nhac gi dau").await?;
        return Ok(());
    }

    let handler_lock = match manager.get(guild_id) {
        Some(handler_lock) => handler_lock,
        None => {
            msg.reply_ping(ctx, "ko vao duoc voice sob").await?;
            return Ok(());
        }
    };
    let mut handler = handler_lock.lock().await;
    let cur = match handler.queue().current() {
        Some(cur) => cur,
        None => {
            msg.reply_ping(ctx, "co bat nhac gi dau").await?;
            return Ok(());
        }
    };
    let _ = handler.queue().pause();

    msg.reply_ping(ctx, format!("da pause bai **{}**", match &cur.metadata().title {
        Some(title) => &title,
        None => ""
    })).await?;

    Ok(())
}

#[command]
pub async fn resume(ctx: &Context, msg: &Message) -> CommandResult {
    let guild = msg.guild(&ctx.cache).unwrap();
    let guild_id = guild.id;

    let manager = songbird::serenity::get(ctx).await.expect("Placed at init").clone();

    // check if in a vc or not
    if !manager.get(guild_id).is_some() {
        msg.reply_ping(ctx, "co bat nhac gi dau").await?;
        return Ok(());
    }

    let handler_lock = match manager.get(guild_id) {
        Some(handler_lock) => handler_lock,
        None => {
            msg.reply_ping(ctx, "ko vao duoc voice sob").await?;
            return Ok(());
        }
    };
    let mut handler = handler_lock.lock().await;
    let cur = match handler.queue().current() {
        Some(cur) => cur,
        None => {
            msg.reply_ping(ctx, "co bat nhac gi dau").await?;
            return Ok(());
        }
    };
    let _ = handler.queue().skip();

    msg.reply_ping(ctx, format!("da resume bai **{}**", match &cur.metadata().title {
        Some(title) => &title,
        None => ""
    })).await?;

    Ok(())
}


#[command]
pub async fn tts(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let m = args.rest();
    let f = format!("https://translate.google.com/translate_tts?ie=UTF-8&q={}&tl=en&tk=418730.60457&client=webapp", m);
    let encoded_url = urlencoding::encode(&f);
    let mut url = encoded_url.into_owned();
    let download = Command::new("curl").arg(&url).arg("-o tts.mp3").output().await;

    match download {
        Ok(_s) => _s,
        Err(_e) => {
            msg.reply_ping(ctx, "LOI ROI BRO??").await;
            return Ok(());
        }
    };

    let guild = msg.guild(&ctx.cache).unwrap();
    let guild_id = guild.id;

    let handler_lock = match join_channel(ctx, msg).await {
        Some(handler) => handler,
        None => {
            return Ok(());
        }
    };

    let mut handler = handler_lock.lock().await;
    
    let source = match Restartable::ffmpeg("tts.mp3", true).await {
        Ok(song) => song,
        Err(e) => {
            return Ok(());
        }
    };

    handler.play_source(source.into());

    Ok(())
}
