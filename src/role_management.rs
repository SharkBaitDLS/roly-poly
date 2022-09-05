use std::{fmt, str::FromStr, sync::RwLock};

use bimap::BiMap;
use log::error;
use pickledb::PickleDb;
use serenity::{
    model::prelude::{
        interaction::{
            application_command::{
                ApplicationCommandInteraction, CommandDataOption, CommandDataOptionValue,
            },
            InteractionResponseType,
        },
        EmojiIdentifier, ReactionType,
    },
    prelude::Context,
};

use crate::{
    database::{get_guild_data, update_guild_data},
    guild_data::GuildData,
    util::get_guild_id,
};

pub async fn respond_to_command<S>(
    ctx: &Context,
    command: &ApplicationCommandInteraction,
    content: S,
) where
    S: fmt::Display,
{
    if let Err(e) = command
        .create_interaction_response(&ctx, |response| {
            response
                .kind(InteractionResponseType::ChannelMessageWithSource)
                .interaction_response_data(|data| data.ephemeral(true).content(content))
        })
        .await
    {
        error!("Could not respond to command: {:?}", e);
    }
}

pub async fn enable_role(
    ctx: &Context,
    db: &RwLock<PickleDb>,
    command: &ApplicationCommandInteraction,
    opt: &CommandDataOption,
) {
    if let Some(CommandDataOptionValue::Role(role)) =
        opt.options.first().and_then(|arg| arg.resolved.as_ref())
    {
        if let Some(CommandDataOptionValue::String(emoji_name)) =
            opt.options.get(1).and_then(|arg| arg.resolved.as_ref())
        {
            let guild_id = get_guild_id(command);
            let guild_data = get_guild_data(db, &guild_id);
            let maybe_emoji = get_emoji(emoji_name).await;

            if let Some(emoji) = maybe_emoji {
                match guild_data {
                    Some(mut data) => {
                        data.add_role(ctx, *role.id.as_u64(), emoji).await;
                        update_guild_data(db, &guild_id, &data);
                    }
                    None => {
                        let mut roles_to_emoji: BiMap<u64, ReactionType> = BiMap::new();
                        roles_to_emoji.insert(*role.id.as_u64(), emoji);

                        update_guild_data(db, &guild_id, &GuildData::new(roles_to_emoji));
                    }
                }

                respond_to_command(
                    ctx,
                    command,
                    format!("Enabled {} for self-service access", role.name),
                )
                .await;
            } else {
                respond_to_command(
                    ctx,
                    command,
                    format!("Could not find emoji: {}", emoji_name),
                )
                .await;
            }
        }
    }
}

pub async fn disable_role(
    ctx: &Context,
    db: &RwLock<PickleDb>,
    command: &ApplicationCommandInteraction,
    opt: &CommandDataOption,
) {
    if let Some(CommandDataOptionValue::Role(role)) =
        opt.options.first().and_then(|arg| arg.resolved.as_ref())
    {
        let guild_id = get_guild_id(command);
        let guild_data = get_guild_data(db, &guild_id);

        if let Some(mut data) = guild_data {
            data.remove_role(ctx, role.id.as_u64()).await;
            update_guild_data(db, &guild_id, &data);
        }

        respond_to_command(
            ctx,
            command,
            format!("Disabled {} for self-service access", role.name),
        )
        .await;
    }
}

pub async fn create_message(
    ctx: &Context,
    db: &RwLock<PickleDb>,
    command: &ApplicationCommandInteraction,
    opt: &CommandDataOption,
) {
    if let Some(CommandDataOptionValue::Channel(channel)) =
        opt.options.first().and_then(|arg| arg.resolved.as_ref())
    {
        let guild_id = get_guild_id(command);
        let guild_data = get_guild_data(db, &guild_id);

        match guild_data {
            Some(mut data) => {
                respond_to_command(
                    ctx,
                    command,
                    format!(
                        "Sending a message to #{} if one does not already exist",
                        channel.name.as_ref().expect("Channels should be named")
                    ),
                )
                .await;

                update_guild_data(db, &guild_id, data.send_message(ctx, channel.id).await);
            }
            None => {
                respond_to_command(ctx, command, "You have not configured any roles").await;
            }
        }
    }
}

async fn get_emoji(emoji_name: &str) -> Option<ReactionType> {
    if emoji_name.starts_with('<') {
        // TODO: validate that the bot can use the provided emoji
        EmojiIdentifier::from_str(emoji_name)
            .ok()
            .map(|identifier| identifier.into())
    } else {
        emoji_name.chars().next().map(|char| char.into())
    }
}
