use poise::serenity_prelude as serenity;

struct Data {}
type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, Data, Error>;

use asucksra_crawler::get_manhwa_chapter_img_urls;

const BOT_TOKEN: &str = "YOUR BOT TOKEN HEREEE";

#[poise::command(slash_command, prefix_command)]
async fn scrape(
    ctx: Context<'_>,
    #[description = "Name of series to be searched"] manhwa: String,
    #[description = "Chapter to scrape"] chapter: u16,
) -> Result<(), Error> {
    let img_urls = get_manhwa_chapter_img_urls(&manhwa, chapter).await;
    match img_urls {
        Ok(Some(v)) => {
            if v.is_empty() {
                ctx.say("couldnt find shit lmao").await?;
            } else {
                for chunk in v.chunks(5) {
                    let message = chunk.join("\n");
                    ctx.say(message).await?;
                }
            }
        }
        _ => {
            ctx.say("couldnt find shit lmao").await?;
        }
    }
    Ok(())
}

#[tokio::main]
async fn main() {
    let intents = serenity::GatewayIntents::non_privileged();

    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: vec![scrape()],
            ..Default::default()
        })
        .setup(|ctx, _ready, framework| {
            Box::pin(async move {
                poise::builtins::register_globally(ctx, &framework.options().commands).await?;
                Ok(Data {})
            })
        })
        .build();

    let client = serenity::ClientBuilder::new(BOT_TOKEN, intents)
        .framework(framework)
        .await;
    client.unwrap().start().await.unwrap();
}
