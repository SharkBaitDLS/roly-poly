use serenity::model::{application::CommandInteraction, id::GuildId};

pub fn get_guild_id(command: &CommandInteraction) -> GuildId {
    command
        .guild_id
        .expect("Command is not allowed for use in DMs")
}
