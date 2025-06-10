/**
   Copyright 2025 Ivan Agarkov

   Licensed under the Apache License, Version 2.0 (the "License");
   you may not use this file except in compliance with the License.
   You may obtain a copy of the License at

       http://www.apache.org/licenses/LICENSE-2.0

   Unless required by applicable law or agreed to in writing, software
   distributed under the License is distributed on an "AS IS" BASIS,
   WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
   See the License for the specific language governing permissions and
   limitations under the License.
**/
mod api;
mod sender;
mod text;

use crate::api::{Api, SubmissionResult};
use crate::sender::MessageSender;
use crate::text::*;
use serde::Deserialize;
use std::env;
use std::sync::Arc;
use teloxide::dispatching::{Dispatcher, UpdateFilterExt};
use teloxide::requests::Requester;
use teloxide::types::{
    CallbackQuery, InlineKeyboardButton, InlineKeyboardMarkup, InputFile, MediaKind, Message,
    MessageKind, Update, User, UserId,
};
use teloxide::{Bot, dptree};
use tokio::runtime::Builder;

fn main() -> anyhow::Result<()> {
    env_logger::try_init()?;
    let rt = Builder::new_current_thread().enable_all().build()?;
    rt.block_on(rt_main())
}

#[derive(Debug, Deserialize)]
struct Config {
    telegram_token: String,
    vas3k_token: String,
    #[serde(default)]
    test_group: Vec<i64>,
    #[serde(default)]
    admin_group: Vec<i64>,
    #[serde(default)]
    notify_group: Vec<i64>,
    #[serde(default)]
    event_start: u64,
    #[serde(default)]
    event_end: u64,
}

async fn filter_users(_: Bot, api: Arc<Api>, msg: Message) -> bool {
    match msg.from.as_ref() {
        None => false,
        Some(user) => {
            if user.is_bot {
                false
            } else {
                api.check_user_is_in_scope(user.id.0).await
            }
        }
    }
}

/** We accept ONLY text messages **/
async fn filter_messages(_: Bot, _: Arc<Api>, msg: Message) -> bool {
    matches!(msg.kind, MessageKind::Common(x) if matches!(x.media_kind, MediaKind::Text(_)))
}

async fn answer_messages(bot: Bot, api: Arc<Api>, msg: Message) -> anyhow::Result<()> {
    let Some(text) = msg.text() else {
        return Ok(());
    };
    let user = msg.from.as_ref().unwrap();
    let state = api.get_user_state(user.id.0).await;
    let mut data = if state.is_some_and(|x| !x.is_empty()) {
        process_data(&bot, user, &api, text).await
    } else if text.starts_with("/") {
        process_command(&bot, user, &api, text).await
    } else {
        process_data(&bot, user, &api, text).await
    };
    data.reverse();
    // pack messages to make it more compact
    let mut message = String::new();

    while let Some(next) = data.pop() {
        let str: String = next.into();
        if str.len() + message.len() + 2 < 4096 {
            if !message.is_empty() {
                message.push_str("\n\n");
            }
            message.push_str(&str);
        } else {
            api.send_message(msg.chat.id.0, &message).await?;
            message.clear();
        }
    }
    if !message.is_empty() {
        api.send_message(msg.chat.id.0, &message).await?;
    }

    Ok(())
}

enum BotCommands {
    AdminCreate,
    AdminDelete,
    AdminScoreboard,
    AdminMessageAll,
    AdminEdit,
    UserScore,
    UserContact(Option<String>),
    UserHelp,
    UserRules,
    UserTasks,
    UserCode,
    UserSecretFlag,
    Unknown,
}

impl From<&str> for BotCommands {
    fn from(value: &str) -> Self {
        if value.starts_with("/contact") {
            let parts: Vec<&str> = value.split('_').collect();
            if parts.len() == 2 {
                Self::UserContact(Some(parts[1].trim().to_string()))
            } else {
                Self::UserContact(None)
            }
        } else {
            match value {
                "/start" => Self::UserHelp,
                "/create" => Self::AdminCreate,
                "/delete" => Self::AdminDelete,
                "/edit" => Self::AdminEdit,
                "/message" => Self::AdminMessageAll,
                "/board" => Self::AdminScoreboard,
                "/help" => Self::UserHelp,
                "/code" => Self::UserCode,
                "/tasks" => Self::UserTasks,
                "/rules" => Self::UserRules,
                "/score" => Self::UserScore,
                "/s3cr3t_comm4nd" => Self::UserSecretFlag,
                _ => Self::Unknown,
            }
        }
    }
}

enum ReplyText {
    Static(&'static str),
    String(String),
}

impl From<&'static str> for ReplyText {
    fn from(value: &'static str) -> Self {
        Self::Static(value)
    }
}

impl From<String> for ReplyText {
    fn from(value: String) -> Self {
        Self::String(value)
    }
}

