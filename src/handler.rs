use std::sync::RwLock;

use log::{error, warn};
use pickledb::PickleDb;
use serenity::{
    async_trait,
    futures::TryFutureExt,
    model::prelude::{interaction::Interaction, Reaction, Ready},
    prelude::{Context, EventHandler},
};

use crate::{
    commands::create_test_guild_commands,
    database::get_guild_data,
    role_management::{create_message, disable_role, enable_role},
};

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
        if let Interaction::ApplicationCommand(command) = interaction {
            // We only define one sub-command group under role so we don't inspect the first option
            match command
                .data
                .options
                .first()
                .and_then(|sub| sub.options.first())
            {
                Some(opt) if opt.name == "enable" => {
                    enable_role(&ctx, &self.db, &command, opt).await
                }
                Some(opt) if opt.name == "disable" => {
                    disable_role(&ctx, &self.db, &command, opt).await
                }
                Some(opt) if opt.name == "message" => {
                    create_message(&ctx, &self.db, &command, opt).await
                }
                _ => warn!("A command was invoked with unexpected arguments"),
            }
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

    async fn reaction_add(&self, ctx: Context, add_reaction: Reaction) {
        let bot_user = ctx.http.get_current_user().await.map(|user| user.id);
        if let (Some(guild_id), Some(user_id), Ok(bot_id)) =
            (add_reaction.guild_id, add_reaction.user_id, bot_user)
        {
            if user_id != bot_id {
                if let Some(role_id) = get_guild_data(&self.db, &guild_id)
                    .and_then(|data| data.get_role(&add_reaction.emoji).cloned())
                {
                    if let Err(e) = guild_id
                        .member(ctx.clone(), user_id)
                        .and_then(|mut member| async move { member.add_role(&ctx, role_id).await })
                        .await
                    {
                        error!("Could not add role to user {:?}: {:?}", user_id, e)
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
                if let Some(role_id) = get_guild_data(&self.db, &guild_id)
                    .and_then(|data| data.get_role(&removed_reaction.emoji).cloned())
                {
                    if let Err(e) = guild_id
                        .member(ctx.clone(), user_id)
                        .and_then(
                            |mut member| async move { member.remove_role(&ctx, role_id).await },
                        )
                        .await
                    {
                        error!("Could not remove role from user {:?}: {:?}", user_id, e)
                    }
                }
            }
        }
    }
}
