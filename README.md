# dayz-monitor

DayZ monitoring discord bot

## Features

- Retrieves player count (including those in queue).
- Retrieves the time on the server.
- Auto update a channel with number of players on server.

## Configuration options

Configuration can be done by controlling the below environment variables.

```.env
# Discord bot token (Required)
DISCORD_TOKEN="yep"

# This is your query port, not the primary port. (Required)
SERVER_ADDRESS="1.2.3.4:2303"

# Whatever you want, or empty as below (Required)
SERVER_NAME=""

# Discord ID of the channel you want updated with the player count. (Optional)
VOICE_CHANNEL_ID=1234
```
## Setup

### docker-compose (preferred)

Step 1. Edit the `docker-compose.yml` to reflect your settings and then

```bash
$ docker compose up
```

### Manual

Step 1. Download (or compile) the binary for your platform.
Step 2. Create a file called `.env` containing the above configuration options.
Step 3. Run the binary.

## Commands

| Command | Alias | Description |
| !time | !t | Retrieves the current time of the DayZ server |
| !count | !c | Retrieves the current player count of the DayZ server |

