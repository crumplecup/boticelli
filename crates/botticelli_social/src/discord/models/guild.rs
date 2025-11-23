//! Guild (Discord server) models.

use chrono::NaiveDateTime;
use diesel::prelude::*;

/// Database row for discord_guilds table.
///
/// Represents a Discord guild (server) with all metadata, settings, and bot-specific state.
#[derive(Debug, Clone, Queryable, Identifiable, Selectable)]
#[diesel(table_name = botticelli_database::schema::discord_guilds)]
#[diesel(check_for_backend(diesel::pg::Pg))]
#[allow(dead_code)] // Model fields used for database operations
pub struct GuildRow {
    id: i64,
    name: String,
    icon: Option<String>,
    banner: Option<String>,
    splash: Option<String>,
    owner_id: i64,

    // Guild features
    features: Option<Vec<Option<String>>>,
    description: Option<String>,
    vanity_url_code: Option<String>,

    // Member counts
    member_count: Option<i32>,
    approximate_member_count: Option<i32>,
    approximate_presence_count: Option<i32>,

    // Guild settings
    afk_channel_id: Option<i64>,
    afk_timeout: Option<i32>,
    system_channel_id: Option<i64>,
    rules_channel_id: Option<i64>,
    public_updates_channel_id: Option<i64>,

    // Verification and content filtering
    verification_level: Option<i16>,
    explicit_content_filter: Option<i16>,
    mfa_level: Option<i16>,

    // Premium features
    premium_tier: Option<i16>,
    premium_subscription_count: Option<i32>,

    // Server boost progress
    max_presences: Option<i32>,
    max_members: Option<i32>,
    max_video_channel_users: Option<i32>,

    // Status flags
    large: Option<bool>,
    unavailable: Option<bool>,

    // Timestamps
    joined_at: Option<NaiveDateTime>,
    created_at: NaiveDateTime,
    updated_at: NaiveDateTime,
    left_at: Option<NaiveDateTime>,

    // Bot-specific metadata
    bot_permissions: Option<i64>,
    bot_active: Option<bool>,
}

/// Insertable struct for discord_guilds table.
///
/// Used to create new guild records in the database.
#[derive(Debug, Clone, Insertable, derive_getters::Getters)]
#[diesel(table_name = botticelli_database::schema::discord_guilds)]
pub struct NewGuild {
    pub(crate) id: i64,
    pub(crate) name: String,
    pub(crate) icon: Option<String>,
    pub(crate) banner: Option<String>,
    pub(crate) splash: Option<String>,
    pub(crate) owner_id: i64,

    // Guild features
    pub(crate) features: Option<Vec<Option<String>>>,
    pub(crate) description: Option<String>,
    pub(crate) vanity_url_code: Option<String>,

    // Member counts
    pub(crate) member_count: Option<i32>,
    pub(crate) approximate_member_count: Option<i32>,
    pub(crate) approximate_presence_count: Option<i32>,

    // Guild settings
    pub(crate) afk_channel_id: Option<i64>,
    pub(crate) afk_timeout: Option<i32>,
    pub(crate) system_channel_id: Option<i64>,
    pub(crate) rules_channel_id: Option<i64>,
    pub(crate) public_updates_channel_id: Option<i64>,

    // Verification and content filtering
    pub(crate) verification_level: Option<i16>,
    pub(crate) explicit_content_filter: Option<i16>,
    pub(crate) mfa_level: Option<i16>,

    // Premium features
    pub(crate) premium_tier: Option<i16>,
    pub(crate) premium_subscription_count: Option<i32>,

    // Server boost progress
    pub(crate) max_presences: Option<i32>,
    pub(crate) max_members: Option<i32>,
    pub(crate) max_video_channel_users: Option<i32>,

    // Status flags
    pub(crate) large: Option<bool>,
    pub(crate) unavailable: Option<bool>,

    // Timestamps
    pub(crate) joined_at: Option<NaiveDateTime>,
    pub(crate) left_at: Option<NaiveDateTime>,

    // Bot-specific metadata
    pub(crate) bot_permissions: Option<i64>,
    pub(crate) bot_active: Option<bool>,
}