impl From<ReplyText> for String {
    fn from(value: ReplyText) -> Self {
        match value {
            ReplyText::Static(s1) => s1.into(),
            ReplyText::String(s2) => s2,
        }
    }
}

async fn process_command(bot: &Bot, user: &User, api: &Arc<Api>, text: &str) -> Vec<ReplyText> {
    let mut ret: Vec<ReplyText> = Vec::new();
    let user_id = user.id.0;
    let command: BotCommands = text.into();
    let is_admin = api.is_admin(user_id);
    let can_process = api.can_process_command(user_id);
    match command {
        BotCommands::AdminCreate => {
            if is_admin {
                api.set_user_state(user_id, "create").await;
                ret.push(CREATE_TASK.into());
            } else {
                ret.push(DENIED.into());
            }
        }
        BotCommands::AdminDelete => {
            if is_admin {
                let mut keyboard: Vec<Vec<InlineKeyboardButton>> = Vec::new();
                let tasks = api.list_tasks(0).await;
                tasks
                    .into_iter()
                    .map(|task| InlineKeyboardButton::callback(task.name, task.id))
                    .for_each(|btn| keyboard.push(vec![btn]));
                let _ = api
                    .send_message_with_markup(
                        user_id as i64,
                        CHOOSE,
                        InlineKeyboardMarkup::new(keyboard).into(),
                    )
                    .await;
                api.set_user_state(user_id, "delete").await;
            } else {
                ret.push(DENIED.into());
            }
        }
        BotCommands::AdminEdit => {
            if is_admin {
                let mut keyboard: Vec<Vec<InlineKeyboardButton>> = Vec::new();
                let tasks = api.list_tasks(0).await;
                tasks
                    .into_iter()
                    .map(|task| InlineKeyboardButton::callback(task.name, task.id))
                    .for_each(|btn| keyboard.push(vec![btn]));
                let _ = api
                    .send_message_with_markup(
                        user_id as i64,
                        CHOOSE,
                        InlineKeyboardMarkup::new(keyboard).into(),
                    )
                    .await;
                api.set_user_state(user_id, "edit").await;
            } else {
                ret.push(DENIED.into());
            }
        }
        BotCommands::AdminScoreboard => {
            if is_admin {
                let board = api.get_scoreboard().await;
                let mut msg = String::new();
                for (i, (user, score)) in board.into_iter().enumerate() {
                    msg.push_str(&Format::format_score_board(i + 1, &user, score));
                }
                ret.push(ReplyText::String(msg));
            } else {
                ret.push(DENIED.into());
            }
        }
        BotCommands::AdminMessageAll => {
            if is_admin {
                api.set_user_state(user_id, "message").await;
                ret.push(MESSAGE_TEXT.into());
            } else {
                ret.push(DENIED.into());
            }
        }
        BotCommands::UserScore => {
            if !can_process {
                ret.push(NOT_YET.into());
            } else {
                let (place, score) = api.get_score(user_id).await;
                ret.push(Format::format_score(place, score).into());
            }
        }
        BotCommands::UserContact(task_id) => {
            let state = if let Some(task_id) = task_id {
                format!("contact_{}", task_id)
            } else {
                String::from("contact")
            };
            api.set_user_state(user_id, state).await;
            ret.push(CONTACT_TEXT.into());
        }
        BotCommands::UserHelp => {
            ret.push(HELP_TEXT.into());
        }
        BotCommands::UserRules => {
            ret.push(RULES_TEXT.into());
        }
        BotCommands::UserTasks => {
            if !can_process {
                ret.push(NOT_YET.into());
            } else {
                let tasks = api.list_tasks(user_id).await;
                if tasks.is_empty() {
                    ret.push(ALL_SOLVED.into());
                } else {
                    ret.append(
                        &mut tasks
                            .into_iter()
                            .map(|x| Format::format_task_user(&x).into())
                            .collect(),
                    );
                }
            }
        }
        BotCommands::UserCode => {
            if !can_process {
                ret.push(NOT_YET.into());
            } else {
                let file = InputFile::file("src/main.rs");
                if bot.send_document(UserId(user_id), file).await.is_err() {
                    ret.push(CODE_TEXT.into());
                }
            }
        }
        BotCommands::UserSecretFlag => {
            if !can_process {
                ret.push(NOT_YET.into());
            } else if let Ok(s) = env::var(VAR_NAME) {
                ret.push(s.into());
            } else {
                ret.push(UNKNOWN_TEXT.into());
            }
        }
        BotCommands::Unknown => {
            ret.push(UNKNOWN_TEXT.into());
        }
    }

    ret
}

