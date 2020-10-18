# dayz-monitor
DayZ monitoring discord bot, that doesn't require RCon! (Thanks to BattleMetrics)

## Features:
- Retrieves player count
- Retrieves the time on the server
- Lightweight
- Self hosted
- Will post time updates into a specified Discord channel (optional)

## Setup
### Step 1. Compilation

To do this you will need [Rust](https://www.rust-lang.org/tools/install) installed, so go do that, then come back!

Once Rust has been installed, something like this should do the trick!

```shell script
git clone https://github.com/Huskehhh/dayz-monitor
cd dayz-monitor
cargo build --release
```

Now dayz-monitor has been compiled. You're almost ready! All that is left is some configuration.

### Step 2. Configuration

First of all, all configuration options are handled via environment variables, 
which can either be set globally or simply through a .env file (I recommend the latter!)
So simply go ahead and create a ``.env`` file contaning these following values:

```.env
DISCORD_TOKEN=putyoursupersecretdiscordbottokenhere
TIME_CHANNEL_ID=putthediscordtimechannelidhere
BATTLEMETRICS_SERVER_ID=putyourbattlemetricsserveridhere
```

Individually looking at each of the variables:

``DISCORD_TOKEN`` (required)
This is the token you will get once you create a Bot on [Discord Development portal](https://discord.com/developers/applications)

Steps:
1. Create a new application at https://discordapp.com/developers/applications/
2. On the application's page, go to the "Bot" tab, click "Add Bot", and confirm!
3. Copy the "Token"

``TIME_CHANNEL_ID`` (optional)
This is the Discord ID of the channel you wish the bot to post updates in, if you want to disable this, remove this.

Steps:
1. Enable developer mode in Discord
2. Right click channel you want -> Copy ID

``BATTLEMETRICS_SERVER_ID`` (required)
Steps:
1. Go onto [BattleMetrics website](https://www.battlemetrics.com/servers) and find your server
2. Copy the ID out of the URL, for example ``https://www.battlemetrics.com/servers/dayz/5526398`` would be ``5526398``

### Step 3. Invite the bot

Now you've configured it, you need to invite the bot to your Discord server. To do this, you will need to run the bot.

Simply running ``cargo run --release`` will do the trick, I personally run mine in a ``screen`` so I can leave it unattended.

The bot will then provide you with an invite link to click, which will drag him into your Discord server!

### Step 4. Done!

Enjoy the bot, if you pick up on any bugs or would like more functionality, feel free to let me know via the issue
 tracker [here](https://github.com/Huskehhh/dayz-monitor/issues)
 
## Massive thanks to BattleMetrics, this project wouldn't be so easy without their API!