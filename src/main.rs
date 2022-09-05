mod commands;
mod database;
mod guild_data;
mod handler;
mod role_management;
mod util;

use std::{env, path::Path};

use handler::Handler;
use log::error;
use pickledb::{PickleDb, PickleDbDumpPolicy};
use serenity::prelude::{Client, GatewayIntents};

#[tokio::main]
async fn main() {
    env_logger::init();

    let token = env::var("DISCORD_BOT_TOKEN")
        .expect("Expected DISCORD_BOT_TOKEN environment variable to be set");

    let db_name = "roly-poly-rolies.db";
    let db = if Path::new(db_name).exists() {
        PickleDb::load_json(db_name, PickleDbDumpPolicy::AutoDump).expect("A valid database file")
    } else {
        PickleDb::new_json(db_name, PickleDbDumpPolicy::AutoDump)
    };

    let mut client = Client::builder(token, GatewayIntents::GUILD_MESSAGE_REACTIONS)
        .event_handler(Handler::new(db))
        .await
        .expect("Could not start bot");

    if let Err(why) = client.start().await {
        error!("Bot client error: {:?}", why);
    }
}