async fn process_data(_bot: &Bot, user: &User, api: &Arc<Api>, text: &str) -> Vec<ReplyText> {
    let mut ret: Vec<ReplyText> = Vec::new();
    let user_id = user.id.0;
    let can_process = api.can_process_command(user_id);
    let state = api.get_user_state(user_id).await;

    match state {
        None => {
            if !can_process {
                ret.push(NOT_YET.into());
            } else {
                match api.try_submit_flag(user_id, text).await {
                    SubmissionResult::NotAFlag => {
                        ret.push(UNKNOWN_TEXT.into());
                    }
                    SubmissionResult::AlreadySolved => {
                        ret.push(ALREADY_SOLVED.into());
                    }
                    SubmissionResult::Solved(name) => {
                        let id = match user.username {
                            None => {
                                format!("{} ({})", user.first_name, user.id.0)
                            }
                            Some(ref username) => {
                                format!("{} (@{})", user.first_name, username)
                            }
                        };
                        let _ = api
                            .send_notification(Format::format_solved_admin(&id, &name))
                            .await;
                        ret.push(Format::format_solved(&name).into());
                    }
                }
            }
        }
        Some(state) => {
            if state.starts_with("contact") {
                if text.eq(".") {
                    let parts = state.split("_").collect::<Vec<&str>>();
                    let topic = if parts.len() == 2 {
                        let task = api.get_task(parts[1]).await;
                        task
                    } else {
                        None
                    };
                    let message = api.retrieve_and_erase_contact(user_id).await;
                    let user_id_str = user.id.0.to_string();
                    let message = Format::format_message(
                        user.username.as_deref().unwrap_or_else(|| &user_id_str),
                        &message,
                        topic.as_ref().map(|x| x.name.as_str()),
                    );

                    if let Err(e) = api.send_notification(message).await {
                        ret.push(Format::format_error(e).into());
                    } else {
                        ret.push(MESSAGE_SENT.into());
                    }
                    api.set_user_state(user_id, "").await;
                } else {
                    api.append_to_contact(user_id, text).await;
                }
            } else if state.eq("create") {
                match api.create_task(text).await {
                    Ok(id) => ret.push(Format::format_created(&id).into()),
                    Err(e) => ret.push(Format::format_error(e).into()),
                }
                api.set_user_state(user_id, "").await;
            } else if state.eq("message") {
                if text.eq(".") {
                    let message = api.retrieve_and_erase_contact(user_id).await;
                    let message = Format::format_message_broadcast(&message);
                    api.set_user_state(user_id, "").await;
                    for uid in api.get_all_users().await {
                        if uid != 0 {
                            if let Err(e) = api.send_message(uid as i64, &message).await {
                                ret.push(Format::format_error(e).into());
                            }
                        }
                    }
                } else {
                    api.append_to_contact(user_id, text).await;
                }
            } else if state.starts_with("edit_") {
                if let Some(id) = state.split("_").last() {
                    match api.edit_task(id, text).await {
                        Ok(_) => ret.push(Format::format_modified(id).into()),
                        Err(e) => ret.push(Format::format_error(e).into()),
                    }
                }
                api.set_user_state(user_id, "").await;
            } else {
                api.set_user_state(user_id, "").await; // reset state
                ret.push(NOT_IMPLEMENTED.into());
            }
        }
    }
    ret
}

async fn callback_handler(bot: Bot, api: Arc<Api>, query: CallbackQuery) -> anyhow::Result<()> {
    let user_id = query.from.id.0;
    let state = api.get_user_state(user_id).await;
    api.set_user_state(user_id, "").await;
    let message = match query.message {
        Some(m) => m,
        None => return Ok(()),
    };
    bot.delete_message(query.from.id, message.id()).await?;
    let id = match query.data {
        Some(s) => s,
        None => return Ok(()),
    };

    match state {
        None => return Ok(()),
        Some(ref state) => {
            let Some(task) = api.get_task(&id).await else {
                return Ok(());
            };
            match state.as_str() {
                "edit" => {
                    api.send_message(query.from.id.0 as i64, CREATE_TASK)
                        .await?;
                    api.send_message(query.from.id.0 as i64, Format::format_task_admin(&task))
                        .await?;
                    api.set_user_state(user_id, format!("edit_{id}")).await;
                }
                "delete" => {
                    api.delete_task(id).await?;
                    api.send_message(query.from.id.0 as i64, Format::format_deleted(&task.name))
                        .await?;
                }
                _ => (),
            };
        }
    }
    Ok(())
}

async fn rt_main() -> anyhow::Result<()> {
    let config: Arc<Config> = {
        let data = tokio::fs::read(CONFIG_NAME).await?;
        Arc::new(serde_json::from_slice(&data)?)
    };

    let bot = Bot::new(&config.telegram_token);
    let sender = MessageSender::new(bot.clone());
    let api = Api::new(config, sender.sender()).await;
    tokio::spawn(sender.start());
    let msg_handler = Update::filter_message()
        .filter_async(filter_users)
        .filter_async(filter_messages)
        .endpoint(answer_messages);
    let btn_handler = Update::filter_callback_query().endpoint(callback_handler);
    let handler = dptree::entry().branch(btn_handler).branch(msg_handler);
    Dispatcher::builder(bot, handler)
        .dependencies(dptree::deps![api])
        .enable_ctrlc_handler()
        .build()
        .dispatch()
        .await;
    Ok(())
}
