use std::sync::RwLock;

use log::error;
use pickledb::PickleDb;
use serenity::model::prelude::GuildId;

use crate::guild_data::GuildData;

pub fn get_guild_data(db: &RwLock<PickleDb>, guild_id: &GuildId) -> Option<GuildData> {
    // TODO: decide how to handle poisoned lock
    db.read().unwrap().get::<GuildData>(&guild_id.to_string())
}

pub fn update_guild_data(db: &RwLock<PickleDb>, guild_id: &GuildId, new_data: &GuildData) {
    // TODO: decide how to handle poisoned lock
    if let Err(e) = db
        .write()
        .unwrap()
        .set::<GuildData>(&guild_id.to_string(), new_data)
    {
        error!(
            "Could not write guild data to database for guild {:?}: {}",
            guild_id, e
        )
    }
}
