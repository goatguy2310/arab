use serenity::framework::standard::macros::{command, group};
use serenity::framework::standard::{Args, CommandResult};
use serenity::model::prelude::*;
use serenity::prelude::*;
use serenity::builder::{CreateMessage, CreateEmbed};

use lazy_static::lazy_static;

use tetr_ch::client::{
    Client,
    stream::{StreamType, StreamContext}
};
use tetr_ch::model::{
    user::{
        UserRecordsResponse, 
        UserResponse,
        User}, 
    league::Rank,
    stream::StreamResponse};

use futures_util::{
    future::{join_all, ready, FutureExt},
    stream::{FuturesUnordered, StreamExt},
};

use sqlx::types::chrono::Utc;

use crate::db::tetr_user::{TetrSavedUsers, TetrUser};

static TETR_API_BASE: &str = "https://ch.tetr.io/api/";
static JSTRIS_API_BASE: &str = "https://jstris.jezevec10.com/api/";

pub fn get_flag_emoji(country: Option<String>) -> String {
    match country {
        Some(c) => format!(":flag_{}:", c.to_lowercase()),
        None => "".to_string()
    }
}

pub fn get_rank(r : &Rank) -> String {
    if r.as_str() == "z" {
        return "NA".to_string();
    }
    return r.as_str().to_string();
}

pub fn forty_lines_profile_fields(urec: &UserRecordsResponse) -> Vec<(String, String, bool)> {
    let mut result: Vec<(String, String, bool)> = Vec::new();
    if !urec.has_40l_record() {
        return result;
    }
    let rec = &urec.data.as_ref().unwrap().records.forty_lines;

    result.push((format!("40 LINES **<t:{}:R>**", rec.recorded_at()), String::new(), false));
    result.push(("Best Time".to_string(), format!("[{:.3}s]({})", 
        rec.record.as_ref().unwrap().endcontext.clone().single_play().as_ref().unwrap().final_time.unwrap() / 1000., 
        rec.record_url()),
        true));
    result.push(("Rank".to_string(),
        if let Some(rank) = rec.rank {format!("#{}", rank)} else {">#1000".to_string()},
        true));
    result.push(("PPS".to_string(), format!("{:.2}", rec.pps()), true));
    return result;
}

pub fn blitz_profile_fields(urec: &UserRecordsResponse) -> Vec<(String, String, bool)> {
    let mut result: Vec<(String, String, bool)> = Vec::new();
    if !urec.has_blitz_record() {
        return result;
    }
    let rec = &urec.data.as_ref().unwrap().records.blitz;
    let endctx = rec.record.as_ref().unwrap().endcontext.clone().single_play().unwrap();

    result.push((format!("BLITZ **<t:{}:R>**", rec.recorded_at()), String::new(), false));
    result.push(("Best Score".to_string(), format!("[{}]({})", 
        endctx.score.unwrap(), 
        rec.record_url()),
        true));
    result.push(("Rank".to_string(),
        if let Some(rank) = rec.rank {format!("#{}", rank)} else {">#1000".to_string()},
        true));
    result.push(("Level".to_string(), format!("{}", endctx.level.unwrap()), true));

    // result.push(("Quads".to_string(), format!("{}", endctx.clears.as_ref().unwrap().quads.unwrap()), true));
    // result.push(("T-Spins".to_string(), format!("{}", endctx.t_spins.unwrap()), true));
    // result.push(("Perfect Clears".to_string(), format!("{}", endctx.clears.as_ref().unwrap().singles.unwrap()), true));
    return result;
}

