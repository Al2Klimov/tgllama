mod cli;

use cli::{exit_err, log_err, noempty_utf8_env, require_noempty_utf8_env};
use ollama_rs::{Ollama, generation::completion::request::GenerationRequest};
use std::{collections::HashSet, sync::Arc};
use teloxide::{
    Bot,
    payloads::SetMessageReactionSetters,
    prelude::Requester,
    repl,
    sugar::request::RequestReplyExt,
    types::{Message, ReactionType},
};

#[tokio::main]
async fn main() {
    let bot = Bot::new(exit_err(require_noempty_utf8_env("TGLLAMA_BOTTOKEN")));
    let model = Arc::new(exit_err(require_noempty_utf8_env("TGLLAMA_MODEL")));

    let whitelist = Arc::new(match exit_err(noempty_utf8_env("TGLLAMA_WHITELIST")) {
        None => Default::default(),
        Some(v) => v
            .split(" ")
            .map(|s| s.to_string())
            .collect::<HashSet<String>>(),
    });

    let ollama = Arc::new(match exit_err(noempty_utf8_env("TGLLAMA_OLLAMAURL")) {
        None => Ollama::default(),
        Some(url) => exit_err(Ollama::try_new(url)),
    });

    repl(bot, move |bot: Bot, msg: Message| {
        let model = model.clone();
        let whitelist = whitelist.clone();
        let ollama = ollama.clone();

        async move {
            if whitelist.contains(&msg.chat.id.to_string()) {
                match msg.text() {
                    None => {
                        log_err(
                            bot.set_message_reaction(msg.chat.id, msg.id)
                                .reaction(vec![ReactionType::Emoji {
                                    emoji: "🤷".into()
                                }])
                                .await,
                        );
                    }
                    Some("/start") => {
                        log_err(
                            bot.set_message_reaction(msg.chat.id, msg.id)
                                .reaction(vec![ReactionType::Emoji {
                                    emoji: "👍".into()
                                }])
                                .await,
                        );
                    }
                    Some(input) => {
                        log_err(
                            bot.set_message_reaction(msg.chat.id, msg.id)
                                .reaction(vec![ReactionType::Emoji {
                                    emoji: "🤔".into()
                                }])
                                .await,
                        );

                        match ollama
                            .generate(GenerationRequest::new((*model).clone(), input))
                            .await
                        {
                            Err(err) => {
                                eprintln!("{}", err);
                                log_err(
                                    bot.send_message(msg.chat.id, "Error!")
                                        .reply_to(msg.id)
                                        .await,
                                );
                            }
                            Ok(resp) => {
                                log_err(
                                    bot.send_message(msg.chat.id, resp.response)
                                        .reply_to(msg.id)
                                        .await,
                                );
                            }
                        }

                        log_err(bot.set_message_reaction(msg.chat.id, msg.id).await);
                    }
                }
            } else {
                log_err(
                    bot.send_message(
                        msg.chat.id,
                        format!(
                            "Access denied! Ask my admin to whitelist this chat #{}.",
                            msg.chat.id.0
                        ),
                    )
                    .reply_to(msg.id)
                    .await,
                );
            }

            Ok(())
        }
    })
    .await;
}
