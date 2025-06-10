### Telegram bot

#### How to build

- Download and install [Rust](https://rustup.rs)
- Run `cargo build -r`
- Run `target/release/v3k-ctf-bot`

#### Required components

- [Valkey](https://valkey.io) or [Redis](https://redis.io) KV storage (at localhost at default port)
- [Vas3k API key](https://vas3k.club/apps/)
- [Telegram Bot API key](https://core.telegram.org/bots/api)

#### Configuration

Specify the following fields in [config.json](config.json)

```json
{
  "telegram_token": "XXXXXXX:YYYYYYYYYYYYYYYYYYYYYYYYYYYY",
  "vas3k_token": "*****************************",
  "event_start": 1749110400,
  "event_end": 1749315600,
  "test_group": [
    -1
  ],
  "admin_group": [
    -1
  ],
  "notify_group": [
    -1
  ]
}
```

- **event_start** - Unixtime
- **event_end** - Unixtime
- **test_group** - list of users (telegram IDs) who can access even outside of start/end window
- **admin_group** - list of users (telegram IDs) who can perform admin commands
- **notify_group** - list of chats (telegram IDs) to notify about solves and questions

#### User commands

- /**help**,/**start** - displays [help](src/text.rs)
- /**rules** - displays [rules](src/text.rs)
- /**score** - displays score and place
- /**tasks** - displays list of unsolved tasks
- /**code** - uploads bot source code
- /**contact** - allows to send a message to notify_group

#### Admin commands

- /**create** - creates tasks
- /**edit** - edits tasks
- /**delete** - deletes tasks
- /**board** - provides scoreboard
- /**message** - sends message to all users

#### Hidden tasks

Task with prefix name ['hidden:'](src/api.rs) is not displayed in the task list, but can be solved.