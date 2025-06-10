use crate::Config;
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
use crate::sender::Message;
use anyhow::bail;
use log::info;
use redis::AsyncCommands;
use redis::aio::MultiplexedConnection;
use reqwest::Client;
use reqwest::header::HeaderMap;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fmt::{Display, Formatter};
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use teloxide::types::ReplyMarkup;
use tokio::sync::Mutex;
use tokio::sync::mpsc::Sender;

#[derive(Serialize, Deserialize)]
pub struct Vas3kUser {
    #[serde(skip)]
    telegram_id: i64,
    id: String,
    slug: String,
    full_name: String,
    avatar: String,
    bio: String,
    upvotes: i64,
    created_at: String,
    membership_started_at: String,
    membership_expires_at: String,
    moderation_status: String,
    payment_status: String,
    company: Option<String>,
    position: Option<String>,
    city: Option<String>,
    country: Option<String>,
    is_active_member: bool,
}

trait FillId {
    fn fill_id<S: AsRef<str>>(&mut self, key: S);
}

impl FillId for String {
    fn fill_id<S: AsRef<str>>(&mut self, _key: S) {}
}

impl FillId for Vas3kUser {
    fn fill_id<S: AsRef<str>>(&mut self, key: S) {
        if let Some(rest) = key
            .as_ref()
            .split(':')
            .next_back()
            .and_then(|x| x.parse::<i64>().ok())
        {
            self.telegram_id = rest;
        }
    }
}

impl FillId for Task {
    fn fill_id<S: AsRef<str>>(&mut self, key: S) {
        if let Some(rest) = key.as_ref().split(':').next_back() {
            self.id = rest.to_owned();
        }
    }
}

impl FillId for Solve {
    fn fill_id<S: AsRef<str>>(&mut self, _: S) {}
}

impl Display for Vas3kUser {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} ({})", self.full_name, self.slug)
    }
}

#[derive(Serialize, Deserialize)]
struct Vas3kUserReply {
    user: Option<Vas3kUser>,
    error: Option<Vas3kError>,
}
#[derive(Serialize, Deserialize, Debug)]
struct Vas3kError {
    code: String,
    title: String,
    message: Option<String>,
}

impl Display for Vas3kError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self.message.as_ref() {
            None => write!(f, "vas3k api error: {} ({})", self.code, self.title),
            Some(message) => write!(
                f,
                "vas3k api error: {} ({}): {}",
                self.code, self.title, message
            ),
        }
    }
}

impl Error for Vas3kError {}

#[derive(Serialize, Deserialize, Debug)]
#[serde(untagged)]
pub enum FlagType {
    Single(String),
    Multi(Vec<String>),
}

impl Default for FlagType {
    fn default() -> Self {
        Self::Single(String::new())
    }
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct Task {
    pub name: String,
    pub flag: FlagType,
    pub hint: String,
    #[serde(skip)]
    pub id: String,
    #[serde(default)]
    pub hidden: bool,
}

pub enum SubmissionResult {
    NotAFlag,
    AlreadySolved,
    Solved(String),
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Solve {
    solves: Vec<String>,
}

impl From<Solve> for u64 {
    fn from(value: Solve) -> Self {
        value.solves.len() as u64
    }
}

pub struct Api {
    client: Client,
    conn: MultiplexedConnection,
    mutex: Mutex<bool>,
    sender: Sender<Message>,
    config: Arc<Config>,
}

impl Api {
    pub async fn send_notification<S: AsRef<str>>(&self, message: S) -> anyhow::Result<()> {
        for i in &self.config.notify_group {
            self.sender.send((*i, message.as_ref()).into()).await?;
        }
        Ok(())
    }
    pub async fn send_message<S: AsRef<str>>(&self, to: i64, message: S) -> anyhow::Result<()> {
        self.sender
            .send((to, message).into())
            .await
            .map_err(|e| e.into())
    }

    fn is_test_user(&self, user_id: u64) -> bool {
        self.config.test_group.iter().any(|x| *x == user_id as i64)
    }

