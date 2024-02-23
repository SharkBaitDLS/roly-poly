use serenity::{
    builder::{CreateCommand, CreateCommandOption},
    model::{application::CommandOptionType, Permissions},
};

#[cfg(debug_assertions)]
use serenity::{client::Context, model::application::Command};

pub fn create() -> CreateCommand {
    CreateCommand::new("role")
        .dm_permission(false)
        .description("modify roles")
        .default_member_permissions(Permissions::MANAGE_ROLES)
        .add_option(
            CreateCommandOption::new(
                CommandOptionType::SubCommandGroup,
                "self-service",
                "modify self-service role enrollment permissions",
            )
            .add_sub_option(
                CreateCommandOption::new(
                    CommandOptionType::SubCommand,
                    "enable",
                    "enable a role for self-service enrollment",
                )
                .add_sub_option(
                    CreateCommandOption::new(CommandOptionType::Role, "role", "the role to enable")
                        .required(true),
                )
                .add_sub_option(
                    CreateCommandOption::new(
                        CommandOptionType::String,
                        "emoji",
                        "the emoji to associate with this role",
                    )
                    .required(true),
                ),
            )
            .add_sub_option(
                CreateCommandOption::new(
                    CommandOptionType::SubCommand,
                    "disable",
                    "disable a role for self-service enrollment",
                )
                .add_sub_option(
                    CreateCommandOption::new(
                        CommandOptionType::Role,
                        "role",
                        "the role to disable",
                    )
                    .required(true),
                ),
            )
            .add_sub_option(
                CreateCommandOption::new(
                    CommandOptionType::SubCommand,
                    "message",
                    "create a message for users to react to",
                )
                .add_sub_option(
                    CreateCommandOption::new(
                        CommandOptionType::Channel,
                        "channel",
                        "the channel to message in",
                    )
                    .required(true),
                ),
            ),
        )
}

#[cfg(debug_assertions)]
pub async fn create_for_test_guild(ctx: &Context) -> serenity::Result<Command> {
    use serenity::model::prelude::GuildId;
    use std::env;

    let guild_id = GuildId::new(
        env::var("DEBUG_GUILD_ID")
            .expect("Expected DEBUG_GUILD_ID to be set for testing")
            .parse()
            .expect("DEBUG_GUILD_ID must be an integer"),
    );

    guild_id.create_command(&ctx, create()).await
}
