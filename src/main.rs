use std::env;

use log::error;
use serenity::{
    async_trait,
    builder::CreateApplicationCommand,
    model::{
        application::command::Command,
        prelude::{command::CommandOptionType, interaction::Interaction, GuildId, Reaction, Ready},
        Permissions,
    },
    prelude::{Client, Context, EventHandler, GatewayIntents},
};

fn create_commands(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
    command
        .name("role")
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
async fn create_test_guild_commands(ctx: &Context) -> serenity::Result<Command> {
    let guild_id = GuildId(
        env::var("DEBUG_GUILD_ID")
            .expect("Expected DEBUG_GUILD_ID to be set for testing")
            .parse()
            .expect("DEBUG_GUILD_ID must be an integer"),
    );

    guild_id
        .create_application_command(&ctx, create_commands)
        .await
}

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn interaction_create(&self, _ctx: Context, interaction: Interaction) {
        if let Interaction::ApplicationCommand(_command) = interaction {
            todo!("Command responses here");
        }
    }

    async fn ready(&self, ctx: Context, _ready: Ready) {
        // Discord's SLA for updating global commands is 1 hour
        // For better iteration, debug builds update a provided debug guild directly.
        #[cfg(debug_assertions)]
        let result = create_test_guild_commands(&ctx).await;

        #[cfg(not(debug_assertions))]
        let result = Command::create_global_application_command(&ctx, create_commands).await;

        if let Err(e) = result {
            error!("Failed to create app command: {}", e)
        }
    }

    // TODO: add a user to a role when they react to the corresponding emoji
    async fn reaction_add(&self, _ctx: Context, _add_reaction: Reaction) {}

    // TODO: remove a user from a role when they remove a given reaction
    async fn reaction_remove(&self, _ctx: Context, _removed_reaction: Reaction) {}
}

#[tokio::main]
async fn main() {
    env_logger::init();

    let token = env::var("DISCORD_BOT_TOKEN")
        .expect("Expected DISCORD_BOT_TOKEN environment variable to be set");

    let mut client = Client::builder(token, GatewayIntents::empty())
        .event_handler(Handler)
        .await
        .expect("Could not start bot");

    if let Err(why) = client.start().await {
        error!("Bot client error: {:?}", why);
    }
}