    pub fn is_admin(&self, user_id: u64) -> bool {
        self.config.admin_group.iter().any(|x| *x == user_id as i64)
    }

    pub fn can_process_command(&self, user_id: u64) -> bool {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_or_else(|_| 0, |t| t.as_secs());
        if now > self.config.event_start && now < self.config.event_end {
            true
        } else {
            self.is_test_user(user_id) || self.is_admin(user_id)
        }
    }

    pub async fn send_message_with_markup<S: AsRef<str>>(
        &self,
        to: i64,
        message: S,
        kbd: ReplyMarkup,
    ) -> anyhow::Result<()> {
        self.sender
            .send((to, message, kbd).into())
            .await
            .map_err(|e| e.into())
    }
    pub async fn new(config: Arc<Config>, sender: Sender<Message>) -> Arc<Api> {
        let mut headers = HeaderMap::new();
        if let Ok(token) = config.vas3k_token.parse() {
            headers.insert("X-Service-Token", token);
        }
        if let Ok(cli) = redis::Client::open("redis://127.0.0.1/") {
            if let Ok(conn) = cli.get_multiplexed_async_connection().await {
                Arc::new(Self {
                    client: Client::builder()
                        .default_headers(headers)
                        .build()
                        .expect("Client::build"),
                    conn,
                    sender,
                    config,
                    mutex: Mutex::new(false),
                })
            } else {
                panic!("Failed to obtain async Redis connection");
            }
        } else {
            panic!("Failed to connect to Redis")
        }
    }

    async fn collect_from_cache<T>(&self, key: &str) -> Option<T>
    where
        T: DeserializeOwned + FillId,
    {
        let mut conn = self.conn.clone();
        if let Ok(value) = conn.get::<&str, Vec<u8>>(key).await {
            if let Ok(mut ttype) = serde_json::from_slice::<T>(&value[..]) {
                ttype.fill_id(key);
                return Some(ttype);
            }
        }
        None
    }

    async fn put_into_cache<T>(&self, key: &str, value: &T)
    where
        T: Serialize,
    {
        let mut conn = self.conn.clone();
        if let Ok(serialized) = serde_json::to_vec(value) {
            if let Err(e) = conn.set::<&str, &Vec<u8>, ()>(key, &serialized).await {
                info!("Failed to set Redis key {}: {e}", key);
            }
        }
    }

    async fn del_from_cache(&self, key: &str) {
        let mut conn = self.conn.clone();
        if let Err(e) = conn.del::<_, String>(key).await {
            info!("Failed to delete Redis key {}: {e}", key);
        }
    }

    pub async fn get_user_state(&self, user_id: u64) -> Option<String> {
        let key = format!("user_state:{}", user_id);
        self.collect_from_cache::<String>(&key)
            .await
            .filter(|x| !x.is_empty())
    }

    pub async fn set_user_state<S: AsRef<str>>(&self, user_id: u64, state: S) {
        let key = format!("user_state:{}", user_id);
        let value = String::from(state.as_ref());
        self.put_into_cache(&key, &value).await;
    }

    pub async fn receive_user_by_telegram(&self, user_id: u64) -> anyhow::Result<Vas3kUser> {
        let key = format!("user:{}", user_id);
        if let Some(user) = self.collect_from_cache(&key).await {
            return Ok(user);
        }
        let url = format!("https://vas3k.club/user/by_telegram_id/{}.json", user_id);
        let reply = self
            .client
            .get(url)
            .send()
            .await?
            .json::<Vas3kUserReply>()
            .await?;
        if let Some(error) = reply.error {
            Err(error.into())
        } else if let Some(user) = reply.user {
            self.put_into_cache(&key, &user).await;
            Ok(user)
        } else {
            unreachable!()
        }
    }

    pub async fn check_user_is_in_scope(&self, user_id: u64) -> bool {
        if let Ok(user) = self.receive_user_by_telegram(user_id).await {
            user.is_active_member
        } else {
            false
        }
    }

