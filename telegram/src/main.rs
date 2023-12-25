use teloxide::prelude::*;

mod sheets;

#[tokio::main]
async fn main() {
    pretty_env_logger::init();
    log::info!("Starting bot...");

    let bot = Bot::from_env();

    bot.delete_webhook().send().await.log_on_error().await;

    teloxide::repl(bot, |bot: Bot, msg: Message| async move {
        let text = msg
            .text()
            .clone()
            .unwrap_or_default()
            .split(' ')
            .collect::<Vec<&str>>();

        let sum = text
            .get(0)
            .unwrap_or(&"0")
            .parse::<i32>()
            .unwrap_or_default();
        let description = text.get(1).unwrap_or(&"nothing");

        sheets::sheets::write_new_spend(sum, description.to_string()).await;

        let concated = "I write ".to_string() + &sum.to_string() + " for " + &description;

        bot.send_message(msg.chat.id, concated).await?;
        Ok(())
    })
    .await;
}