pub fn profile_embed(ur: UserResponse, urrec: UserRecordsResponse, username: String, m: &mut CreateEmbed) -> &mut CreateEmbed {
    let u = &ur.data.as_ref().unwrap().user;
    m.title(username)
        .url(ur.profile_url())
        .color(0x2bca79)
        .thumbnail(ur.face())
        .description(format!("Member since **<t:{}:R>**\nPlaytime: {}h", 
            ur.account_created_at().unwrap(), 
            if u.play_time > -1.0 {((u.play_time / 3600.0) as i32).to_string()} else {"Hidden".to_string()}))
        .field("\nTETRA LEAGUE", "", false)
        .field(
            "Rating",
            format!("**{:.2}** TR\n(**{}** / **{}**)", 
                u.league.rating, 
                get_rank(&u.league.rank), 
                get_rank(&u.league.best_rank.as_ref().unwrap_or_else(|| &Rank::Z)),
            ),
            false,
        )
        .field(
            "Ranks",
            format!("#**{}** (**{}**th) / **{}** #**{}**", 
                if u.league.standing != -1 {u.league.standing.to_string()} else {"NA".to_string()},
                ((1.0 - u.league.percentile) * 100.0) as i32,
                get_flag_emoji(u.country.clone()),
                if u.league.standing_local != -1 {u.league.standing_local.to_string()} else {"NA".to_string()}
            ),
            true,
        )
        .field(
            "Glicko",
            format!("{:.2} ±{:.2}",
                u.league.glicko.unwrap(),
                u.league.rd.unwrap()),
            true,
        )
        .field("Play / Win", format!("{} / {}", 
                u.league.play_count, 
                u.league.win_count), 
            true,
        )
        .field("Stats", format!("{:.2} APM | {:.2} PPS | {:.2} VS | {:.2} APP",
                u.league.apm.unwrap_or_else(|| 0.0),
                u.league.pps.unwrap_or_else(|| 0.0),
                u.league.vs.unwrap_or_else(|| 0.0),
                u.league.apm.unwrap_or_else(|| 0.0) / u.league.pps.unwrap_or_else(|| 1.0) / 60.0),
            false,
        )
        .fields(forty_lines_profile_fields(&urrec))
        .fields(blitz_profile_fields(&urrec))
}

pub fn profile_40l_embed(ur: UserResponse, urec: UserRecordsResponse, username: String, m: &mut CreateEmbed) -> &mut CreateEmbed {
    let u = &ur.data.as_ref().unwrap().user;
    let rec = &urec.data.as_ref().unwrap().records.forty_lines;
    let endctx = rec.record.as_ref().unwrap().endcontext.clone().single_play().unwrap();
    m.title(username)
        .url(ur.profile_url())
        .color(0x2bca79)
        .thumbnail(ur.face())
        .description(format!("Member since **<t:{}:R>**\nPlaytime: {}h", 
            ur.account_created_at().unwrap(), 
            if u.play_time > -1.0 {((u.play_time / 3600.0) as i32).to_string()} else {"Hidden".to_string()}))
        .field(format!("\n40 LINES **<t:{}:R>**", rec.recorded_at()), "", false)
        .field(
            "Best Time",
            format!("**{:.3}**s", 
                endctx.final_time.unwrap() / 1000.,
            ),
            false,
        )
        .field(
            "Rank",
            if let Some(rank) = rec.rank {format!("#{}", rank)} else {">#1000".to_string()},
            true,
        )
        .field("PPS", format!("{:.2}", rec.pps()), true)
        .field("Finesse", format!("{:.2}%", rec.finesse_rate()), true)

        .field("Pieces", format!("{:.2}", endctx.pieces_placed.unwrap()), true)
        .field("KPP", format!("{:.2}", rec.kpp()), true)
        .field("KPS", format!("{:.2}", rec.kps()), true)

        .field("Replay", format!("[[Link]]({})", rec.record_url()), true)
}