    async fn get_keys<S: AsRef<str>>(&self, pattern: S) -> Vec<String> {
        let mut conn = self.conn.clone();
        conn.keys::<&str, Vec<String>>(pattern.as_ref())
            .await
            .unwrap_or_default()
    }

    pub async fn try_submit_flag<S: AsRef<str>>(&self, user_id: u64, text: S) -> SubmissionResult {
        let try_flag = text.as_ref().trim().to_lowercase();
        for task_key in self.get_keys("task:*").await {
            if let Some(task) = self.collect_from_cache::<Task>(&task_key).await {
                if match task.flag {
                    FlagType::Single(s) => s.as_str().eq(try_flag.as_str()),
                    FlagType::Multi(vs) => vs.iter().any(|s| s.as_str().eq(try_flag.as_str())),
                } {
                    let mut val = self.mutex.lock().await;
                    *val = true;
                    let ret = if self.is_solved(user_id, &task_key).await {
                        SubmissionResult::AlreadySolved
                    } else {
                        self.set_solved(user_id, &task_key).await;
                        SubmissionResult::Solved(task.name)
                    };
                    *val = false;
                    drop(val);
                    return ret;
                }
            }
        }
        SubmissionResult::NotAFlag
    }

    async fn is_solved<S: AsRef<str>>(&self, user_id: u64, task_key: S) -> bool {
        let key = format!("solve:{}", user_id);
        if let Some(solve) = self.collect_from_cache::<Solve>(&key).await {
            if solve
                .solves
                .iter()
                .any(|s| s.as_str().eq(task_key.as_ref()))
            {
                return true;
            }
        }
        false
    }

    async fn set_solved<S: AsRef<str>>(&self, user_id: u64, task_key: S) {
        let key = format!("solve:{}", user_id);
        let solve = if let Some(mut solve) = self.collect_from_cache::<Solve>(&key).await {
            solve.solves.push(String::from(task_key.as_ref()));
            solve
        } else {
            Solve {
                solves: vec![String::from(task_key.as_ref())],
            }
        };
        self.put_into_cache(&key, &solve).await;
    }

    pub async fn get_score(&self, user_id: u64) -> (u64, u64) {
        let mut data = Vec::new();
        let user_key = format!("solve:{}", user_id);
        let hidden = self.is_test_user(user_id) || self.is_admin(user_id);
        for key in self.get_keys("solve:*").await {
            let score = self
                .collect_from_cache::<Solve>(&key)
                .await
                .map(|x| x.into())
                .unwrap_or(0u64);
            data.push((key, score));
        }
        let size = data.len() as u64;
        data.sort_by(|x, y| y.1.cmp(&x.1));
        let (place, score) = || -> (u64, u64) {
            for (i, (key, score)) in data.into_iter().enumerate() {
                if key.eq(&user_key) {
                    return ((i + 1) as u64, score);
                }
            }
            (size + 1, 0)
        }();
        if hidden {
            (u64::MAX, score)
        } else {
            (place, score)
        }
    }

    pub async fn create_task<S: AsRef<str>>(&self, text: S) -> anyhow::Result<String> {
        let task = Self::string_to_task(text)?;
        let mut key = format!(
            "task:{}",
            uuid::Uuid::new_v4().to_string().split('-').next().unwrap()
        );
        // lock
        let mut val = self.mutex.lock().await;
        *val = true;
        while self.collect_from_cache::<Task>(&key).await.is_some() {
            key = format!(
                "task:{}",
                uuid::Uuid::new_v4().to_string().split('-').next().unwrap()
            );
        }
        self.put_into_cache(&key, &task).await;
        *val = false;
        drop(val);
        Ok(key)
    }

    pub async fn list_tasks(&self, user_id: u64) -> Vec<Task> {
        let mut tasks = Vec::new();
        for task_key in self.get_keys("task:*").await {
            if user_id == 0 || !self.is_solved(user_id, &task_key).await {
                if let Some(task) = self.collect_from_cache::<Task>(&task_key).await {
                    if !task.hidden {
                        tasks.push(task);
                    }
                }
            }
        }
        tasks.sort_by(|x, y| x.name.cmp(&y.name));
        tasks
    }

