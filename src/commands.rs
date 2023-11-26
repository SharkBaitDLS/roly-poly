use serenity::{
    builder::CreateApplicationCommand,
    model::{prelude::command::CommandOptionType, Permissions},
};

#[cfg(debug_assertions)]
use serenity::{model::prelude::command::Command, prelude::Context};

pub fn create(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
    command
        .name("role")
        .dm_permission(false)
        .create_option(|self_service| {
            self_service
                .name("self-service")
                .kind(CommandOptionType::SubCommandGroup)
                .description("modify self-service role enrollment permissions")
                .create_sub_option(|enable| {
                    enable
                        .name("enable")
                        .kind(CommandOptionType::SubCommand)
                        .description("enable a role for self-service enrollment")
                        .create_sub_option(|role| {
                            role.name("role")
                                .kind(CommandOptionType::Role)
                                .description("the role to enable")
                                .required(true)
                        })
                        .create_sub_option(|emoji| {
                            emoji
                                .name("emoji")
                                .kind(CommandOptionType::String)
                                .description("the emoji to associate with this role")
                                .required(true)
                        })
                })
                .create_sub_option(|disable| {
                    disable
                        .name("disable")
                        .kind(CommandOptionType::SubCommand)
                        .description("disable a role for self-service enrollment")
                        .create_sub_option(|role| {
                            role.name("role")
                                .kind(CommandOptionType::Role)
                                .description("the role to disable")
                                .required(true)
                        })
                })
                .create_sub_option(|message| {
                    message
                        .name("message")
                        .kind(CommandOptionType::SubCommand)
                        .description("create a message for users to react to")
                        .create_sub_option(|channel| {
                            channel
                                .name("channel")
                                .kind(CommandOptionType::Channel)
                                .description("the channel to message in")
                                .required(true)
                        })
                })
        })
        .description("modify roles")
        .default_member_permissions(Permissions::MANAGE_ROLES)
}

#[cfg(debug_assertions)]
pub async fn create_for_test_guild(ctx: &Context) -> serenity::Result<Command> {
    use serenity::model::prelude::GuildId;
    use std::env;

    let guild_id = GuildId(
        env::var("DEBUG_GUILD_ID")
            .expect("Expected DEBUG_GUILD_ID to be set for testing")
            .parse()
            .expect("DEBUG_GUILD_ID must be an integer"),
    );

    guild_id.create_application_command(&ctx, create).await
}