pub fn profile_blitz_embed(ur: UserResponse, urec: UserRecordsResponse, username: String, m: &mut CreateEmbed) -> &mut CreateEmbed {
    let u = &ur.data.as_ref().unwrap().user;
    let rec = &urec.data.as_ref().unwrap().records.blitz;
    let endctx = rec.record.as_ref().unwrap().endcontext.clone().single_play().unwrap();
    m.title(username)
        .url(ur.profile_url())
        .color(0x2bca79)
        .thumbnail(ur.face())
        .description(format!("Member since **<t:{}:R>**\nPlaytime: {}h", 
            ur.account_created_at().unwrap(), 
            if u.play_time > -1.0 {((u.play_time / 3600.0) as i32).to_string()} else {"Hidden".to_string()}))
        .field(format!("BLITZ **<t:{}:R>**", rec.recorded_at()), "", false)
        .field(
            "Best Score",
            format!("**{}**", 
                endctx.score.unwrap(),
            ),
            false,
        )
        .field(
            "Rank",
            if let Some(rank) = rec.rank {format!("#{}", rank)} else {">#1000".to_string()},
            true,
        )
        .field("PPS", format!("{:.2}", rec.pps()), true)
        .field("Level", format!("{}", endctx.level.unwrap()), true)

        .field("Finesse", format!("{:.2}%", rec.finesse_rate()), true)
        .field("Pieces", format!("{}", endctx.pieces_placed.unwrap()), true)
        .field("Lines", format!("{}", endctx.cleared_lines.unwrap()), true)

        .field("Quads", format!("{}", endctx.clears.as_ref().unwrap().quads.unwrap()), true)
        .field("T-Spins", format!("{}", endctx.t_spins.unwrap()), true)
        .field("Perfect Clears", format!("{}", endctx.clears.as_ref().unwrap().singles.unwrap()), true)

        .field("Replay", format!("[[Link]]({})", rec.record_url()), true)
}

pub fn leaguers_message(ur: UserResponse, sr: StreamResponse, username: String) -> String {
    let mut result = format!("Recent TL matches for `{}`:\n```", username);
    let vec = &sr.data.as_ref().unwrap().records;
    for (i, game) in vec.iter().enumerate() { 
        let mut endctx = game.endcontext.clone().multi_play().unwrap();
        let mut win: bool = true;
        if endctx[0].user.as_ref().unwrap().name != username {
            endctx.swap(0, 1);
            win = false;
        }
        let data = [endctx[0].points.as_ref().unwrap(), endctx[1].points.as_ref().unwrap()];
        result += format!("{} | {} ({}) vs ({}) {} ({})\n   {:.1}APM {:.1}PPS {:.1}VS {:.1}APP | {:.1}APM {:.1}PPS {:.1}VS {:.1}APP", 
            i + 1,

            endctx[0].user.as_ref().unwrap().name.to_uppercase(),
            data[0].primary.as_ref().unwrap(),
            data[1].primary.as_ref().unwrap(),
            endctx[1].user.as_ref().unwrap().name.to_uppercase(),

            if win {"VICTORY"} else {"DEFEAT"},

            data[0].secondary.as_ref().unwrap(),
            data[0].tertiary.as_ref().unwrap(),
            data[0].extra.vs.as_ref().unwrap(),
            data[0].secondary.as_ref().unwrap() / data[0].tertiary.as_ref().unwrap() / 60.0,

            data[1].secondary.as_ref().unwrap(),
            data[1].tertiary.as_ref().unwrap(),
            data[1].extra.vs.as_ref().unwrap(),
            data[1].secondary.as_ref().unwrap() / data[0].tertiary.as_ref().unwrap() / 60.0,
        ).as_str();
        result += "\n\n";
    }
    result += "```";
    return result;
}

#[command]
pub async fn profile(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let usn = args.rest();

    let ur = Client::new().get_user(usn).await.unwrap();
    let urrec = Client::new().get_user_records(usn).await.unwrap();

    msg.channel_id.send_message(&ctx,
        |m| {
            m.content("dog").
            embed(|m| profile_embed(ur.clone(), urrec.clone(), usn.to_string(), m))
        }).await?;
    Ok(())
}

#[command]
pub async fn profile40l(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let usn = args.rest();

    let ur = Client::new().get_user(usn).await.unwrap();
    let urrec = Client::new().get_user_records(usn).await.unwrap();

    if !urrec.has_40l_record() {
        msg.channel_id.say(&ctx, "This user doesn't have a 40l record").await?;
    } 
    else {
        msg.channel_id.send_message(&ctx,
            |m| {
                m.content("dog").
                embed(|m| profile_40l_embed(ur.clone(), urrec.clone(), usn.to_string(), m))
            }).await?;
    }
    Ok(())
}

