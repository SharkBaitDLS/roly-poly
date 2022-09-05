use serenity::model::prelude::{
    interaction::application_command::ApplicationCommandInteraction, GuildId,
};

pub fn get_guild_id(command: &ApplicationCommandInteraction) -> GuildId {
    command
        .guild_id
        .expect("Command is not allowed for use in DMs")
}
