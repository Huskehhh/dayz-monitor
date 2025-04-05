use std::{sync::Arc, time::Duration};

use dayz_monitor::{DayzMonitorConfig, ServerInfo, retrieve_server_info};
use serenity::all::GatewayIntents;
use tokio::sync::RwLock;
use tracing_subscriber::EnvFilter;

type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, Data, Error>;

struct Data {
    server_info: Arc<RwLock<ServerInfo>>,
}

fn pretty_print_players_in_queue(server_info: &ServerInfo) -> String {
    match &server_info.players_in_queue {
        Some(num_queued) => {
            format!(
                "{} (+ {}) / {} players online.",
                server_info.players, num_queued, server_info.max_players
            )
        }
        None => {
            format!(
                "{} / {} players online.",
                server_info.players, server_info.max_players
            )
        }
    }
}

#[poise::command(slash_command, prefix_command, aliases("t"))]
async fn time(ctx: Context<'_>) -> Result<(), Error> {
    let server_info = ctx.data().server_info.read().await;

    match &server_info.server_time {
        Some(time) => {
            ctx.say(format!("Time on the DayZ server is: {}", time))
                .await?;
        }
        None => {
            ctx.say("Unable to get the current time of the DayZ server.")
                .await?;
        }
    }

    Ok(())
}

#[poise::command(slash_command, prefix_command, aliases("c"))]
async fn count(ctx: Context<'_>) -> Result<(), Error> {
    let server_info = ctx.data().server_info.read().await;

    ctx.say(pretty_print_players_in_queue(&server_info)).await?;

    Ok(())
}

#[tokio::main]
async fn main() -> eyre::Result<()> {
    let _ = dotenv::dotenv();

    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    tracing::info!("Loading dayz-monitor configuration from environment variables");
    let config: DayzMonitorConfig = serde_env::from_env()?;
    tracing::debug!("Loaded config: {config:#?}");

    let client = a2s::A2SClient::new().await?;

    tracing::info!("Polling server for current information.");
    let server_info = Arc::new(RwLock::new(
        retrieve_server_info(&client, config.server_address).await?,
    ));

    let info_clone = server_info.clone();
    let http_client = serenity::http::Http::new(&config.discord_token);
    let voice_channel_id = config.voice_channel_id;
    tokio::spawn(async move {
        tokio::time::sleep(Duration::from_secs(60)).await;

        loop {
            tracing::trace!("Updating internal server info state");

            match retrieve_server_info(&client, config.server_address).await {
                Ok(server_info) => {
                    if let Some(voice_channel_id) = voice_channel_id {
                        let updated_name = format!(
                            "{}: {}",
                            config.server_name,
                            pretty_print_players_in_queue(&server_info)
                        );
                        tracing::trace!("Updating voice channel with name '{updated_name}'");

                        let builder = serenity::all::EditChannel::new().name(updated_name);
                        if let Err(why) = http_client
                            .edit_channel(voice_channel_id.into(), &builder, None)
                            .await
                        {
                            tracing::error!(
                                "Failed to update voice channel with id {voice_channel_id} because: {why:#?}"
                            )
                        }
                    }

                    *info_clone.write().await = server_info;
                }
                Err(why) => tracing::error!("Failed to update server info: {why}"),
            }

            tokio::time::sleep(Duration::from_secs(60)).await;
        }
    });

    let intents = GatewayIntents::non_privileged()
        | GatewayIntents::MESSAGE_CONTENT
        | GatewayIntents::DIRECT_MESSAGES
        | GatewayIntents::GUILDS;

    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: vec![time(), count()],
            prefix_options: poise::PrefixFrameworkOptions {
                prefix: Some("!".into()),
                edit_tracker: Some(Arc::new(poise::EditTracker::for_timespan(
                    Duration::from_secs(3600),
                ))),
                additional_prefixes: vec![poise::Prefix::Literal("~"), poise::Prefix::Literal(".")],
                ..Default::default()
            },
            ..Default::default()
        })
        .setup(|ctx, _ready, framework| {
            Box::pin(async move {
                poise::builtins::register_globally(ctx, &framework.options().commands).await?;
                Ok(Data { server_info })
            })
        })
        .build();

    let mut client = serenity::client::ClientBuilder::new(config.discord_token, intents)
        .framework(framework)
        .await?;

    client.start().await?;

    tracing::info!("dayz-monitor shutting down.");

    Ok(())
}
