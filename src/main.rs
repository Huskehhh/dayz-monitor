extern crate dotenv;

use std::error::Error;
use std::sync::{Arc, Mutex};
use std::thread::sleep;
use std::time::Duration;
use std::{env, thread};

use async_rwlock::RwLock;
use dotenv::dotenv;
use once_cell::sync::Lazy;
use serde::Deserialize;
use serenity::client::{Context, EventHandler};
use serenity::framework::standard::macros::*;
use serenity::framework::standard::{Args, CommandResult, DispatchError};
use serenity::framework::StandardFramework;
use serenity::http::Http;
use serenity::model::channel::{ChannelType, Message};
use serenity::model::gateway::Activity;
use serenity::model::id::ChannelId;
use serenity::Client;
use serenity::{async_trait, CacheAndHttp};
use serenity::{model::gateway::Ready, model::Permissions};

static CACHED: Lazy<Arc<RwLock<BattleMetricResponse>>> = Lazy::new(|| {
    let battle_metrics_response = BattleMetricResponse::default();
    let mutex = RwLock::new(battle_metrics_response);
    let arc = Arc::new(mutex);
    arc
});

#[group]
#[commands(time, count, status, info, ip)]
struct General;

#[command]
#[aliases("t")]
async fn time(ctx: &Context, msg: &Message, mut _args: Args) -> CommandResult {
    let lock = &CACHED.read().await;

    if let Some(result) = &lock.data {
        let formatted_result = format!(
            "Time on the DayZ Server is: ``{}``",
            result.attributes.details.time
        );
        send_message(&ctx.http, &msg.channel_id, &formatted_result).await;
    }

    Ok(())
}

#[command]
#[aliases("c")]
async fn count(ctx: &Context, msg: &Message, mut _args: Args) -> CommandResult {
    let lock = &CACHED.read().await;

    if let Some(result) = &lock.data {
        let formatted_result = format!(
            "There are ``{}`` players on the DayZ Server",
            &result.attributes.players
        );
        send_message(&ctx.http, &msg.channel_id, &formatted_result).await;
    }

    Ok(())
}

#[command]
async fn status(ctx: &Context, msg: &Message, mut _args: Args) -> CommandResult {
    let lock = CACHED.read().await;

    if let Some(result) = &lock.data {
        let name = &result.attributes.name;
        let status = &result.attributes.status;
        let formatted_result = format!("{} is {}", &name, &status);
        send_message(&ctx.http, &msg.channel_id, &formatted_result).await;
    }

    Ok(())
}

#[command]
#[aliases("i")]
async fn info(ctx: &Context, msg: &Message, mut _args: Args) -> CommandResult {
    if let Some(cached_result) = &CACHED.read().await.data {
        create_embedded_message(&ctx.http, &cached_result, msg.channel_id).await;
    }
    Ok(())
}

#[command]
async fn ip(ctx: &Context, msg: &Message, mut _args: Args) -> CommandResult {
    if let Some(cached_result) = &CACHED.read().await.data {
        send_message(
            &ctx.http,
            &msg.channel_id,
            &format!(
                "IP: {}:{}",
                cached_result.attributes.ip, cached_result.attributes.port
            ),
        )
        .await;
    }
    Ok(())
}

#[hook]
async fn normal_message(_ctx: &Context, msg: &Message) {
    println!("{}: {}", msg.author.name, msg.content);
}

#[hook]
async fn dispatch_error(ctx: &Context, msg: &Message, error: DispatchError) {
    if let DispatchError::Ratelimited(info) = error {
        if info.is_first_try {
            let _ = msg
                .channel_id
                .say(
                    &ctx.http,
                    &format!("Try this again in {} seconds.", info.as_secs()),
                )
                .await;
        }
    }
}

#[derive(Debug, Deserialize, Clone)]
struct BattleMetricResponse {
    data: Option<DataObject>,
}

#[derive(Debug, Deserialize, Clone)]
struct BattleMetricSearchResponse {
    data: Option<Vec<SearchResponseData>>,
}

#[derive(Debug, Deserialize, Clone)]
struct SearchResponseData {
    id: String,
}

impl Default for BattleMetricSearchResponse {
    fn default() -> Self {
        BattleMetricSearchResponse { data: None }
    }
}

impl Default for BattleMetricResponse {
    fn default() -> Self {
        BattleMetricResponse { data: None }
    }
}

#[derive(Debug, Deserialize, Clone)]
struct DataObject {
    attributes: Attributes,
}

#[derive(Debug, Deserialize, Clone)]
struct Attributes {
    name: String,
    players: i32,
    status: String,
    details: Details,
    ip: String,
    port: i32,
}

#[derive(Debug, Deserialize, Clone)]
struct Details {
    time: String,
}
struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, ctx: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);

        if let Some(result) = &CACHED.read().await.data {
            ctx.set_activity(Activity::playing(&result.attributes.name))
                .await;
        }

        match ready.user.invite_url(&ctx.http, Permissions::empty()).await {
            Ok(url) => {
                println!("You can invite me using this url! {}", &url);
            }
            Err(why) => {
                eprintln!("Error getting invite url: {:?}", why);
            }
        };
    }
}

async fn get_server_status() -> Result<BattleMetricResponse, Box<dyn Error>> {
    let server_id = match get_battlemetrics_server_id().await {
        Some(id) => {
            format!("{}", id)
        }
        None => env::var("BATTLEMETRICS_SERVER_ID")
            .expect("BATTLEMETRICS_SERVER_ID environment variable not found!"),
    };

    let url = format!("https://api.battlemetrics.com/servers/{}", server_id);
    Ok(reqwest::get(&url)
        .await?
        .json::<BattleMetricResponse>()
        .await?)
}

