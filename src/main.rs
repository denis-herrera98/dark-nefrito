mod repository;
mod models;
use std::path::Path;

fn main(){
    let path = Path::new("./data/dark-souls.json");
    let json_string = repository::read_file(path).unwrap();
    let data = repository::string_to_hasmap(&json_string);

    let dark_data = models::DarkSoulsData {
     users: data 
        .into_iter()
        .map(|(key, value)| {
            let player: models::DarkSoulsUsers = serde_json::from_value(value)
                .expect("Error al convertir el valor a Player");
            (key, player)
        })
        .collect()
    };

    println!("{:?}", dark_data.users.get("1").unwrap());
}

//use std::env;
//use dotenv::dotenv;
//
//use serenity::async_trait;
//use serenity::prelude::*;
//use serenity::model::channel::Message;
//use serenity::framework::standard::macros::{command, group};
//use serenity::framework::standard::{StandardFramework, CommandResult};
//
//#[group]
//#[commands(ping)]
//struct General;
//
//struct Handler;
//
//#[async_trait]
//impl EventHandler for Handler {}
//
//#[tokio::main]
//async fn main() {
//    dotenv().ok();
//
//    const PREFIX: &str = "!d";
//    
//    let framework = StandardFramework::new()
//        .configure(|c| c.prefix(PREFIX)) // set the bot's prefix to "~"
//        .group(&GENERAL_GROUP);
//
//    // Login with a bot token from the environment
//    let token = env::var("DISCORD_TOKEN").expect("token");
//    let intents = GatewayIntents::non_privileged() | GatewayIntents::MESSAGE_CONTENT;
//    let mut client = Client::builder(token, intents)
//        .event_handler(Handler)
//        .framework(framework)
//        .await
//        .expect("Error creating client");
//
//    // start listening for events by starting a single shard
//    if let Err(why) = client.start().await {
//        println!("An error occurred while running the client: {:?}", why);
//    }
//}
//
//#[command]
//async fn ping(ctx: &Context, msg: &Message) -> CommandResult {
//    msg.reply(ctx, "Pong!").await?;
//
//    Ok(())
//}
