use std::sync::RwLock;

use log::{error, warn};
use pickledb::PickleDb;
use serenity::{
    async_trait,
    client::{Context, EventHandler},
    futures::TryFutureExt,
    model::{
        application::{CommandDataOptionValue, Interaction},
        channel::Reaction,
        gateway::Ready,
    },
};

#[cfg(not(debug_assertions))]
use crate::commands::create;
#[cfg(debug_assertions)]
use crate::commands::create_for_test_guild;
use crate::{
    database::get_guild_data,
    role_management::{create_message, disable_role, enable_role},
};
#[cfg(not(debug_assertions))]
use serenity::model::application::Command;

pub struct Handler {
    db: RwLock<PickleDb>,
}

impl Handler {
    pub fn new(db: PickleDb) -> Self {
        Self {
            db: RwLock::new(db),
        }
    }
}

#[async_trait]
impl EventHandler for Handler {
    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        if let Interaction::Command(command) = interaction {
            match command
                .data
                .options
                .first()
                .and_then(|opt| match &opt.value {
                    CommandDataOptionValue::SubCommandGroup(group)
                        if command.data.name == "role" && opt.name == "self-service" =>
                    {
                        group.first()
                    }
                    _ => None,
                }) {
                Some(opt) if opt.name == "enable" => {
                    enable_role(&ctx, &self.db, &command, opt).await;
                }
                Some(opt) if opt.name == "disable" => {
                    disable_role(&ctx, &self.db, &command, opt).await;
                }
                Some(opt) if opt.name == "message" => {
                    create_message(&ctx, &self.db, &command, opt).await;
                }
                _ => warn!("A command was invoked with unexpected arguments, Discord should have prevented this"),
            }
        }
    }

    async fn ready(&self, ctx: Context, _ready: Ready) {
        // Discord's SLA for updating global commands is 1 hour
        // For better iteration, debug builds update a provided debug guild directly.
        #[cfg(debug_assertions)]
        let result = create_for_test_guild(&ctx).await;

        #[cfg(not(debug_assertions))]
        let result = Command::create_global_command(&ctx, create()).await;

        if let Err(e) = result {
            error!("Failed to create app command: {}", e);
        }
    }

    async fn reaction_add(&self, ctx: Context, add_reaction: Reaction) {
        let bot_user = ctx.http.get_current_user().await.map(|user| user.id);
        if let (Some(guild_id), Some(user_id), Ok(bot_id)) =
            (add_reaction.guild_id, add_reaction.user_id, bot_user)
        {
            if user_id != bot_id {
                if let Some(role_id) = get_guild_data(&self.db, guild_id)
                    .filter(|data| {
                        data.get_message_id()
                            .is_some_and(|message| add_reaction.message_id == message)
                    })
                    .and_then(|data| data.get_role(&add_reaction.emoji).copied())
                {
                    if let Err(e) = guild_id
                        .member(ctx.clone(), user_id)
                        .and_then(|member| async move { member.add_role(&ctx, role_id).await })
                        .await
                    {
                        error!("Could not add role to user {:?}: {:?}", user_id, e);
                    }
                }
            }
        }
    }

    async fn reaction_remove(&self, ctx: Context, removed_reaction: Reaction) {
        let bot_user = ctx.http.get_current_user().await.map(|user| user.id);
        if let (Some(guild_id), Some(user_id), Ok(bot_id)) = (
            removed_reaction.guild_id,
            removed_reaction.user_id,
            bot_user,
        ) {
            if user_id != bot_id {
                if let Some(role_id) = get_guild_data(&self.db, guild_id)
                    .filter(|data| {
                        data.get_message_id()
                            .is_some_and(|message| removed_reaction.message_id == message)
                    })
                    .and_then(|data| data.get_role(&removed_reaction.emoji).copied())
                {
                    if let Err(e) = guild_id
                        .member(ctx.clone(), user_id)
                        .and_then(|member| async move { member.remove_role(&ctx, role_id).await })
                        .await
                    {
                        error!("Could not remove role from user {:?}: {:?}", user_id, e);
                    }
                }
            }
        }
    }
}
