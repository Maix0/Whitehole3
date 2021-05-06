use serenity::{
    client::{Context, EventHandler},
    model::prelude::Ready,
};

pub struct WhEventHandlerManager {
    inners: Vec<Box<dyn EventHandler>>,
}

impl WhEventHandlerManager {
    pub fn new() -> Self {
        Self { inners: Vec::new() }
    }

    pub fn push(&mut self, handler: impl EventHandler + 'static) {
        self.inners.push(Box::new(handler));
    }
}

#[serenity::async_trait]
impl EventHandler for WhEventHandlerManager {
    async fn ready(&self, ctx: Context, bot: Ready) {
        for handler in &self.inners {
            handler.ready(ctx.clone(), bot.clone()).await;
        }
    }

    async fn cache_ready(&self, _ctx: Context, _guilds: Vec<serenity::model::id::GuildId>) {
        for handler in &self.inners {
            handler.cache_ready(_ctx.clone(), _guilds.clone()).await;
        }
    }

    async fn channel_create(
        &self,
        _ctx: Context,
        _channel: &serenity::model::channel::GuildChannel,
    ) {
        for handler in &self.inners {
            handler.channel_create(_ctx.clone(), _channel).await;
        }
    }

    async fn category_create(
        &self,
        _ctx: Context,
        _category: &serenity::model::channel::ChannelCategory,
    ) {
        for handler in &self.inners {
            handler.category_create(_ctx.clone(), _category).await;
        }
    }

    async fn category_delete(
        &self,
        _ctx: Context,
        _category: &serenity::model::channel::ChannelCategory,
    ) {
        for handler in &self.inners {
            handler.category_delete(_ctx.clone(), _category).await;
        }
    }

    async fn channel_delete(
        &self,
        _ctx: Context,
        _channel: &serenity::model::channel::GuildChannel,
    ) {
        for handler in &self.inners {
            handler.channel_delete(_ctx.clone(), _channel).await;
        }
    }

    async fn channel_pins_update(
        &self,
        _ctx: Context,
        _pin: serenity::model::event::ChannelPinsUpdateEvent,
    ) {
        for handler in &self.inners {
            handler
                .channel_pins_update(_ctx.clone(), _pin.clone())
                .await;
        }
    }

    async fn channel_update(
        &self,
        _ctx: Context,
        _old: Option<serenity::model::channel::Channel>,
        _new: serenity::model::channel::Channel,
    ) {
        for handler in &self.inners {
            handler
                .channel_update(_ctx.clone(), _old.clone(), _new.clone())
                .await;
        }
    }

    async fn guild_ban_addition(
        &self,
        _ctx: Context,
        _guild_id: serenity::model::id::GuildId,
        _banned_user: serenity::model::prelude::User,
    ) {
        for handler in &self.inners {
            handler
                .guild_ban_addition(_ctx.clone(), _guild_id.clone(), _banned_user.clone())
                .await;
        }
    }

    async fn guild_ban_removal(
        &self,
        _ctx: Context,
        _guild_id: serenity::model::id::GuildId,
        _unbanned_user: serenity::model::prelude::User,
    ) {
        for handler in &self.inners {
            handler
                .guild_ban_removal(_ctx.clone(), _guild_id.clone(), _unbanned_user.clone())
                .await;
        }
    }

    async fn guild_create(
        &self,
        _ctx: Context,
        _guild: serenity::model::guild::Guild,
        _is_new: bool,
    ) {
        for handler in &self.inners {
            handler
                .guild_create(_ctx.clone(), _guild.clone(), _is_new.clone())
                .await;
        }
    }

    async fn guild_delete(
        &self,
        _ctx: Context,
        _incomplete: serenity::model::guild::GuildUnavailable,
        _full: Option<serenity::model::guild::Guild>,
    ) {
        for handler in &self.inners {
            handler
                .guild_delete(_ctx.clone(), _incomplete.clone(), _full.clone())
                .await;
        }
    }

    async fn guild_emojis_update(
        &self,
        _ctx: Context,
        _guild_id: serenity::model::id::GuildId,
        _current_state: std::collections::HashMap<
            serenity::model::id::EmojiId,
            serenity::model::guild::Emoji,
        >,
    ) {
        for handler in &self.inners {
            handler
                .guild_emojis_update(_ctx.clone(), _guild_id.clone(), _current_state.clone())
                .await;
        }
    }

    async fn guild_integrations_update(
        &self,
        _ctx: Context,
        _guild_id: serenity::model::id::GuildId,
    ) {
        for handler in &self.inners {
            handler
                .guild_integrations_update(_ctx.clone(), _guild_id.clone())
                .await;
        }
    }

