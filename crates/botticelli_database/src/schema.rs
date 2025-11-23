// @generated automatically by Diesel CLI.

pub mod sql_types {
    #[derive(diesel::query_builder::QueryId, diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "discord_channel_type"))]
    pub struct DiscordChannelType;
}

diesel::table! {
    act_executions (id) {
        id -> Int4,
        execution_id -> Int4,
        act_name -> Text,
        sequence_number -> Int4,
        model -> Nullable<Text>,
        temperature -> Nullable<Float4>,
        max_tokens -> Nullable<Int4>,
        response -> Text,
        created_at -> Timestamp,
    }
}

diesel::table! {
    act_inputs (id) {
        id -> Int4,
        act_execution_id -> Int4,
        input_order -> Int4,
        input_type -> Text,
        text_content -> Nullable<Text>,
        mime_type -> Nullable<Text>,
        filename -> Nullable<Text>,
        created_at -> Timestamp,
        media_ref_id -> Nullable<Uuid>,
    }
}

diesel::table! {
    actor_preferences (id) {
        id -> Int4,
        #[max_length = 100]
        actor_name -> Varchar,
        min_post_interval_minutes -> Nullable<Int4>,
        max_posts_per_day -> Nullable<Int4>,
        preferred_tags -> Nullable<Array<Nullable<Text>>>,
        excluded_tags -> Nullable<Array<Nullable<Text>>>,
        time_window_start -> Nullable<Time>,
        time_window_end -> Nullable<Time>,
        #[max_length = 50]
        timezone -> Nullable<Varchar>,
        randomize_schedule -> Nullable<Bool>,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

diesel::table! {
    content (id) {
        id -> Int4,
        #[max_length = 50]
        content_type -> Varchar,
        text_content -> Nullable<Text>,
        media_urls -> Nullable<Array<Nullable<Text>>>,
        media_types -> Nullable<Array<Nullable<Varchar>>>,
        #[max_length = 255]
        source -> Nullable<Varchar>,
        priority -> Nullable<Int4>,
        tags -> Nullable<Array<Nullable<Text>>>,
        approved_at -> Nullable<Timestamp>,
        #[max_length = 100]
        approved_by -> Nullable<Varchar>,
        scheduled_for -> Nullable<Timestamp>,
        expires_at -> Nullable<Timestamp>,
        post_count -> Nullable<Int4>,
        last_posted_at -> Nullable<Timestamp>,
        created_at -> Timestamp,
        metadata -> Nullable<Jsonb>,
    }
}

diesel::table! {
    content_generation_tables (table_name) {
        table_name -> Text,
        template_source -> Text,
        created_at -> Timestamp,
        narrative_file -> Nullable<Text>,
        description -> Nullable<Text>,
    }
}

diesel::table! {
    content_generations (id) {
        id -> Int4,
        table_name -> Text,
        narrative_file -> Text,
        narrative_name -> Text,
        generated_at -> Timestamptz,
        completed_at -> Nullable<Timestamptz>,
        row_count -> Nullable<Int4>,
        generation_duration_ms -> Nullable<Int4>,
        status -> Text,
        error_message -> Nullable<Text>,
        created_by -> Nullable<Text>,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::DiscordChannelType;

    discord_channels (id) {
        id -> Int8,
        guild_id -> Nullable<Int8>,
        #[max_length = 100]
        name -> Nullable<Varchar>,
        channel_type -> DiscordChannelType,
        position -> Nullable<Int4>,
        topic -> Nullable<Text>,
        nsfw -> Nullable<Bool>,
        rate_limit_per_user -> Nullable<Int4>,
        bitrate -> Nullable<Int4>,
        user_limit -> Nullable<Int4>,
        parent_id -> Nullable<Int8>,
        owner_id -> Nullable<Int8>,
        message_count -> Nullable<Int4>,
        member_count -> Nullable<Int4>,
        archived -> Nullable<Bool>,
        auto_archive_duration -> Nullable<Int4>,
        archive_timestamp -> Nullable<Timestamp>,
        locked -> Nullable<Bool>,
        invitable -> Nullable<Bool>,
        available_tags -> Nullable<Jsonb>,
        default_reaction_emoji -> Nullable<Jsonb>,
        default_thread_rate_limit -> Nullable<Int4>,
        default_sort_order -> Nullable<Int2>,
        default_forum_layout -> Nullable<Int2>,
        created_at -> Timestamp,
        updated_at -> Timestamp,
        last_message_at -> Nullable<Timestamp>,
        last_read_message_id -> Nullable<Int8>,
        bot_has_access -> Nullable<Bool>,
    }
}

diesel::table! {
    discord_guild_members (guild_id, user_id) {
        guild_id -> Int8,
        user_id -> Int8,
        #[max_length = 32]
        nick -> Nullable<Varchar>,
        #[max_length = 255]
        avatar -> Nullable<Varchar>,
        joined_at -> Timestamp,
        premium_since -> Nullable<Timestamp>,
        communication_disabled_until -> Nullable<Timestamp>,
        deaf -> Nullable<Bool>,
        mute -> Nullable<Bool>,
        pending -> Nullable<Bool>,
        created_at -> Timestamp,
        updated_at -> Timestamp,
        left_at -> Nullable<Timestamp>,
    }
}

diesel::table! {
    discord_guilds (id) {
        id -> Int8,
        #[max_length = 100]
        name -> Varchar,
        #[max_length = 255]
        icon -> Nullable<Varchar>,
        #[max_length = 255]
        banner -> Nullable<Varchar>,
        #[max_length = 255]
        splash -> Nullable<Varchar>,
        owner_id -> Int8,
        features -> Nullable<Array<Nullable<Text>>>,
        description -> Nullable<Text>,
        #[max_length = 50]
        vanity_url_code -> Nullable<Varchar>,
        member_count -> Nullable<Int4>,
        approximate_member_count -> Nullable<Int4>,
        approximate_presence_count -> Nullable<Int4>,
        afk_channel_id -> Nullable<Int8>,
        afk_timeout -> Nullable<Int4>,
        system_channel_id -> Nullable<Int8>,
        rules_channel_id -> Nullable<Int8>,
        public_updates_channel_id -> Nullable<Int8>,
        verification_level -> Nullable<Int2>,
        explicit_content_filter -> Nullable<Int2>,
        mfa_level -> Nullable<Int2>,
        premium_tier -> Nullable<Int2>,
        premium_subscription_count -> Nullable<Int4>,
        max_presences -> Nullable<Int4>,
        max_members -> Nullable<Int4>,
        max_video_channel_users -> Nullable<Int4>,
        large -> Nullable<Bool>,
        unavailable -> Nullable<Bool>,
        joined_at -> Nullable<Timestamp>,
        created_at -> Timestamp,
        updated_at -> Timestamp,
        left_at -> Nullable<Timestamp>,
        bot_permissions -> Nullable<Int8>,
        bot_active -> Nullable<Bool>,
    }
}

diesel::table! {
    discord_member_roles (guild_id, user_id, role_id) {
        guild_id -> Int8,
        user_id -> Int8,
        role_id -> Int8,
        assigned_at -> Timestamp,
        assigned_by -> Nullable<Int8>,
    }
}

diesel::table! {
    discord_roles (id) {
        id -> Int8,
        guild_id -> Int8,
        #[max_length = 100]
        name -> Varchar,
        color -> Int4,
        hoist -> Nullable<Bool>,
        #[max_length = 255]
        icon -> Nullable<Varchar>,
        #[max_length = 100]
        unicode_emoji -> Nullable<Varchar>,
        position -> Int4,
        permissions -> Int8,
        managed -> Nullable<Bool>,
        mentionable -> Nullable<Bool>,
        tags -> Nullable<Jsonb>,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

diesel::table! {
    discord_users (id) {
        id -> Int8,
        #[max_length = 32]
        username -> Varchar,
        #[max_length = 4]
        discriminator -> Nullable<Varchar>,
        #[max_length = 32]
        global_name -> Nullable<Varchar>,
        #[max_length = 255]
        avatar -> Nullable<Varchar>,
        #[max_length = 255]
        banner -> Nullable<Varchar>,
        accent_color -> Nullable<Int4>,
        bot -> Nullable<Bool>,
        system -> Nullable<Bool>,
        mfa_enabled -> Nullable<Bool>,
        verified -> Nullable<Bool>,
        premium_type -> Nullable<Int2>,
        public_flags -> Nullable<Int4>,
        #[max_length = 10]
        locale -> Nullable<Varchar>,
        first_seen -> Timestamp,
        last_seen -> Timestamp,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

diesel::table! {
    media_references (id) {
        id -> Uuid,
        media_type -> Text,
        mime_type -> Text,
        size_bytes -> Int8,
        content_hash -> Text,
        storage_backend -> Text,
        storage_path -> Text,
        uploaded_at -> Timestamp,
        last_accessed_at -> Nullable<Timestamp>,
        access_count -> Nullable<Int4>,
        width -> Nullable<Int4>,
        height -> Nullable<Int4>,
        duration_seconds -> Nullable<Float4>,
    }
}

diesel::table! {
    model_responses (id) {
        id -> Uuid,
        created_at -> Timestamp,
        #[max_length = 50]
        provider -> Varchar,
        #[max_length = 100]
        model_name -> Varchar,
        request_messages -> Jsonb,
        request_temperature -> Nullable<Float4>,
        request_max_tokens -> Nullable<Int4>,
        #[max_length = 100]
        request_model -> Nullable<Varchar>,
        response_outputs -> Jsonb,
        duration_ms -> Nullable<Int4>,
        error_message -> Nullable<Text>,
    }
}

diesel::table! {
    narrative_executions (id) {
        id -> Int4,
        narrative_name -> Text,
        narrative_description -> Nullable<Text>,
        started_at -> Timestamp,
        completed_at -> Nullable<Timestamp>,
        status -> Text,
        error_message -> Nullable<Text>,
        created_at -> Timestamp,
    }
}

diesel::table! {
    post_history (id) {
        id -> Int4,
        content_id -> Nullable<Int4>,
        #[max_length = 100]
        actor_name -> Varchar,
        #[max_length = 50]
        platform -> Varchar,
        #[max_length = 100]
        channel_id -> Nullable<Varchar>,
        #[max_length = 255]
        post_id -> Nullable<Varchar>,
        posted_at -> Timestamp,
        engagement_count -> Nullable<Int4>,
        metadata -> Nullable<Jsonb>,
    }
}

diesel::joinable!(act_executions -> narrative_executions (execution_id));
diesel::joinable!(act_inputs -> act_executions (act_execution_id));
diesel::joinable!(act_inputs -> media_references (media_ref_id));
diesel::joinable!(discord_channels -> discord_guilds (guild_id));
diesel::joinable!(discord_guild_members -> discord_guilds (guild_id));
diesel::joinable!(discord_guild_members -> discord_users (user_id));
diesel::joinable!(discord_member_roles -> discord_roles (role_id));
diesel::joinable!(discord_roles -> discord_guilds (guild_id));
diesel::joinable!(post_history -> content (content_id));

diesel::allow_tables_to_appear_in_same_query!(
    act_executions,
    act_inputs,
    actor_preferences,
    content,
    content_generation_tables,
    content_generations,
    discord_channels,
    discord_guild_members,
    discord_guilds,
    discord_member_roles,
    discord_roles,
    discord_users,
    media_references,
    model_responses,
    narrative_executions,
    post_history,
);
