# dayz-monitor

DayZ monitoring discord bot

## Features

- Retrieves player count (including those in queue).
- Retrieves the time on the server.
- Auto update a channel with number of players on server.

## Configuration options

Configuration can be done by controlling the below environment variables.

| Variable | Description | Required |
|----------|-------------|----------|
| `RUST_LOG` | Set this to the log level you want from the bot. | Optional |
| `DISCORD_TOKEN` | Discord bot token | Required |
| `SERVER_ADDRESS` | This is your query port, not the primary port. | Required |
| `SERVER_NAME` | Whatever you want, or empty as below | Required |
| `VOICE_CHANNEL_ID` | Discord ID of the channel you want updated with the player count. | Optional |

## Setup

### docker compose

1. Edit the `docker-compose.yml` to reflect your settings and then

2. Run the below

```bash
$ docker compose up
```

### Manual

1. Download (or compile) the binary for your platform.
2. Create a file called `.env` containing the above configuration options.
3. Run the binary.

### Required permissions

- Message Content Intent
- Send messages
- Read message history
- Manage channels (if using channel updating feature)

| Command | Alias | Description |
|---------|-------|-------------|
| !time | !t | Retrieves the current time of the DayZ server |
| !count | !c | Retrieves the current player count of the DayZ server |