    async fn guild_member_addition(
        &self,
        _ctx: Context,
        _guild_id: serenity::model::id::GuildId,
        _new_member: serenity::model::guild::Member,
    ) {
        for handler in &self.inners {
            handler
                .guild_member_addition(_ctx.clone(), _guild_id.clone(), _new_member.clone())
                .await;
        }
    }

    async fn guild_member_removal(
        &self,
        _ctx: Context,
        _guild_id: serenity::model::id::GuildId,
        _user: serenity::model::prelude::User,
        _member_data_if_available: Option<serenity::model::guild::Member>,
    ) {
        for handler in &self.inners {
            handler
                .guild_member_removal(
                    _ctx.clone(),
                    _guild_id.clone(),
                    _user.clone(),
                    _member_data_if_available.clone(),
                )
                .await;
        }
    }

    async fn guild_member_update(
        &self,
        _ctx: Context,
        _old_if_available: Option<serenity::model::guild::Member>,
        _new: serenity::model::guild::Member,
    ) {
        for handler in &self.inners {
            handler
                .guild_member_update(_ctx.clone(), _old_if_available.clone(), _new.clone())
                .await;
        }
    }

    async fn guild_members_chunk(
        &self,
        _ctx: Context,
        _chunk: serenity::model::event::GuildMembersChunkEvent,
    ) {
        for handler in &self.inners {
            handler
                .guild_members_chunk(_ctx.clone(), _chunk.clone())
                .await;
        }
    }

    async fn guild_role_create(
        &self,
        _ctx: Context,
        _guild_id: serenity::model::id::GuildId,
        _new: serenity::model::guild::Role,
    ) {
        for handler in &self.inners {
            handler
                .guild_role_create(_ctx.clone(), _guild_id.clone(), _new.clone())
                .await;
        }
    }

    async fn guild_role_delete(
        &self,
        _ctx: Context,
        _guild_id: serenity::model::id::GuildId,
        _removed_role_id: serenity::model::id::RoleId,
        _removed_role_data_if_available: Option<serenity::model::guild::Role>,
    ) {
        for handler in &self.inners {
            handler
                .guild_role_delete(
                    _ctx.clone(),
                    _guild_id.clone(),
                    _removed_role_id.clone(),
                    _removed_role_data_if_available.clone(),
                )
                .await;
        }
    }

    async fn guild_role_update(
        &self,
        _ctx: Context,
        _guild_id: serenity::model::id::GuildId,
        _old_data_if_available: Option<serenity::model::guild::Role>,
        _new: serenity::model::guild::Role,
    ) {
        for handler in &self.inners {
            handler
                .guild_role_update(
                    _ctx.clone(),
                    _guild_id.clone(),
                    _old_data_if_available.clone(),
                    _new.clone(),
                )
                .await;
        }
    }

    async fn guild_unavailable(&self, _ctx: Context, _guild_id: serenity::model::id::GuildId) {
        for handler in &self.inners {
            handler
                .guild_unavailable(_ctx.clone(), _guild_id.clone())
                .await;
        }
    }

    async fn guild_update(
        &self,
        _ctx: Context,
        _old_data_if_available: Option<serenity::model::guild::Guild>,
        _new_but_incomplete: serenity::model::guild::PartialGuild,
    ) {
        for handler in &self.inners {
            handler
                .guild_update(
                    _ctx.clone(),
                    _old_data_if_available.clone(),
                    _new_but_incomplete.clone(),
                )
                .await;
        }
    }

    async fn invite_create(&self, _ctx: Context, _data: serenity::model::event::InviteCreateEvent) {
        for handler in &self.inners {
            handler.invite_create(_ctx.clone(), _data.clone()).await;
        }
    }

    async fn invite_delete(&self, _ctx: Context, _data: serenity::model::event::InviteDeleteEvent) {
        for handler in &self.inners {
            handler.invite_delete(_ctx.clone(), _data.clone()).await;
        }
    }

    async fn message(&self, _ctx: Context, _new_message: serenity::model::channel::Message) {
        for handler in &self.inners {
            handler.message(_ctx.clone(), _new_message.clone()).await;
        }
    }

    async fn message_delete(
        &self,
        _ctx: Context,
        _channel_id: serenity::model::id::ChannelId,
        _deleted_message_id: serenity::model::id::MessageId,
        _guild_id: Option<serenity::model::id::GuildId>,
    ) {
        for handler in &self.inners {
            handler
                .message_delete(
                    _ctx.clone(),
                    _channel_id.clone(),
                    _deleted_message_id.clone(),
                    _guild_id.clone(),
                )
                .await;
        }
    }

