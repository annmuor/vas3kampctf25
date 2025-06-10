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
use crate::api::{FlagType, Task, Vas3kUser};
use std::fmt::Display;

pub const HELP_TEXT: &str = r"
Привет!

Это бот для Вастрик.Кемпа в Либерленде!

Он показывает задания (/tasks), правила (/rules) и твой счёт (/score).
Ты всегда можешь написать (/contact) оргам и что-то спросить.

Ответом на каждое задания является флаг: ключевое слово, набор букв и цифр или чего-то ещё.
Например, флаг может выглядеть так: CTF{Th1s_1s_fl4g}

Задания могут (и будут) появляться по мере возможности, так что бот будет писать тебе сообщения с анонсами.";
pub const RULES_TEXT: &str = r"Правила!

1. Не укради флаг у ближнего своего, ищи сам!
2. Не подавай флага ближнему своему, пусть ищет сам!
3. Не взламывай бота, он тут не для этого!
4. Каждый флаг даёт 1 балл.
5. Кто наберет больше всех баллов - выиграл.
6. Призовых мест будет от 1 до 3, в зависимости от числа участников.
7. Флаг может быть где угодно! У организаторов богатая фантазия!
8. Игра начинается 5 июня в 10:00 утра.
9. Игра заканчивается 7 июня в 19:00 вечера.
";

pub const CONTACT_TEXT: &str = r"Напиши своё сообщение. Или несколько.
Всё, что ты напишешь, будет отправлено организаторам AS IS. Допускается только текст.
Когда закончишь писать - поставь точку (.) отдельным сообщением";

pub const MESSAGE_TEXT: &str = r"Напиши своё сообщение. Всё, что ты напишешь, будет отправлено всем участникам (кто ходил в бота) AS IS.
Когда закончишь писать - поставь точку (.) отдельным сообщением";

pub const CODE_TEXT: &str = r"К сожалению, код бота сейчас недоступен";

pub const UNKNOWN_TEXT: &str = r"Неизвестная команда, попробуй начать с /help";

pub const DENIED: &str = r"Доступ запрещен!";

pub const NOT_YET: &str =
    r"Игра начнётся в 10:00 четверга и закончится в 19:00 субботы! Посмотри пока /rules и /help";

pub const NOT_IMPLEMENTED: &str = r"Еще не готово!";

pub const MESSAGE_SENT: &str = r"Ваше сообщение было отправлено";

pub const CREATE_TASK: &str = r"Отправь задание в 3+ строки одним сообщением:
1. Название
2. Флаг
3. Описание";

pub const ALREADY_SOLVED: &str = r"Это задание уже решено!";

pub const ALL_SOLVED: &str =
    r"Ты уже всё решил! Подожди немного, может быть появятся новые задания...";

pub const CHOOSE: &str = r"Выбери задание:";

pub const CONFIG_NAME: &str = r"config.json";

pub const VAR_NAME: &str = r"BOTFLAG";
pub struct Format(());

impl Format {
    fn score(score: u64) -> String {
        let end = match score {
            1 => r"балл",
            2..=4 => r"балла",
            0 | 5..=21 => r"баллов",
            x => match x % 10 {
                1 => r"балл",
                2..=4 => r"балла",
                _ => r"баллов",
            },
        };
        format!("{} {}", score, end)
    }
    pub fn format_score_board(place: usize, user: &Vas3kUser, score: u64) -> String {
        format!("{}. {} - {}\n", place, user, Self::score(score))
    }
    pub fn format_score(place: u64, score: u64) -> String {
        if place == u64::MAX {
            format!(r"Ты в тестовой группе со счётом {}!", Self::score(score))
        } else {
            format!(r"Ты на {place} месте со счётом {}!", Self::score(score))
        }
    }
    pub fn format_task_user(task: &Task) -> String {
        format!(
            r"<b>{}</b>
<i>{}</i>
<tg-spoiler>/contact_{} - Сообщить о проблеме</tg-spoiler>
---
",
            task.name, task.hint, task.id
        )
    }

    pub fn format_task_admin(task: &Task) -> String {
        let flag = match task.flag {
            FlagType::Single(ref s) => s.clone(),
            FlagType::Multi(ref vs) => vs.join(","),
        };
        let prefix = if task.hidden { "hidden:" } else { "" };
        format!(
            r"Старые поля задания:
<code>
{prefix}{}
{}
{}
</code>
",
            task.name, flag, task.hint
        )
    }

    pub fn format_solved(name: &str) -> String {
        format!(r"Задание <b>{name}</b> успешно решено!")
    }

    pub fn format_deleted(name: &str) -> String {
        format!(r"Задание <b>{name}</b> было удалено")
    }

    pub fn format_modified(name: &str) -> String {
        format!(r"Задание <b>{name}</b> было изменено")
    }

    pub fn format_created(name: &str) -> String {
        format!(r"Задание <b>{name}</b> было создано")
    }

    pub fn format_message_broadcast(text: &str) -> String {
        format!(
            r"<b>Вам сообщение</b>:
{text}"
        )
    }

    pub fn format_message(from: &str, message: &str, task: Option<&str>) -> String {
        match task {
            None => format!(
                r"<b>Сообщение от @{from}</b>:

{message}
",
            ),
            Some(task) => {
                format!(
                    r"<b>Сообщение от @{from} по поводу задания <i>{task}</i></b>:

{message}
",
                )
            }
        }
    }
    pub fn format_error<E: Display>(error: E) -> String {
        format!(r"Возникла ошибка: {error}")
    }

    pub fn format_solved_admin<S1: Display, S2: Display>(user: S1, task: S2) -> String {
        format!(r"Пользователь {user} решил задачу {task}")
    }
}
