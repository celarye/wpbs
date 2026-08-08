/* SPDX-License-Identifier: GPL-3.0-or-later */
/* Copyright © 2026 Eduard Smet */

use std::collections::HashSet;

use serde::Deserialize;
use twilight_gateway::Intents;

#[derive(Default, Deserialize)]
pub struct ConfigDiscord {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default)]
    pub settings: ConfigDiscordSettings,
}

#[derive(Default, Deserialize)]
pub struct ConfigDiscordSettings {
    #[serde(default)]
    pub intents: HashSet<ConfigDiscordIntents>,
}

#[derive(Deserialize, Eq, Hash, PartialEq)]
pub enum ConfigDiscordIntents {
    All,
    AllNonPrivileged,
    Guilds,
    GuildMembers,
    GuildModeration,
    GuildEmojisAndStickers,
    GuildIntegrations,
    GuildWebhooks,
    GuildInvites,
    GuildVoiceStates,
    GuildPresences,
    GuildMessages,
    GuildMessageReactions,
    GuildMessageTyping,
    DirectMessages,
    DirectMessageReactions,
    DirectMessageTyping,
    MessageContent,
    GuildScheduledEvents,
    AutoModerationConfiguration,
    AutoModerationExecution,
    GuildMessagePolls,
    DirectMessagePolls,
}

pub struct InternalIntents(pub Intents);

impl From<HashSet<ConfigDiscordIntents>> for InternalIntents {
    fn from(values: HashSet<ConfigDiscordIntents>) -> Self {
        if values.contains(&ConfigDiscordIntents::All) {
            return Self(Intents::all());
        }

        let mut result = Intents::empty();

        for value in values {
            result |= match value {
                ConfigDiscordIntents::All => unreachable!(),
                ConfigDiscordIntents::AllNonPrivileged => {
                    !(Intents::GUILD_MEMBERS | Intents::GUILD_PRESENCES | Intents::MESSAGE_CONTENT)
                }
                ConfigDiscordIntents::Guilds => Intents::GUILDS,
                ConfigDiscordIntents::GuildMembers => Intents::GUILD_MEMBERS,
                ConfigDiscordIntents::GuildModeration => Intents::GUILD_MODERATION,
                ConfigDiscordIntents::GuildEmojisAndStickers => Intents::GUILD_EMOJIS_AND_STICKERS,
                ConfigDiscordIntents::GuildIntegrations => Intents::GUILD_INTEGRATIONS,
                ConfigDiscordIntents::GuildWebhooks => Intents::GUILD_WEBHOOKS,
                ConfigDiscordIntents::GuildInvites => Intents::GUILD_INVITES,
                ConfigDiscordIntents::GuildVoiceStates => Intents::GUILD_VOICE_STATES,
                ConfigDiscordIntents::GuildPresences => Intents::GUILD_PRESENCES,
                ConfigDiscordIntents::GuildMessages => Intents::GUILD_MESSAGES,
                ConfigDiscordIntents::GuildMessageReactions => Intents::GUILD_MESSAGE_REACTIONS,
                ConfigDiscordIntents::GuildMessageTyping => Intents::GUILD_MESSAGE_TYPING,
                ConfigDiscordIntents::DirectMessages => Intents::DIRECT_MESSAGES,
                ConfigDiscordIntents::DirectMessageReactions => Intents::DIRECT_MESSAGE_REACTIONS,
                ConfigDiscordIntents::DirectMessageTyping => Intents::DIRECT_MESSAGE_TYPING,
                ConfigDiscordIntents::MessageContent => Intents::MESSAGE_CONTENT,
                ConfigDiscordIntents::GuildScheduledEvents => Intents::GUILD_SCHEDULED_EVENTS,
                ConfigDiscordIntents::AutoModerationConfiguration => {
                    Intents::AUTO_MODERATION_CONFIGURATION
                }
                ConfigDiscordIntents::AutoModerationExecution => Intents::AUTO_MODERATION_EXECUTION,
                ConfigDiscordIntents::GuildMessagePolls => Intents::GUILD_MESSAGE_POLLS,
                ConfigDiscordIntents::DirectMessagePolls => Intents::DIRECT_MESSAGE_POLLS,
            };
        }

        Self(result)
    }
}
