use std::sync::RwLock;

use log::error;
use pickledb::PickleDb;
use serenity::model::prelude::GuildId;

use crate::guild_data::GuildData;

pub fn get_guild_data(db: &RwLock<PickleDb>, guild_id: GuildId) -> Option<GuildData> {
    db.read()
        .expect("The database lock is poisoned due to a panic on write")
        .get::<GuildData>(&guild_id.to_string())
}

pub fn update_guild_data(db: &RwLock<PickleDb>, guild_id: GuildId, new_data: &GuildData) {
    if let Err(e) = db
        .write()
        .expect("The database lock is poisoned due to a panic on write")
        .set::<GuildData>(&guild_id.to_string(), new_data)
    {
        error!(
            "Could not write guild data to database for guild {:?}: {}",
            guild_id, e
        );
    }
}