    pub async fn get_task<S: AsRef<str>>(&self, name: S) -> Option<Task> {
        let key = format!("task:{}", name.as_ref());
        self.collect_from_cache::<Task>(&key).await
    }

    pub async fn append_to_contact<S: AsRef<str>>(&self, user_id: u64, text: S) {
        let key = format!("contact:{}", user_id);
        let message = if let Some(mut old) = self.collect_from_cache::<String>(&key).await {
            old.push('\n');
            old.push_str(text.as_ref());
            old
        } else {
            String::from(text.as_ref())
        };
        self.put_into_cache(&key, &message).await;
    }

    pub async fn retrieve_and_erase_contact(&self, user_id: u64) -> String {
        let key = format!("contact:{}", user_id);
        let message = self
            .collect_from_cache::<String>(&key)
            .await
            .unwrap_or_default();
        self.put_into_cache(&key, &String::new()).await;
        message
    }

    pub async fn get_all_users(&self) -> Vec<u64> {
        self.get_keys("user:*")
            .await
            .into_iter()
            .map(|key| {
                key.split(':')
                    .next_back()
                    .unwrap_or("0")
                    .parse()
                    .unwrap_or(0)
            })
            .collect()
    }

    fn string_to_task<S: AsRef<str>>(text: S) -> anyhow::Result<Task> {
        let lines = text
            .as_ref()
            .lines()
            .map(|x| x.trim().to_string())
            .collect::<Vec<String>>();
        if lines.len() < 3 {
            bail!(r"Должно быть 3 или больше строки: имя, флаг, описание.")
        }
        let flag = {
            let flag_str = lines[1]
                .split(',')
                .map(|x| x.trim().to_lowercase())
                .collect::<Vec<String>>();
            if flag_str.len() == 1 {
                FlagType::Single(flag_str.into_iter().next().unwrap())
            } else {
                FlagType::Multi(flag_str)
            }
        };
        let mut name = lines[0].as_str();
        let hint = lines[2..].join("\n");
        let hidden = if name.starts_with("hidden:") {
            name = name.strip_prefix("hidden:").unwrap();
            true
        } else {
            false
        };

        Ok(Task {
            name: name.trim().to_owned(),
            flag,
            hidden,
            hint: hint.trim().to_owned(),
            id: String::new(),
        })
    }

    pub async fn edit_task<S1: AsRef<str>, S2: AsRef<str>>(
        &self,
        task_id: S1,
        text: S2,
    ) -> anyhow::Result<()> {
        let task = Self::string_to_task(text)?;
        let key = format!("task:{}", task_id.as_ref());
        // lock
        let mut val = self.mutex.lock().await;
        *val = true;
        self.put_into_cache(&key, &task).await;
        *val = false;
        drop(val);
        Ok(())
    }
    pub async fn delete_task<S1: AsRef<str>>(&self, task_id: S1) -> anyhow::Result<()> {
        let key = format!("task:{}", task_id.as_ref());
        // lock
        let mut val = self.mutex.lock().await;
        *val = true;
        self.del_from_cache(&key).await;
        *val = false;
        drop(val);
        Ok(())
    }

    pub async fn get_scoreboard(&self) -> Vec<(Vas3kUser, u64)> {
        let mut ret: Vec<(Vas3kUser, u64)> = Vec::new();
        for key in self.get_keys("user:*").await {
            let Some(user) = self.collect_from_cache::<Vas3kUser>(&key).await else {
                continue;
            };
            let solv_key = format!("solve:{}", key.strip_prefix("user:").unwrap());
            let score = match self.collect_from_cache::<Solve>(&solv_key).await {
                Some(solve) => solve.into(),
                None => 0u64,
            };
            if self.is_test_user(user.telegram_id as u64) {
                ret.push((user, 0));
            } else {
                ret.push((user, score));
            }
        }
        ret.sort_by(|x, y| y.1.cmp(&x.1));

        ret
    }
}