#[command]
pub async fn profileblitz(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let usn = args.rest();

    let ur = Client::new().get_user(usn).await.unwrap();
    let urrec = Client::new().get_user_records(usn).await.unwrap();

    if !urrec.has_blitz_record() {
        msg.channel_id.say(&ctx, "This user doesn't have a blitz record").await?;
    } 
    else {
        msg.channel_id.send_message(&ctx,
            |m| {
                m.content("dog").
                embed(|m| profile_blitz_embed(ur.clone(), urrec.clone(), usn.to_string(), m))
            }).await?;
    }
    Ok(())
}

#[command]
pub async fn leaguerecent(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let usn = args.rest();

    let ur = Client::new().get_user(usn).await.unwrap();

    let id = &ur.data.as_ref().unwrap().user.id.id();
    let sr = Client::new().get_stream(StreamType::League, StreamContext::UserRecent, Some(id)).await.unwrap();
    msg.channel_id.send_message(&ctx,
        |m| {
            m.content(leaguers_message(ur.clone(), sr, usn.to_string()))
        }).await?;
    Ok(())
}

#[command]
pub async fn save(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let data = ctx.data.read().await;
    let tetr_users = data.get::<TetrSavedUsers>().unwrap();

    let targ = args.single::<serenity::model::id::UserId>().unwrap();

    let usn = args.quoted().trimmed().single::<String>().unwrap();
    let ur = Client::new().get_user(&usn).await.unwrap();
    let urrec = Client::new().get_user_records(&usn).await.unwrap();

    let u = &ur.data.as_ref().unwrap().user;
    let mut sprint = None;
    if urrec.has_40l_record() {
        sprint = Some(urrec.data.as_ref().unwrap().records.forty_lines.record.as_ref().unwrap().endcontext.clone().single_play().as_ref().unwrap().final_time.unwrap() / 1000.);
    }

    let mut blitz = None;
    if urrec.has_blitz_record() {
        let blitz_score = urrec.data.as_ref().unwrap().records.blitz.record.as_ref().unwrap().endcontext.clone().single_play().as_ref().unwrap().score;
        blitz = Some(blitz_score.unwrap() as f64);
    }
    let tu = TetrUser::new(
        targ.0 as i64,
        u.id.0.clone(),
        Utc::now(),
        u.league.rating,
        u.league.rank.as_str().to_string(),
        u.league.apm,
        u.league.pps,
        u.league.vs,
        sprint,
        blitz,
    );

    tetr_users.save(&tu).await?;
    msg.channel_id.say(&ctx, format!("User saved as {}", usn)).await?;
    Ok(())
}

#[command]
#[max_args(1)]
pub async fn lb(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let data = ctx.data.read().await;
    let tetr_users = data.get::<TetrSavedUsers>().unwrap();

    let mut mode = args.rest();
    if !["apm", "pps", "vs", "app", "40l", "sprint", "blitz"].contains(&mode) {
        mode = "tr";
    }
    let mut users = tetr_users
        .all()
        .await?
        .iter()
        .filter(|u| u.get_stat(mode).is_some())
        .map(|u| async move{
            msg
                .guild_id
                .expect("Guild-only command")
                .member(&ctx, UserId(u.user_id as u64))
                .await
                .ok()
                .and_then(|m|
                    Some((m.distinct(), u.get_stat(mode), u.rank.clone()))
                )
        })
        .collect::<FuturesUnordered<_>>()
        .filter_map(ready)
        .collect::<Vec<_>>()
        .await;

    users.sort_by(|a, b| (*b).1.partial_cmp(&a.1).unwrap());
    if mode == "40l" || mode == "sprint" { users.reverse(); }

    let mut res = String::from(format!("```No | Name | {} | TL Rank\n", mode.to_uppercase()));
    for (i, u) in users.iter().enumerate() {
        if mode != "blitz" { res += format!("{} | {} | {:.3} | {}\n", i + 1, u.0, u.1.unwrap(), u.2).as_str(); }
        else { res += format!("{} | {} | {} | {}\n", i + 1, u.0, u.1.unwrap(), u.2).as_str(); }
    }
    res += "```";
    msg.channel_id.say(&ctx, res).await?;

    Ok(())
}