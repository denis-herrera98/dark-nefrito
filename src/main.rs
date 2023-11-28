mod commands;
mod interpreter;

use std::env;

use serenity::{
    async_trait,
    client::{Client, Context, EventHandler},
    model::gateway::Ready,
    prelude::GatewayIntents,
    framework::{
        standard::macros::group,
        StandardFramework,
    },
};

use songbird::{
    driver::DecodeMode,
    Config,
    SerenityInit,
};

use crate::commands::voice::*;

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, _: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);
    }
}

#[group]
#[commands(join, leave)]
struct General;

#[tokio::main]
async fn main() {
//      interpreter::start_model().unwrap();
//      interpreter::write_example();
   // let mut recognizer = RecognizerSingleton::get_instance().lock().unwrap();
    dotenv::dotenv().expect("Failed to load .env file");
    let token = env::var("DISCORD_TOKEN").expect("Expected a token in the environment");
    let prefix = "n!";
    tracing_subscriber::fmt::init();

    let framework = StandardFramework::new().group(&GENERAL_GROUP);
    framework.configure(|c| c.prefix(prefix));

    let intents = GatewayIntents::non_privileged() | GatewayIntents::MESSAGE_CONTENT;

    // Here, we need to configure Songbird to decode all incoming voice packets.
    // If you want, you can do this on a per-call basis---here, we need it to
    // read the audio data that other people are sending us!
    let songbird_config = Config::default().decode_mode(DecodeMode::Decode);

    let mut client = Client::builder(&token, intents)
        .event_handler(Handler)
        .framework(framework)
        .register_songbird_from_config(songbird_config)
        .await
        .expect("Err creating client");

    let _ = client
        .start()
        .await
        .map_err(|why| println!("Client ended: {:?}", why));
}
