extern crate dotenv;
#[macro_use]
extern crate lazy_static;

use std::error::Error;
use std::sync::Arc;
use std::thread::sleep;
use std::time::Duration;
use std::{env, thread};

use dashmap::DashMap;
use dotenv::dotenv;
use serde::Deserialize;
use serenity::async_trait;
use serenity::client::{Context, EventHandler};
use serenity::framework::standard::macros::*;
use serenity::framework::standard::{Args, CommandResult, DispatchError};
use serenity::framework::StandardFramework;
use serenity::futures::lock::Mutex;
use serenity::http::Http;
use serenity::model::channel::Message;
use serenity::model::gateway::Activity;
use serenity::model::id::ChannelId;
use serenity::{model::gateway::Ready, model::Permissions};
use serenity::{CacheAndHttp, Client};

lazy_static! {
    static ref CACHE: DashMap<bool, BattleMetricResponse> = DashMap::new();
}

#[group]
#[commands(time, count)]
struct General;

#[command]
#[aliases("t")]
async fn time(ctx: &Context, msg: &Message, mut _args: Args) -> CommandResult {
    if let Some(result) = CACHE.get(&true) {
        let formatted_result = format!(
            "Time on the DayZ Server is: ``{}``",
            &result.data.attributes.details.time
        );
        send_message(&ctx.http, &msg.channel_id, &formatted_result).await;
    }
    Ok(())
}

#[command]
#[aliases("c")]
async fn count(ctx: &Context, msg: &Message, mut _args: Args) -> CommandResult {
    if let Some(result) = CACHE.get(&true) {
        let formatted_result = format!(
            "There are ``{}`` players on the DayZ Server",
            &result.data.attributes.players
        );
        send_message(&ctx.http, &msg.channel_id, &formatted_result).await;
    }
    Ok(())
}

#[hook]
async fn normal_message(_ctx: &Context, msg: &Message) {
    println!("{}: {}", msg.author.name, msg.content);
}

#[hook]
async fn dispatch_error(ctx: &Context, msg: &Message, error: DispatchError) {
    if let DispatchError::Ratelimited(duration) = error {
        let _ = msg
            .channel_id
            .say(
                &ctx.http,
                &format!("Try this again in {} seconds.", duration.as_secs()),
            )
            .await;
    }

    println!("Error on dispatch: {:#?}", error);
}

#[derive(Debug, Deserialize, Clone)]
struct BattleMetricResponse {
    data: DataObject,
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

        ctx.set_activity(Activity::playing(
            "Monitoring DayZ server!\
         More info: https://github.com/Huskehhh/dayz-monitor/",
        ))
        .await;

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
    let server_id = env::var("BATTLEMETRICS_SERVER_ID")
        .expect("BATTLEMETRICS_SERVER_ID environment variable not found!");
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

async fn create_embedded_message(http: &Http, result: &BattleMetricResponse) {
    let channel_id_string =
        env::var("TIME_CHANNEL_ID").expect("No TIME_CHANNEL_ID environment variable set!");

    match channel_id_string.parse::<u64>() {
        Ok(channel_id) => {
            let channel = ChannelId(channel_id);

            let server_status = format!(
                "{} is {}",
                &result.data.attributes.name, &result.data.attributes.status
            );

            if let Err(e) = channel
                .send_message(&http, |m| {
                    m.content(&result.data.attributes.name);

                    m.embed(|e| {
                        e.title(server_status);
                        e.field("Time:", &result.data.attributes.details.time, false);
                        e.field("Player count:", &result.data.attributes.players, false);

                        e
                    })
                })
                .await
            {
                eprintln!("Error sending message to channel, {}", e);
            }
        }
        Err(why) => {
            eprintln!(
                "Error parsing the TIME_CHANNEL_ID environment variable value! Is it correct? {}",
                why
            );
        }
    }
}

#[tokio::main]
pub async fn application_task(mutex_http: Mutex<Arc<CacheAndHttp>>) {
    loop {
        let lock = mutex_http.lock().await;
        let http = &lock.http;

        if let Ok(result) = get_server_status().await {
            if let Some(cached_result) = CACHE.get(&true) {
                // If the cached result time is not eq to the current time, lets make a message!
                if !cached_result.data.attributes.details.time.eq(&result
                    .data
                    .attributes
                    .details
                    .time)
                {
                    // Create embedded message
                    create_embedded_message(&http, &result).await;

                    // Then overwrite cache with new data
                    CACHE.insert(true, result);
                }
            } else {
                CACHE.insert(true, result);
            }
        }

        sleep(Duration::from_secs(30));
    }
}

#[tokio::main]
async fn main() {
    dotenv().ok();
    let token =
        env::var("DISCORD_TOKEN").expect("Expected a token in your environment (DISCORD_TOKEN)");

    let framework = StandardFramework::new()
        .configure(|c| c.prefix("!").with_whitespace(true))
        .normal_message(normal_message)
        .on_dispatch_error(dispatch_error)
        .group(&GENERAL_GROUP);

    let mut client = Client::new(&token)
        .framework(framework)
        .event_handler(Handler)
        .await
        .expect("Error creating client");

    let mutex_http = Mutex::new(client.cache_and_http.clone());

    thread::spawn(move || {
        application_task(mutex_http);
    });

    if let Err(why) = client.start().await {
        println!("Client error: {:?}", why);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_api_call() {
        let result = get_server_status().await;
        assert!(result.is_ok());

        let unwrapped = result.unwrap();

        assert_ne!(unwrapped.data.attributes.name, "".to_owned())
    }

    #[tokio::test]
    async fn test_cache() {
        let result = get_server_status().await;
        assert!(result.is_ok());

        let cached_result = CACHE.get(&true);

        assert_eq!(
            result.unwrap().data.attributes.name,
            cached_result.unwrap().data.attributes.name
        );
    }
}