    async fn message_delete_bulk(
        &self,
        _ctx: Context,
        _channel_id: serenity::model::id::ChannelId,
        _multiple_deleted_messages_ids: Vec<serenity::model::id::MessageId>,
        _guild_id: Option<serenity::model::id::GuildId>,
    ) {
        for handler in &self.inners {
            handler
                .message_delete_bulk(
                    _ctx.clone(),
                    _channel_id.clone(),
                    _multiple_deleted_messages_ids.clone(),
                    _guild_id.clone(),
                )
                .await;
        }
    }

    async fn message_update(
        &self,
        _ctx: Context,
        _old_if_available: Option<serenity::model::channel::Message>,
        _new: Option<serenity::model::channel::Message>,
        _event: serenity::model::event::MessageUpdateEvent,
    ) {
        for handler in &self.inners {
            handler
                .message_update(
                    _ctx.clone(),
                    _old_if_available.clone(),
                    _new.clone(),
                    _event.clone(),
                )
                .await;
        }
    }

    async fn reaction_add(&self, _ctx: Context, _add_reaction: serenity::model::channel::Reaction) {
        for handler in &self.inners {
            handler
                .reaction_add(_ctx.clone(), _add_reaction.clone())
                .await;
        }
    }

    async fn reaction_remove(
        &self,
        _ctx: Context,
        _removed_reaction: serenity::model::channel::Reaction,
    ) {
        for handler in &self.inners {
            handler
                .reaction_remove(_ctx.clone(), _removed_reaction.clone())
                .await;
        }
    }

    async fn reaction_remove_all(
        &self,
        _ctx: Context,
        _channel_id: serenity::model::id::ChannelId,
        _removed_from_message_id: serenity::model::id::MessageId,
    ) {
        for handler in &self.inners {
            handler
                .reaction_remove_all(
                    _ctx.clone(),
                    _channel_id.clone(),
                    _removed_from_message_id.clone(),
                )
                .await;
        }
    }

    async fn presence_replace(&self, _ctx: Context, _v: Vec<serenity::model::prelude::Presence>) {
        for handler in &self.inners {
            handler.presence_replace(_ctx.clone(), _v.clone()).await;
        }
    }

    async fn presence_update(
        &self,
        _ctx: Context,
        _new_data: serenity::model::event::PresenceUpdateEvent,
    ) {
        for handler in &self.inners {
            handler
                .presence_update(_ctx.clone(), _new_data.clone())
                .await;
        }
    }

    async fn resume(&self, _ctx: Context, _r: serenity::model::event::ResumedEvent) {
        for handler in &self.inners {
            handler.resume(_ctx.clone(), _r.clone()).await;
        }
    }

    async fn shard_stage_update(
        &self,
        _ctx: Context,
        _s: serenity::client::bridge::gateway::event::ShardStageUpdateEvent,
    ) {
        for handler in &self.inners {
            handler.shard_stage_update(_ctx.clone(), _s.clone()).await;
        }
    }

    async fn typing_start(&self, _ctx: Context, _t: serenity::model::event::TypingStartEvent) {
        for handler in &self.inners {
            handler.typing_start(_ctx.clone(), _t.clone()).await;
        }
    }

    async fn user_update(
        &self,
        _ctx: Context,
        _old_data: serenity::model::prelude::CurrentUser,
        _new: serenity::model::prelude::CurrentUser,
    ) {
        for handler in &self.inners {
            handler
                .user_update(_ctx.clone(), _old_data.clone(), _new.clone())
                .await;
        }
    }

    async fn voice_server_update(
        &self,
        _ctx: Context,
        _v: serenity::model::event::VoiceServerUpdateEvent,
    ) {
        for handler in &self.inners {
            handler.voice_server_update(_ctx.clone(), _v.clone()).await;
        }
    }

    async fn voice_state_update(
        &self,
        _ctx: Context,
        _g: Option<serenity::model::id::GuildId>,
        _old: Option<serenity::model::prelude::VoiceState>,
        _new: serenity::model::prelude::VoiceState,
    ) {
        for handler in &self.inners {
            handler
                .voice_state_update(_ctx.clone(), _g.clone(), _old.clone(), _new.clone())
                .await;
        }
    }

    async fn webhook_update(
        &self,
        _ctx: Context,
        _guild_id: serenity::model::id::GuildId,
        _belongs_to_channel_id: serenity::model::id::ChannelId,
    ) {
        for handler in &self.inners {
            handler
                .webhook_update(
                    _ctx.clone(),
                    _guild_id.clone(),
                    _belongs_to_channel_id.clone(),
                )
                .await;
        }
    }
}
