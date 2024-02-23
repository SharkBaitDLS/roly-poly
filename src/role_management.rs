use std::{str::FromStr, sync::RwLock};

use bimap::BiMap;
use log::{error, warn};
use pickledb::PickleDb;
use serenity::{
    builder::{CreateInteractionResponse, CreateInteractionResponseMessage},
    futures::{stream::FuturesUnordered, StreamExt},
    model::{
        application::{CommandDataOption, CommandDataOptionValue, CommandInteraction},
        channel::ReactionType,
        id::EmojiId,
        misc::EmojiIdentifier,
    },
    prelude::Context,
};

use crate::{
    database::{get_guild_data, update_guild_data},
    guild_data::GuildData,
    util::get_guild_id,
};

pub async fn respond_to_command<S>(ctx: &Context, command: &CommandInteraction, content: S)
where
    S: Into<String>,
{
    if let Err(e) = command
        .create_response(
            &ctx,
            CreateInteractionResponse::Message(
                CreateInteractionResponseMessage::new()
                    .ephemeral(true)
                    .content(content),
            ),
        )
        .await
    {
        error!("Could not respond to command: {:?}", e);
    }
}

pub async fn enable_role(
    ctx: &Context,
    db: &RwLock<PickleDb>,
    command: &CommandInteraction,
    opt: &CommandDataOption,
) {
    if let CommandDataOptionValue::SubCommand(options) = &opt.value {
        match &options[0..2] {
            [CommandDataOption {
                name: opt1_name,
                value: CommandDataOptionValue::Role(role_id),
                ..
            }, CommandDataOption {
                name: op2_name,
                value: CommandDataOptionValue::String(emoji_name),
                ..
            }] if opt1_name == "role" && op2_name == "emoji" => {
                let guild_id = get_guild_id(command);
                let guild_data = get_guild_data(db, guild_id);
                let maybe_emoji = get_emoji(ctx, emoji_name).await;

                if let Some(emoji) = maybe_emoji {
                    if let Some(mut data) = guild_data {
                        data.add_role(ctx, (*role_id).into(), emoji).await;
                        update_guild_data(db, guild_id, &data);
                    } else {
                        let mut roles_to_emoji: BiMap<u64, ReactionType> = BiMap::new();
                        roles_to_emoji.insert((*role_id).into(), emoji);

                        update_guild_data(db, guild_id, &GuildData::new(roles_to_emoji));
                    }

                    respond_to_command(
                        ctx,
                        command,
                        format!(
                            "Enabled {} for self-service access",
                            command.data.resolved.roles[&role_id].name
                        ),
                    )
                    .await;
                } else {
                    respond_to_command(ctx, command, format!("Could not find emoji: {emoji_name}"))
                        .await;
                }
            }
            _ => warn!("A command was invoked with unexpected arguments, Discord should have prevented this"),
        }
    }
}

pub async fn disable_role(
    ctx: &Context,
    db: &RwLock<PickleDb>,
    command: &CommandInteraction,
    opt: &CommandDataOption,
) {
    if let CommandDataOptionValue::SubCommand(options) = &opt.value {
        match options.first() {
            Some(CommandDataOption {
                name,
                value: CommandDataOptionValue::Role(role_id),
                ..
            }) if name == "role" => {
                let guild_id = get_guild_id(command);
                let guild_data = get_guild_data(db, guild_id);

                if let Some(mut data) = guild_data {
                    data.remove_role(ctx, (*role_id).into()).await;
                    update_guild_data(db, guild_id, &data);
                }

                respond_to_command(
                    ctx,
                    command,
                    format!(
                        "Disabled {} for self-service access",
                        command.data.resolved.roles[&role_id].name
                    ),
                )
                .await;
            }
            _ => warn!("A command was invoked with unexpected arguments, Discord should have prevented this"),
        }
    }
}

pub async fn create_message(
    ctx: &Context,
    db: &RwLock<PickleDb>,
    command: &CommandInteraction,
    opt: &CommandDataOption,
) {
    if let CommandDataOptionValue::SubCommand(options) = &opt.value {
        match options.first() {
            Some(CommandDataOption {
                name,
                value: CommandDataOptionValue::Channel(channel_id),
                ..
            }) if name == "channel" => {
                let guild_id = get_guild_id(command);
                let guild_data = get_guild_data(db, guild_id);

                match guild_data {
                    Some(mut data) => {
                        respond_to_command(
                            ctx,
                            command,
                            format!(
                                "Sending a message to #{} if one does not already exist",
                                command.data.resolved.channels[&channel_id]
                                    .name
                                    .as_ref()
                                    .expect("Channels should be named")
                            ),
                        )
                        .await;

                        update_guild_data(db, guild_id, data.send_message(ctx, *channel_id).await);
                    }
                    None => {
                        respond_to_command(ctx, command, "You have not configured any roles").await;
                    }
                }
            }
            _ => warn!("A command was invoked with unexpected arguments, Discord should have prevented this"),
        }
    }
}

async fn get_emoji(ctx: &Context, emoji_name: &str) -> Option<ReactionType> {
    let all_emoji: Vec<EmojiId> = ctx
        .cache
        .guilds()
        .iter()
        .map(|guild| guild.emojis(&ctx))
        .collect::<FuturesUnordered<_>>()
        .collect::<Vec<_>>()
        .await
        .into_iter()
        .filter_map(|result| {
            result
                .ok()
                .map(|emojis| emojis.iter().map(|emoji| emoji.id).collect::<Vec<_>>())
        })
        .flatten()
        .collect();

    if emoji_name.starts_with('<') {
        EmojiIdentifier::from_str(emoji_name)
            .ok()
            .filter(|identifier| all_emoji.contains(&identifier.id))
            .map(Into::into)
    } else {
        emoji_name.chars().next().map(Into::into)
    }
}