async fn send_message(http: &Http, channel: &ChannelId, content: &str) {
    if let Err(why) = channel.say(&http, content).await {
        eprintln!("Error when sending message => {}", why);
    }
}

async fn create_embedded_message(http: &Http, result: &DataObject, channel_id: ChannelId) {
    let server_status = format!(
        "{} is {}",
        &result.attributes.name, &result.attributes.status
    );

    if let Err(e) = channel_id
        .send_message(&http, |m| {
            m.content(&result.attributes.name);

            m.embed(|e| {
                e.title(server_status);
                e.field("Time:", &result.attributes.details.time, false);
                e.field("Player count:", &result.attributes.players, false);

                e
            })
        })
        .await
    {
        eprintln!("Error sending message to channel, {}", e);
    }
}

async fn get_battlemetrics_search_response() -> Result<BattleMetricSearchResponse, Box<dyn Error>> {
    let search_query = env::var("BATTLEMETRICS_SEARCH")?;
    let url = format!("https://api.battlemetrics.com/servers?filter[game]=dayz&filter[search]={}&filter[status]=online&page[size]=1", search_query);

    Ok(reqwest::get(&url)
        .await?
        .json::<BattleMetricSearchResponse>()
        .await?)
}

async fn get_battlemetrics_server_id() -> Option<i32> {
    let battlemetrics_search_response = get_battlemetrics_search_response().await.ok()?;
    let data_vec = battlemetrics_search_response.data?;
    let id_str = &data_vec.first()?.id;
    let id = id_str.parse::<i32>().ok()?;

    Some(id)
}

#[tokio::main]
pub async fn update_cache(mutex_http: Mutex<Arc<CacheAndHttp>>) -> Result<(), Box<dyn Error>> {
    loop {
        let result = get_server_status().await?;
        let mut write_guard = CACHED.write().await;

        // Set to new value
        *write_guard = result.clone();

        std::mem::drop(write_guard);

        let player_count = result.data.unwrap().attributes.players;
        let guild_id_var = env::var("GUILD_ID");
        let server_name_var = env::var("SERVER_NAME");

        if let Ok(guild_id) = guild_id_var {
            if let Ok(server_name) = server_name_var {
                let lock = mutex_http.lock().unwrap();
                let http = &lock.http;

                let parsed_guild_id = guild_id.parse::<u64>()?;
                let guild = http.get_guild(parsed_guild_id).await?;

                let name = format!("{}: {}", server_name, player_count);

                let channels = guild.channels(&http).await?;

                let mut exists = false;

                for mut entry in channels {
                    if entry.1.name.starts_with(&server_name) {
                        exists = true;

                        println!("Updating channel to: '{}'", name);

                        entry
                            .1
                            .edit(&http, |c| {
                                c.name(&name);
                                c
                            })
                            .await?;
                    }
                }

                if !exists {
                    guild
                        .create_channel(http, |c| {
                            c.name(&name);
                            c.kind(ChannelType::Voice);
                            c
                        })
                        .await?;
                }
            }
        }

        sleep(Duration::from_secs(10));
    }
}

#[tokio::main]
async fn main() {
    dotenv().ok();
    let token =
        env::var("DISCORD_TOKEN").expect("Expected a token in your environment (DISCORD_TOKEN)");

    let framework = StandardFramework::new()
        .configure(|c| c.prefixes(vec!["!", "."]).with_whitespace(true))
        .normal_message(normal_message)
        .on_dispatch_error(dispatch_error)
        .group(&GENERAL_GROUP);

    let mut client = Client::builder(&token)
        .framework(framework)
        .event_handler(Handler)
        .await
        .expect("Error creating client");

    let mutex_http = Mutex::new(client.cache_and_http.clone());

    thread::spawn(|| {
        if let Err(why) = update_cache(mutex_http) {
            eprintln!("Error when updating cache: {}", why);
        }
    });

    if let Err(why) = client.start().await {
        println!("Client error: {:?}", why);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn setup_env() {
        // Set as a known server that wont (or shouldn't) change ID
        // https://www.battlemetrics.com/servers/dayz/5526399
        env::set_var("BATTLEMETRICS_SERVER_ID", "5526399");
    }

    fn cleanup_env() {
        env::remove_var("BATTLEMETRICS_SERVER_ID");
        env::remove_var("BATTLEMETRICS_SEARCH");
    }

    #[tokio::test]
    async fn test_api_call() {
        setup_env();

        let result = get_server_status().await;
        assert!(result.is_ok());

        let unwrapped = result.unwrap();

        assert_ne!(unwrapped.data.unwrap().attributes.name, "".to_owned());

        cleanup_env();
    }

    #[tokio::test]
    async fn test_cache() {
        setup_env();

        let result = get_server_status().await.unwrap();
        let mut write_guard = CACHED.write().await;
        *write_guard = result.clone();

        std::mem::drop(write_guard);

        let cached_result = &CACHED.read().await;
        let cached_name = cached_result.data.as_ref().unwrap().attributes.name.clone();

        let result_data = result.data.unwrap();

        assert_eq!(result_data.attributes.name, cached_name);
        assert_ne!(result_data.attributes.ip.is_empty(), true);

        cleanup_env();
    }

    #[tokio::test]
    async fn test_battlemetrics_search() {
        env::set_var("BATTLEMETRICS_SEARCH", "DayZ AP - SY 1023");

        let result = get_battlemetrics_server_id().await;
        assert!(result.is_some());
        assert_eq!(result.unwrap(), 5526399);

        cleanup_env();
    }

    #[tokio::test]
    async fn test_battlemetrics_search_fail() {
        let result = get_battlemetrics_server_id().await;
        assert!(result.is_none());

        cleanup_env();
    }
}
