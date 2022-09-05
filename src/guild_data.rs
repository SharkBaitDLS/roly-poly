use std::fmt::Write;

use bimap::BiMap;
use log::error;
use serde::{Deserialize, Serialize};
use serenity::{
    futures::TryFutureExt,
    model::prelude::{ChannelId, MessageId, ReactionType},
    prelude::Context,
    utils::Color,
};

#[derive(Serialize, Deserialize)]
pub struct GuildData {
    channel_id: Option<ChannelId>,
    message_id: Option<MessageId>,
    roles_to_emoji: BiMap<u64, ReactionType>,
}

impl GuildData {
    pub fn new(roles_to_emoji: BiMap<u64, ReactionType>) -> Self {
        Self {
            channel_id: None,
            message_id: None,
            roles_to_emoji,
        }
    }

    pub async fn send_message(&mut self, ctx: &Context, channel_id: ChannelId) -> &Self {
        if !self.message_exists(ctx, channel_id).await {
            let message_id = channel_id
                .send_message(ctx, |msg| {
                    msg.add_embed(|embed| {
                        embed.color(Color::DARKER_GREY).field(
                            "Self-Assignable Roles",
                            self.generate_message(),
                            true,
                        )
                    })
                    .reactions(self.roles_to_emoji.right_values().cloned())
                })
                .await
                .map(|msg| msg.id);

            if message_id.is_err() {
                error!("Could not send message: {:?}", message_id);
            } else {
                self.channel_id = Some(channel_id);
            }

            self.message_id = message_id.ok();
        }
        self
    }

    pub async fn add_role(&mut self, ctx: &Context, role_id: u64, emoji: ReactionType) {
        self.roles_to_emoji.insert(role_id, emoji.clone());
        self.update_message(ctx, Some(emoji), false).await;
    }

    pub async fn remove_role(&mut self, ctx: &Context, role_id: &u64) {
        let emoji = self
            .roles_to_emoji
            .remove_by_left(role_id)
            .map(|(_, emoji)| emoji);
        self.update_message(ctx, emoji, true).await;
    }

    pub fn get_role(&self, emoji: &ReactionType) -> Option<&u64> {
        self.roles_to_emoji.get_by_right(emoji)
    }

    async fn update_message(&self, ctx: &Context, maybe_emoji: Option<ReactionType>, remove: bool) {
        if let (Some(channel_id), Some(message_id)) = (self.channel_id, self.message_id) {
            if let Err(e) = channel_id
                .edit_message(ctx, message_id, |msg| {
                    let message = self.generate_message();

                    if message.is_empty() {
                        msg.set_embeds(Vec::new())
                            .content("No configured roles to display")
                    } else {
                        msg.embed(|embed| {
                            embed.color(Color::DARKER_GREY).field(
                                "Self-Assignable Roles",
                                message,
                                true,
                            )
                        })
                        .content("")
                    }
                })
                .await
            {
                error!(
                    "Could not edit message for channel {:?}: {:?}",
                    self.channel_id, e
                );
                return;
            }

            match maybe_emoji {
                Some(emoji) if remove => {
                    if let Err(e) = channel_id
                        .message(ctx, message_id)
                        .and_then(|message| async move {
                            message.delete_reaction_emoji(ctx, emoji).await
                        })
                        .await
                    {
                        error!(
                            "Could not remove reactions to message for channel {:?}: {:?}",
                            self.channel_id, e
                        );
                    }
                }
                Some(emoji) => {
                    if let Err(e) = channel_id.create_reaction(ctx, message_id, emoji).await {
                        error!(
                            "Could not react to message for channel {:?}: {:?}",
                            self.channel_id, e
                        );
                    }
                }
                None => {}
            }
        }
    }

    async fn message_exists(&self, ctx: &Context, channel_id: ChannelId) -> bool {
        match self.message_id {
            Some(message_id) => ctx
                .http
                .get_message(*channel_id.as_u64(), *message_id.as_u64())
                .await
                .ok()
                .is_some(),
            None => false,
        }
    }

    fn generate_message(&self) -> String {
        let mut result = String::new();

        self.roles_to_emoji.iter().for_each(|entry| match entry.1 {
            ReactionType::Custom { animated, id, name } => {
                if *animated {
                    writeln!(
                        result,
                        "<@&{}>: <a:{}:{}>",
                        entry.0,
                        name.as_ref().expect("A named emoji"),
                        id
                    )
                    .expect("String concatenation success");
                } else {
                    writeln!(
                        result,
                        "<@&{}>: <:{}:{}>",
                        entry.0,
                        name.as_ref().expect("A named emoji"),
                        id
                    )
                    .expect("String concatenation success");
                }
            }
            ReactionType::Unicode(char) => {
                writeln!(result, "<@&{}>: {}", entry.0, char)
                    .expect("String concatenation success");
            }
            _ => todo!(),
        });

        result
    }
}
