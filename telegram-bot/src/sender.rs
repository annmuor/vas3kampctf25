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
use log::{debug, error, info};
use std::collections::HashMap;
use std::sync::atomic::AtomicI64;
use std::sync::atomic::Ordering::SeqCst;
use std::sync::Arc;
use std::time::{Duration, SystemTime};
use teloxide::adaptors::DefaultParseMode;
use teloxide::payloads::SendMessageSetters;
use teloxide::requests::{Requester, RequesterExt};
use teloxide::sugar::request::RequestLinkPreviewExt;
use teloxide::types::{ChatId, ParseMode, ReplyMarkup};
use teloxide::Bot;
use tokio::sync::mpsc::{Receiver, Sender};
use tokio::time::sleep;

pub struct Message(i64, String, Option<ReplyMarkup>);

impl<S> From<(i64, S)> for Message
where
    S: AsRef<str>,
{
    fn from(value: (i64, S)) -> Self {
        Self(value.0, String::from(value.1.as_ref()), None)
    }
}

impl<S> From<(i64, S, ReplyMarkup)> for Message
where
    S: AsRef<str>,
{
    fn from(value: (i64, S, ReplyMarkup)) -> Self {
        Self(value.0, String::from(value.1.as_ref()), Some(value.2))
    }
}

pub struct MessageSender {
    recv: Receiver<Message>,
    send: Sender<Message>,
    bot: DefaultParseMode<Bot>,
}

const LIMIT_RATE_PER_CHAT: u128 = 1000; // 1 sec
const LIMIT_RATE_PER_ALL: i64 = 30; // 30/sec

impl MessageSender {
    pub fn new(bot: Bot) -> Self {
        let bot = bot.parse_mode(ParseMode::Html);
        let (send, recv) = tokio::sync::mpsc::channel(1024);
        Self { recv, send, bot }
    }

    pub fn sender(&self) -> Sender<Message> {
        self.send.clone()
    }

    #[inline(always)]
    async fn send_message(&self, message: Message) -> Result<(), (anyhow::Error, Message)> {
        let fut = self
            .bot
            .send_message(ChatId(message.0), &message.1)
            .disable_link_preview(true);
        let res = match &message.2 {
            None => fut.await,
            Some(kbd) => fut.reply_markup(kbd.clone()).await,
        };
        if let Err(e) = res {
            Err((e.into(), message))
        } else {
            Ok(())
        }
    }

    pub async fn start(mut self) {
        let mut timeouts: HashMap<i64, SystemTime> = HashMap::new();
        let counter = Arc::new(AtomicI64::new(LIMIT_RATE_PER_ALL));
        {
            let counter = counter.clone();
            tokio::spawn(async move {
                loop {
                    sleep(Duration::from_secs(1)).await;
                    counter.store(LIMIT_RATE_PER_ALL, SeqCst); // set to 30 each second
                }
            });
        }
        while let Some(mut message) = self.recv.recv().await {
            // check for global rate limit
            loop {
                let left = counter.load(SeqCst);
                if left < 1 {
                    sleep(Duration::from_millis(100)).await;
                } else {
                    counter.fetch_add(-1, SeqCst);
                    break;
                }
            }

            if let Some(t) = timeouts.get(&message.0) {
                if let Ok(elapsed) = t.elapsed() {
                    if elapsed.as_millis() < LIMIT_RATE_PER_CHAT {
                        debug!("Message is not ready for {}, push_back", message.0);
                        if let Err(e) = self.send.send(message).await {
                            // we can't push_back - we must wait
                            sleep(Duration::from_millis(
                                (LIMIT_RATE_PER_CHAT - elapsed.as_millis()) as u64,
                            ))
                            .await;
                            message = e.0;
                        } else {
                            continue;
                        }
                    }
                }
            }
            // send immediately
            let id = message.0;
            if let Err((e, message)) = self.send_message(message).await {
                info!("Error sending message to {}: {}", message.0, e);
                // resend
                if let Err(e) = self.send.send(message).await {
                    error!("Error sending message: {:?}", e);
                }
            } else {
                timeouts.insert(id, SystemTime::now());
            }
        }
    }
}
