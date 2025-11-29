erik@rose ~/r/botticelli (curry) [1]> just check-features
./scripts/feature-gate-check.sh
=== Feature Gate Testing ===

Testing: no-default-features
warning: field `registered_at` is never read
--> crates/botticelli_actor/src/server.rs:164:5
|
163 | pub(crate) struct ActorState {
| ---------- field in this struct
164 | registered_at: chrono::DateTime<chrono::Utc>,
| ^^^^^^^^^^^^^
|
= note: `ActorState` has derived impls for the traits `Clone` and `Debug`, but these are intentionally ignored during dead code analysis
= note: `#[warn(dead_code)]` (part of `#[warn(unused)]`) on by default

warning: method `registered_at` is never used
--> crates/botticelli_actor/src/server.rs:169:19
|
167 | impl ActorState {
| --------------- method in this implementation
168 | /// Get the registration timestamp
169 | pub(crate) fn registered_at(&self) -> &chrono::DateTime<chrono::Utc> {
| ^^^^^^^^^^^^^

warning: field `state` is never read
--> crates/botticelli_actor/src/skills/rate_limiting.rs:11:5
|
9 | pub struct RateLimitingSkill {
| ----------------- field in this struct
10 | name: String,
11 | state: HashMap<String, usize>,
| ^^^^^

warning: method `state_mut` is never used
--> crates/botticelli_actor/src/skills/rate_limiting.rs:16:8
|
14 | impl RateLimitingSkill {
| ---------------------- method in this implementation
15 | /// Get mutable reference to rate limiting state
16 | fn state_mut(&mut self) -> &mut HashMap<String, usize> {
| ^^^^^^^^^

warning: `botticelli_actor` (lib) generated 4 warnings
Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.34s
✓ no-default-features passed

Testing: default-features
warning: field `registered_at` is never read
--> crates/botticelli_actor/src/server.rs:164:5
|
163 | pub(crate) struct ActorState {
| ---------- field in this struct
164 | registered_at: chrono::DateTime<chrono::Utc>,
| ^^^^^^^^^^^^^
|
= note: `ActorState` has derived impls for the traits `Clone` and `Debug`, but these are intentionally ignored during dead code analysis
= note: `#[warn(dead_code)]` (part of `#[warn(unused)]`) on by default

warning: method `registered_at` is never used
--> crates/botticelli_actor/src/server.rs:169:19
|
167 | impl ActorState {
| --------------- method in this implementation
168 | /// Get the registration timestamp
169 | pub(crate) fn registered_at(&self) -> &chrono::DateTime<chrono::Utc> {
| ^^^^^^^^^^^^^

warning: field `state` is never read
--> crates/botticelli_actor/src/skills/rate_limiting.rs:11:5
|
9 | pub struct RateLimitingSkill {
| ----------------- field in this struct
10 | name: String,
11 | state: HashMap<String, usize>,
| ^^^^^

warning: method `state_mut` is never used
--> crates/botticelli_actor/src/skills/rate_limiting.rs:16:8
|
14 | impl RateLimitingSkill {
| ---------------------- method in this implementation
15 | /// Get mutable reference to rate limiting state
16 | fn state_mut(&mut self) -> &mut HashMap<String, usize> {
| ^^^^^^^^^

warning: `botticelli_actor` (lib) generated 4 warnings
Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.29s
✓ default-features passed

Testing: all-features
warning: multiple fields are never read
--> crates/botticelli_social/src/discord/models/guild.rs:21:5
|
12 | pub struct GuildRow {
| -------- fields in this struct
...
21 | features: Option<Vec<Option<String>>>,
| ^^^^^^^^
22 | description: Option<String>,
| ^^^^^^^^^^^
23 | vanity_url_code: Option<String>,
| ^^^^^^^^^^^^^^^
...
26 | member_count: Option<i32>,
| ^^^^^^^^^^^^
27 | approximate_member_count: Option<i32>,
| ^^^^^^^^^^^^^^^^^^^^^^^^
28 | approximate_presence_count: Option<i32>,
| ^^^^^^^^^^^^^^^^^^^^^^^^^^
...
31 | afk_channel_id: Option<i64>,
| ^^^^^^^^^^^^^^
32 | afk_timeout: Option<i32>,
| ^^^^^^^^^^^
33 | system_channel_id: Option<i64>,
| ^^^^^^^^^^^^^^^^^
34 | rules_channel_id: Option<i64>,
| ^^^^^^^^^^^^^^^^
35 | public_updates_channel_id: Option<i64>,
| ^^^^^^^^^^^^^^^^^^^^^^^^^
...
38 | verification_level: Option<i16>,
| ^^^^^^^^^^^^^^^^^^
39 | explicit_content_filter: Option<i16>,
| ^^^^^^^^^^^^^^^^^^^^^^^
40 | mfa_level: Option<i16>,
| ^^^^^^^^^
...
43 | premium_tier: Option<i16>,
| ^^^^^^^^^^^^
44 | premium_subscription_count: Option<i32>,
| ^^^^^^^^^^^^^^^^^^^^^^^^^^
...
47 | max_presences: Option<i32>,
| ^^^^^^^^^^^^^
48 | max_members: Option<i32>,
| ^^^^^^^^^^^
49 | max_video_channel_users: Option<i32>,
| ^^^^^^^^^^^^^^^^^^^^^^^
...
52 | large: Option<bool>,
| ^^^^^
53 | unavailable: Option<bool>,
| ^^^^^^^^^^^
...
56 | joined_at: Option<NaiveDateTime>,
| ^^^^^^^^^
57 | created_at: NaiveDateTime,
| ^^^^^^^^^^
58 | updated_at: NaiveDateTime,
| ^^^^^^^^^^
59 | left_at: Option<NaiveDateTime>,
| ^^^^^^^
...
62 | bot_permissions: Option<i64>,
| ^^^^^^^^^^^^^^^
63 | bot_active: Option<bool>,
| ^^^^^^^^^^
|
= note: `GuildRow` has derived impls for the traits `Clone` and `Debug`, but these are intentionally ignored during dead code analysis
= note: `#[warn(dead_code)]` (part of `#[warn(unused)]`) on by default

warning: multiple fields are never read
--> crates/botticelli_social/src/discord/models/member.rs:21:5
|
16 | pub struct GuildMemberRow {
| -------------- fields in this struct
...
21 | nick: Option<String>,
| ^^^^
22 | avatar: Option<String>, // Guild-specific avatar
| ^^^^^^
...
25 | joined_at: NaiveDateTime,
| ^^^^^^^^^
26 | premium_since: Option<NaiveDateTime>, // Server boost date
| ^^^^^^^^^^^^^
27 | communication_disabled_until: Option<NaiveDateTime>, // Timeout
| ^^^^^^^^^^^^^^^^^^^^^^^^^^^^
...
30 | deaf: Option<bool>,
| ^^^^
31 | mute: Option<bool>,
| ^^^^
32 | pending: Option<bool>, // Passed membership screening
| ^^^^^^^
...
35 | created_at: NaiveDateTime,
| ^^^^^^^^^^
36 | updated_at: NaiveDateTime,
| ^^^^^^^^^^
37 | left_at: Option<NaiveDateTime>,
| ^^^^^^^
|
= note: `GuildMemberRow` has derived impls for the traits `Clone` and `Debug`, but these are intentionally ignored during dead code analysis

warning: multiple fields are never read
--> crates/botticelli_social/src/discord/models/user.rs:22:5
|
12 | pub struct UserRow {
| ------- fields in this struct
...
22 | bot: Option<bool>,
| ^^^
23 | system: Option<bool>,
| ^^^^^^
24 | mfa_enabled: Option<bool>,
| ^^^^^^^^^^^
25 | verified: Option<bool>,
| ^^^^^^^^
...
28 | premium_type: Option<i16>,
| ^^^^^^^^^^^^
29 | public_flags: Option<i32>,
| ^^^^^^^^^^^^
...
32 | locale: Option<String>,
| ^^^^^^
...
35 | first_seen: NaiveDateTime,
| ^^^^^^^^^^
36 | last_seen: NaiveDateTime,
| ^^^^^^^^^
37 | created_at: NaiveDateTime,
| ^^^^^^^^^^
38 | updated_at: NaiveDateTime,
| ^^^^^^^^^^
|
= note: `UserRow` has derived impls for the traits `Clone` and `Debug`, but these are intentionally ignored during dead code analysis

warning: missing documentation for a struct field
--> crates/botticelli_social/src/discord/models/guild.rs:13:5
|
13 | pub id: i64,
| ^^^^^^^^^^^
|
note: the lint level is defined here
--> crates/botticelli_social/src/lib.rs:28:9
|
28 | #![warn(missing_docs)]
| ^^^^^^^^^^^^

warning: missing documentation for a struct field
--> crates/botticelli_social/src/discord/models/guild.rs:14:5
|
14 | pub name: String,
| ^^^^^^^^^^^^^^^^

warning: missing documentation for a struct field
--> crates/botticelli_social/src/discord/models/guild.rs:15:5
|
15 | pub icon: Option<String>,
| ^^^^^^^^^^^^^^^^^^^^^^^^

warning: missing documentation for a struct field
--> crates/botticelli_social/src/discord/models/guild.rs:16:5
|
16 | pub banner: Option<String>,
| ^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: missing documentation for a struct field
--> crates/botticelli_social/src/discord/models/guild.rs:17:5
|
17 | pub splash: Option<String>,
| ^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: missing documentation for a struct field
--> crates/botticelli_social/src/discord/models/guild.rs:18:5
|
18 | pub owner_id: i64,
| ^^^^^^^^^^^^^^^^^

warning: missing documentation for a struct field
--> crates/botticelli_social/src/discord/models/member.rs:17:5
|
17 | pub guild_id: i64,
| ^^^^^^^^^^^^^^^^^

warning: missing documentation for a struct field
--> crates/botticelli_social/src/discord/models/member.rs:18:5
|
18 | pub user_id: i64,
| ^^^^^^^^^^^^^^^^

warning: missing documentation for a struct field
--> crates/botticelli_social/src/discord/models/user.rs:13:5
|
13 | pub id: i64,
| ^^^^^^^^^^^

warning: missing documentation for a struct field
--> crates/botticelli_social/src/discord/models/user.rs:14:5
|
14 | pub username: String,
| ^^^^^^^^^^^^^^^^^^^^

warning: missing documentation for a struct field
--> crates/botticelli_social/src/discord/models/user.rs:15:5
|
15 | pub discriminator: Option<String>, // Legacy discriminator
| ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: missing documentation for a struct field
--> crates/botticelli_social/src/discord/models/user.rs:16:5
|
16 | pub global_name: Option<String>, // Display name
| ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: missing documentation for a struct field
--> crates/botticelli_social/src/discord/models/user.rs:17:5
|
17 | pub avatar: Option<String>,
| ^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: missing documentation for a struct field
--> crates/botticelli_social/src/discord/models/user.rs:18:5
|
18 | pub banner: Option<String>,
| ^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: missing documentation for a struct field
--> crates/botticelli_social/src/discord/models/user.rs:19:5
|
19 | pub accent_color: Option<i32>,
| ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: `botticelli_social` (lib) generated 18 warnings
warning: field `token` is never read
--> crates/botticelli_actor/src/platforms/discord.rs:22:5
|
20 | pub struct DiscordPlatform {
| --------------- field in this struct
21 | /// Discord bot token for authentication.
22 | token: String,
| ^^^^^
|
= note: `#[warn(dead_code)]` (part of `#[warn(unused)]`) on by default

warning: field `registered_at` is never read
--> crates/botticelli_actor/src/server.rs:164:5
|
163 | pub(crate) struct ActorState {
| ---------- field in this struct
164 | registered_at: chrono::DateTime<chrono::Utc>,
| ^^^^^^^^^^^^^
|
= note: `ActorState` has derived impls for the traits `Clone` and `Debug`, but these are intentionally ignored during dead code analysis

warning: method `registered_at` is never used
--> crates/botticelli_actor/src/server.rs:169:19
|
167 | impl ActorState {
| --------------- method in this implementation
168 | /// Get the registration timestamp
169 | pub(crate) fn registered_at(&self) -> &chrono::DateTime<chrono::Utc> {
| ^^^^^^^^^^^^^

warning: field `state` is never read
--> crates/botticelli_actor/src/skills/rate_limiting.rs:11:5
|
9 | pub struct RateLimitingSkill {
| ----------------- field in this struct
10 | name: String,
11 | state: HashMap<String, usize>,
| ^^^^^

warning: method `state_mut` is never used
--> crates/botticelli_actor/src/skills/rate_limiting.rs:16:8
|
14 | impl RateLimitingSkill {
| ---------------------- method in this implementation
15 | /// Get mutable reference to rate limiting state
16 | fn state_mut(&mut self) -> &mut HashMap<String, usize> {
| ^^^^^^^^^

warning: `botticelli_actor` (lib) generated 5 warnings
Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.27s
✓ all-features passed

Testing individual features...
Testing: gemini only
warning: field `registered_at` is never read
--> crates/botticelli_actor/src/server.rs:164:5
|
163 | pub(crate) struct ActorState {
| ---------- field in this struct
164 | registered_at: chrono::DateTime<chrono::Utc>,
| ^^^^^^^^^^^^^
|
= note: `ActorState` has derived impls for the traits `Clone` and `Debug`, but these are intentionally ignored during dead code analysis
= note: `#[warn(dead_code)]` (part of `#[warn(unused)]`) on by default

warning: method `registered_at` is never used
--> crates/botticelli_actor/src/server.rs:169:19
|
167 | impl ActorState {
| --------------- method in this implementation
168 | /// Get the registration timestamp
169 | pub(crate) fn registered_at(&self) -> &chrono::DateTime<chrono::Utc> {
| ^^^^^^^^^^^^^

warning: field `state` is never read
--> crates/botticelli_actor/src/skills/rate_limiting.rs:11:5
|
9 | pub struct RateLimitingSkill {
| ----------------- field in this struct
10 | name: String,
11 | state: HashMap<String, usize>,
| ^^^^^

warning: method `state_mut` is never used
--> crates/botticelli_actor/src/skills/rate_limiting.rs:16:8
|
14 | impl RateLimitingSkill {
| ---------------------- method in this implementation
15 | /// Get mutable reference to rate limiting state
16 | fn state_mut(&mut self) -> &mut HashMap<String, usize> {
| ^^^^^^^^^

warning: `botticelli_actor` (lib) generated 4 warnings
Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.26s
✓ gemini only passed

Testing: database only
warning: field `registered_at` is never read
--> crates/botticelli_actor/src/server.rs:164:5
|
163 | pub(crate) struct ActorState {
| ---------- field in this struct
164 | registered_at: chrono::DateTime<chrono::Utc>,
| ^^^^^^^^^^^^^
|
= note: `ActorState` has derived impls for the traits `Clone` and `Debug`, but these are intentionally ignored during dead code analysis
= note: `#[warn(dead_code)]` (part of `#[warn(unused)]`) on by default

warning: method `registered_at` is never used
--> crates/botticelli_actor/src/server.rs:169:19
|
167 | impl ActorState {
| --------------- method in this implementation
168 | /// Get the registration timestamp
169 | pub(crate) fn registered_at(&self) -> &chrono::DateTime<chrono::Utc> {
| ^^^^^^^^^^^^^

warning: field `state` is never read
--> crates/botticelli_actor/src/skills/rate_limiting.rs:11:5
|
9 | pub struct RateLimitingSkill {
| ----------------- field in this struct
10 | name: String,
11 | state: HashMap<String, usize>,
| ^^^^^

warning: method `state_mut` is never used
--> crates/botticelli_actor/src/skills/rate_limiting.rs:16:8
|
14 | impl RateLimitingSkill {
| ---------------------- method in this implementation
15 | /// Get mutable reference to rate limiting state
16 | fn state_mut(&mut self) -> &mut HashMap<String, usize> {
| ^^^^^^^^^

warning: `botticelli_actor` (lib) generated 4 warnings
Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.27s
✓ database only passed

Testing: discord only
warning: multiple fields are never read
--> crates/botticelli_social/src/discord/models/guild.rs:21:5
|
12 | pub struct GuildRow {
| -------- fields in this struct
...
21 | features: Option<Vec<Option<String>>>,
| ^^^^^^^^
22 | description: Option<String>,
| ^^^^^^^^^^^
23 | vanity_url_code: Option<String>,
| ^^^^^^^^^^^^^^^
...
26 | member_count: Option<i32>,
| ^^^^^^^^^^^^
27 | approximate_member_count: Option<i32>,
| ^^^^^^^^^^^^^^^^^^^^^^^^
28 | approximate_presence_count: Option<i32>,
| ^^^^^^^^^^^^^^^^^^^^^^^^^^
...
31 | afk_channel_id: Option<i64>,
| ^^^^^^^^^^^^^^
32 | afk_timeout: Option<i32>,
| ^^^^^^^^^^^
33 | system_channel_id: Option<i64>,
| ^^^^^^^^^^^^^^^^^
34 | rules_channel_id: Option<i64>,
| ^^^^^^^^^^^^^^^^
35 | public_updates_channel_id: Option<i64>,
| ^^^^^^^^^^^^^^^^^^^^^^^^^
...
38 | verification_level: Option<i16>,
| ^^^^^^^^^^^^^^^^^^
39 | explicit_content_filter: Option<i16>,
| ^^^^^^^^^^^^^^^^^^^^^^^
40 | mfa_level: Option<i16>,
| ^^^^^^^^^
...
43 | premium_tier: Option<i16>,
| ^^^^^^^^^^^^
44 | premium_subscription_count: Option<i32>,
| ^^^^^^^^^^^^^^^^^^^^^^^^^^
...
47 | max_presences: Option<i32>,
| ^^^^^^^^^^^^^
48 | max_members: Option<i32>,
| ^^^^^^^^^^^
49 | max_video_channel_users: Option<i32>,
| ^^^^^^^^^^^^^^^^^^^^^^^
...
52 | large: Option<bool>,
| ^^^^^
53 | unavailable: Option<bool>,
| ^^^^^^^^^^^
...
56 | joined_at: Option<NaiveDateTime>,
| ^^^^^^^^^
57 | created_at: NaiveDateTime,
| ^^^^^^^^^^
58 | updated_at: NaiveDateTime,
| ^^^^^^^^^^
59 | left_at: Option<NaiveDateTime>,
| ^^^^^^^
...
62 | bot_permissions: Option<i64>,
| ^^^^^^^^^^^^^^^
63 | bot_active: Option<bool>,
| ^^^^^^^^^^
|
= note: `GuildRow` has derived impls for the traits `Clone` and `Debug`, but these are intentionally ignored during dead code analysis
= note: `#[warn(dead_code)]` (part of `#[warn(unused)]`) on by default

warning: multiple fields are never read
--> crates/botticelli_social/src/discord/models/member.rs:21:5
|
16 | pub struct GuildMemberRow {
| -------------- fields in this struct
...
21 | nick: Option<String>,
| ^^^^
22 | avatar: Option<String>, // Guild-specific avatar
| ^^^^^^
...
25 | joined_at: NaiveDateTime,
| ^^^^^^^^^
26 | premium_since: Option<NaiveDateTime>, // Server boost date
| ^^^^^^^^^^^^^
27 | communication_disabled_until: Option<NaiveDateTime>, // Timeout
| ^^^^^^^^^^^^^^^^^^^^^^^^^^^^
...
30 | deaf: Option<bool>,
| ^^^^
31 | mute: Option<bool>,
| ^^^^
32 | pending: Option<bool>, // Passed membership screening
| ^^^^^^^
...
35 | created_at: NaiveDateTime,
| ^^^^^^^^^^
36 | updated_at: NaiveDateTime,
| ^^^^^^^^^^
37 | left_at: Option<NaiveDateTime>,
| ^^^^^^^
|
= note: `GuildMemberRow` has derived impls for the traits `Clone` and `Debug`, but these are intentionally ignored during dead code analysis

warning: multiple fields are never read
--> crates/botticelli_social/src/discord/models/user.rs:22:5
|
12 | pub struct UserRow {
| ------- fields in this struct
...
22 | bot: Option<bool>,
| ^^^
23 | system: Option<bool>,
| ^^^^^^
24 | mfa_enabled: Option<bool>,
| ^^^^^^^^^^^
25 | verified: Option<bool>,
| ^^^^^^^^
...
28 | premium_type: Option<i16>,
| ^^^^^^^^^^^^
29 | public_flags: Option<i32>,
| ^^^^^^^^^^^^
...
32 | locale: Option<String>,
| ^^^^^^
...
35 | first_seen: NaiveDateTime,
| ^^^^^^^^^^
36 | last_seen: NaiveDateTime,
| ^^^^^^^^^
37 | created_at: NaiveDateTime,
| ^^^^^^^^^^
38 | updated_at: NaiveDateTime,
| ^^^^^^^^^^
|
= note: `UserRow` has derived impls for the traits `Clone` and `Debug`, but these are intentionally ignored during dead code analysis

warning: missing documentation for a struct field
--> crates/botticelli_social/src/discord/models/guild.rs:13:5
|
13 | pub id: i64,
| ^^^^^^^^^^^
|
note: the lint level is defined here
--> crates/botticelli_social/src/lib.rs:28:9
|
28 | #![warn(missing_docs)]
| ^^^^^^^^^^^^

warning: missing documentation for a struct field
--> crates/botticelli_social/src/discord/models/guild.rs:14:5
|
14 | pub name: String,
| ^^^^^^^^^^^^^^^^

warning: missing documentation for a struct field
--> crates/botticelli_social/src/discord/models/guild.rs:15:5
|
15 | pub icon: Option<String>,
| ^^^^^^^^^^^^^^^^^^^^^^^^

warning: missing documentation for a struct field
--> crates/botticelli_social/src/discord/models/guild.rs:16:5
|
16 | pub banner: Option<String>,
| ^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: missing documentation for a struct field
--> crates/botticelli_social/src/discord/models/guild.rs:17:5
|
17 | pub splash: Option<String>,
| ^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: missing documentation for a struct field
--> crates/botticelli_social/src/discord/models/guild.rs:18:5
|
18 | pub owner_id: i64,
| ^^^^^^^^^^^^^^^^^

warning: missing documentation for a struct field
--> crates/botticelli_social/src/discord/models/member.rs:17:5
|
17 | pub guild_id: i64,
| ^^^^^^^^^^^^^^^^^

warning: missing documentation for a struct field
--> crates/botticelli_social/src/discord/models/member.rs:18:5
|
18 | pub user_id: i64,
| ^^^^^^^^^^^^^^^^

warning: missing documentation for a struct field
--> crates/botticelli_social/src/discord/models/user.rs:13:5
|
13 | pub id: i64,
| ^^^^^^^^^^^

warning: missing documentation for a struct field
--> crates/botticelli_social/src/discord/models/user.rs:14:5
|
14 | pub username: String,
| ^^^^^^^^^^^^^^^^^^^^

warning: missing documentation for a struct field
--> crates/botticelli_social/src/discord/models/user.rs:15:5
|
15 | pub discriminator: Option<String>, // Legacy discriminator
| ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: missing documentation for a struct field
--> crates/botticelli_social/src/discord/models/user.rs:16:5
|
16 | pub global_name: Option<String>, // Display name
| ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: missing documentation for a struct field
--> crates/botticelli_social/src/discord/models/user.rs:17:5
|
17 | pub avatar: Option<String>,
| ^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: missing documentation for a struct field
--> crates/botticelli_social/src/discord/models/user.rs:18:5
|
18 | pub banner: Option<String>,
| ^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: missing documentation for a struct field
--> crates/botticelli_social/src/discord/models/user.rs:19:5
|
19 | pub accent_color: Option<i32>,
| ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: `botticelli_social` (lib) generated 18 warnings
warning: field `token` is never read
--> crates/botticelli_actor/src/platforms/discord.rs:22:5
|
20 | pub struct DiscordPlatform {
| --------------- field in this struct
21 | /// Discord bot token for authentication.
22 | token: String,
| ^^^^^
|
= note: `#[warn(dead_code)]` (part of `#[warn(unused)]`) on by default

warning: field `registered_at` is never read
--> crates/botticelli_actor/src/server.rs:164:5
|
163 | pub(crate) struct ActorState {
| ---------- field in this struct
164 | registered_at: chrono::DateTime<chrono::Utc>,
| ^^^^^^^^^^^^^
|
= note: `ActorState` has derived impls for the traits `Clone` and `Debug`, but these are intentionally ignored during dead code analysis

warning: method `registered_at` is never used
--> crates/botticelli_actor/src/server.rs:169:19
|
167 | impl ActorState {
| --------------- method in this implementation
168 | /// Get the registration timestamp
169 | pub(crate) fn registered_at(&self) -> &chrono::DateTime<chrono::Utc> {
| ^^^^^^^^^^^^^

warning: field `state` is never read
--> crates/botticelli_actor/src/skills/rate_limiting.rs:11:5
|
9 | pub struct RateLimitingSkill {
| ----------------- field in this struct
10 | name: String,
11 | state: HashMap<String, usize>,
| ^^^^^

warning: method `state_mut` is never used
--> crates/botticelli_actor/src/skills/rate_limiting.rs:16:8
|
14 | impl RateLimitingSkill {
| ---------------------- method in this implementation
15 | /// Get mutable reference to rate limiting state
16 | fn state_mut(&mut self) -> &mut HashMap<String, usize> {
| ^^^^^^^^^

warning: `botticelli_actor` (lib) generated 5 warnings
Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.27s
✓ discord only passed

Testing: tui only
warning: field `registered_at` is never read
--> crates/botticelli_actor/src/server.rs:164:5
|
163 | pub(crate) struct ActorState {
| ---------- field in this struct
164 | registered_at: chrono::DateTime<chrono::Utc>,
| ^^^^^^^^^^^^^
|
= note: `ActorState` has derived impls for the traits `Clone` and `Debug`, but these are intentionally ignored during dead code analysis
= note: `#[warn(dead_code)]` (part of `#[warn(unused)]`) on by default

warning: method `registered_at` is never used
--> crates/botticelli_actor/src/server.rs:169:19
|
167 | impl ActorState {
| --------------- method in this implementation
168 | /// Get the registration timestamp
169 | pub(crate) fn registered_at(&self) -> &chrono::DateTime<chrono::Utc> {
| ^^^^^^^^^^^^^

warning: field `state` is never read
--> crates/botticelli_actor/src/skills/rate_limiting.rs:11:5
|
9 | pub struct RateLimitingSkill {
| ----------------- field in this struct
10 | name: String,
11 | state: HashMap<String, usize>,
| ^^^^^

warning: method `state_mut` is never used
--> crates/botticelli_actor/src/skills/rate_limiting.rs:16:8
|
14 | impl RateLimitingSkill {
| ---------------------- method in this implementation
15 | /// Get mutable reference to rate limiting state
16 | fn state_mut(&mut self) -> &mut HashMap<String, usize> {
| ^^^^^^^^^

warning: `botticelli_actor` (lib) generated 4 warnings
Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.28s
✓ tui only passed

Testing feature combinations...
Testing: gemini + database
warning: field `registered_at` is never read
--> crates/botticelli_actor/src/server.rs:164:5
|
163 | pub(crate) struct ActorState {
| ---------- field in this struct
164 | registered_at: chrono::DateTime<chrono::Utc>,
| ^^^^^^^^^^^^^
|
= note: `ActorState` has derived impls for the traits `Clone` and `Debug`, but these are intentionally ignored during dead code analysis
= note: `#[warn(dead_code)]` (part of `#[warn(unused)]`) on by default

warning: method `registered_at` is never used
--> crates/botticelli_actor/src/server.rs:169:19
|
167 | impl ActorState {
| --------------- method in this implementation
168 | /// Get the registration timestamp
169 | pub(crate) fn registered_at(&self) -> &chrono::DateTime<chrono::Utc> {
| ^^^^^^^^^^^^^

warning: field `state` is never read
--> crates/botticelli_actor/src/skills/rate_limiting.rs:11:5
|
9 | pub struct RateLimitingSkill {
| ----------------- field in this struct
10 | name: String,
11 | state: HashMap<String, usize>,
| ^^^^^

warning: method `state_mut` is never used
--> crates/botticelli_actor/src/skills/rate_limiting.rs:16:8
|
14 | impl RateLimitingSkill {
| ---------------------- method in this implementation
15 | /// Get mutable reference to rate limiting state
16 | fn state_mut(&mut self) -> &mut HashMap<String, usize> {
| ^^^^^^^^^

warning: `botticelli_actor` (lib) generated 4 warnings
Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.26s
✓ gemini + database passed

Testing: gemini + discord
warning: multiple fields are never read
--> crates/botticelli_social/src/discord/models/guild.rs:21:5
|
12 | pub struct GuildRow {
| -------- fields in this struct
...
21 | features: Option<Vec<Option<String>>>,
| ^^^^^^^^
22 | description: Option<String>,
| ^^^^^^^^^^^
23 | vanity_url_code: Option<String>,
| ^^^^^^^^^^^^^^^
...
26 | member_count: Option<i32>,
| ^^^^^^^^^^^^
27 | approximate_member_count: Option<i32>,
| ^^^^^^^^^^^^^^^^^^^^^^^^
28 | approximate_presence_count: Option<i32>,
| ^^^^^^^^^^^^^^^^^^^^^^^^^^
...
31 | afk_channel_id: Option<i64>,
| ^^^^^^^^^^^^^^
32 | afk_timeout: Option<i32>,
| ^^^^^^^^^^^
33 | system_channel_id: Option<i64>,
| ^^^^^^^^^^^^^^^^^
34 | rules_channel_id: Option<i64>,
| ^^^^^^^^^^^^^^^^
35 | public_updates_channel_id: Option<i64>,
| ^^^^^^^^^^^^^^^^^^^^^^^^^
...
38 | verification_level: Option<i16>,
| ^^^^^^^^^^^^^^^^^^
39 | explicit_content_filter: Option<i16>,
| ^^^^^^^^^^^^^^^^^^^^^^^
40 | mfa_level: Option<i16>,
| ^^^^^^^^^
...
43 | premium_tier: Option<i16>,
| ^^^^^^^^^^^^
44 | premium_subscription_count: Option<i32>,
| ^^^^^^^^^^^^^^^^^^^^^^^^^^
...
47 | max_presences: Option<i32>,
| ^^^^^^^^^^^^^
48 | max_members: Option<i32>,
| ^^^^^^^^^^^
49 | max_video_channel_users: Option<i32>,
| ^^^^^^^^^^^^^^^^^^^^^^^
...
52 | large: Option<bool>,
| ^^^^^
53 | unavailable: Option<bool>,
| ^^^^^^^^^^^
...
56 | joined_at: Option<NaiveDateTime>,
| ^^^^^^^^^
57 | created_at: NaiveDateTime,
| ^^^^^^^^^^
58 | updated_at: NaiveDateTime,
| ^^^^^^^^^^
59 | left_at: Option<NaiveDateTime>,
| ^^^^^^^
...
62 | bot_permissions: Option<i64>,
| ^^^^^^^^^^^^^^^
63 | bot_active: Option<bool>,
| ^^^^^^^^^^
|
= note: `GuildRow` has derived impls for the traits `Clone` and `Debug`, but these are intentionally ignored during dead code analysis
= note: `#[warn(dead_code)]` (part of `#[warn(unused)]`) on by default

warning: multiple fields are never read
--> crates/botticelli_social/src/discord/models/member.rs:21:5
|
16 | pub struct GuildMemberRow {
| -------------- fields in this struct
...
21 | nick: Option<String>,
| ^^^^
22 | avatar: Option<String>, // Guild-specific avatar
| ^^^^^^
...
25 | joined_at: NaiveDateTime,
| ^^^^^^^^^
26 | premium_since: Option<NaiveDateTime>, // Server boost date
| ^^^^^^^^^^^^^
27 | communication_disabled_until: Option<NaiveDateTime>, // Timeout
| ^^^^^^^^^^^^^^^^^^^^^^^^^^^^
...
30 | deaf: Option<bool>,
| ^^^^
31 | mute: Option<bool>,
| ^^^^
32 | pending: Option<bool>, // Passed membership screening
| ^^^^^^^
...
35 | created_at: NaiveDateTime,
| ^^^^^^^^^^
36 | updated_at: NaiveDateTime,
| ^^^^^^^^^^
37 | left_at: Option<NaiveDateTime>,
| ^^^^^^^
|
= note: `GuildMemberRow` has derived impls for the traits `Clone` and `Debug`, but these are intentionally ignored during dead code analysis

warning: multiple fields are never read
--> crates/botticelli_social/src/discord/models/user.rs:22:5
|
12 | pub struct UserRow {
| ------- fields in this struct
...
22 | bot: Option<bool>,
| ^^^
23 | system: Option<bool>,
| ^^^^^^
24 | mfa_enabled: Option<bool>,
| ^^^^^^^^^^^
25 | verified: Option<bool>,
| ^^^^^^^^
...
28 | premium_type: Option<i16>,
| ^^^^^^^^^^^^
29 | public_flags: Option<i32>,
| ^^^^^^^^^^^^
...
32 | locale: Option<String>,
| ^^^^^^
...
35 | first_seen: NaiveDateTime,
| ^^^^^^^^^^
36 | last_seen: NaiveDateTime,
| ^^^^^^^^^
37 | created_at: NaiveDateTime,
| ^^^^^^^^^^
38 | updated_at: NaiveDateTime,
| ^^^^^^^^^^
|
= note: `UserRow` has derived impls for the traits `Clone` and `Debug`, but these are intentionally ignored during dead code analysis

warning: missing documentation for a struct field
--> crates/botticelli_social/src/discord/models/guild.rs:13:5
|
13 | pub id: i64,
| ^^^^^^^^^^^
|
note: the lint level is defined here
--> crates/botticelli_social/src/lib.rs:28:9
|
28 | #![warn(missing_docs)]
| ^^^^^^^^^^^^

warning: missing documentation for a struct field
--> crates/botticelli_social/src/discord/models/guild.rs:14:5
|
14 | pub name: String,
| ^^^^^^^^^^^^^^^^

warning: missing documentation for a struct field
--> crates/botticelli_social/src/discord/models/guild.rs:15:5
|
15 | pub icon: Option<String>,
| ^^^^^^^^^^^^^^^^^^^^^^^^

warning: missing documentation for a struct field
--> crates/botticelli_social/src/discord/models/guild.rs:16:5
|
16 | pub banner: Option<String>,
| ^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: missing documentation for a struct field
--> crates/botticelli_social/src/discord/models/guild.rs:17:5
|
17 | pub splash: Option<String>,
| ^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: missing documentation for a struct field
--> crates/botticelli_social/src/discord/models/guild.rs:18:5
|
18 | pub owner_id: i64,
| ^^^^^^^^^^^^^^^^^

warning: missing documentation for a struct field
--> crates/botticelli_social/src/discord/models/member.rs:17:5
|
17 | pub guild_id: i64,
| ^^^^^^^^^^^^^^^^^

warning: missing documentation for a struct field
--> crates/botticelli_social/src/discord/models/member.rs:18:5
|
18 | pub user_id: i64,
| ^^^^^^^^^^^^^^^^

warning: missing documentation for a struct field
--> crates/botticelli_social/src/discord/models/user.rs:13:5
|
13 | pub id: i64,
| ^^^^^^^^^^^

warning: missing documentation for a struct field
--> crates/botticelli_social/src/discord/models/user.rs:14:5
|
14 | pub username: String,
| ^^^^^^^^^^^^^^^^^^^^

warning: missing documentation for a struct field
--> crates/botticelli_social/src/discord/models/user.rs:15:5
|
15 | pub discriminator: Option<String>, // Legacy discriminator
| ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: missing documentation for a struct field
--> crates/botticelli_social/src/discord/models/user.rs:16:5
|
16 | pub global_name: Option<String>, // Display name
| ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: missing documentation for a struct field
--> crates/botticelli_social/src/discord/models/user.rs:17:5
|
17 | pub avatar: Option<String>,
| ^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: missing documentation for a struct field
--> crates/botticelli_social/src/discord/models/user.rs:18:5
|
18 | pub banner: Option<String>,
| ^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: missing documentation for a struct field
--> crates/botticelli_social/src/discord/models/user.rs:19:5
|
19 | pub accent_color: Option<i32>,
| ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: `botticelli_social` (lib) generated 18 warnings
warning: field `token` is never read
--> crates/botticelli_actor/src/platforms/discord.rs:22:5
|
20 | pub struct DiscordPlatform {
| --------------- field in this struct
21 | /// Discord bot token for authentication.
22 | token: String,
| ^^^^^
|
= note: `#[warn(dead_code)]` (part of `#[warn(unused)]`) on by default

warning: field `registered_at` is never read
--> crates/botticelli_actor/src/server.rs:164:5
|
163 | pub(crate) struct ActorState {
| ---------- field in this struct
164 | registered_at: chrono::DateTime<chrono::Utc>,
| ^^^^^^^^^^^^^
|
= note: `ActorState` has derived impls for the traits `Clone` and `Debug`, but these are intentionally ignored during dead code analysis

warning: method `registered_at` is never used
--> crates/botticelli_actor/src/server.rs:169:19
|
167 | impl ActorState {
| --------------- method in this implementation
168 | /// Get the registration timestamp
169 | pub(crate) fn registered_at(&self) -> &chrono::DateTime<chrono::Utc> {
| ^^^^^^^^^^^^^

warning: field `state` is never read
--> crates/botticelli_actor/src/skills/rate_limiting.rs:11:5
|
9 | pub struct RateLimitingSkill {
| ----------------- field in this struct
10 | name: String,
11 | state: HashMap<String, usize>,
| ^^^^^

warning: method `state_mut` is never used
--> crates/botticelli_actor/src/skills/rate_limiting.rs:16:8
|
14 | impl RateLimitingSkill {
| ---------------------- method in this implementation
15 | /// Get mutable reference to rate limiting state
16 | fn state_mut(&mut self) -> &mut HashMap<String, usize> {
| ^^^^^^^^^

warning: `botticelli_actor` (lib) generated 5 warnings
Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.28s
✓ gemini + discord passed

Testing: database + tui
warning: field `registered_at` is never read
--> crates/botticelli_actor/src/server.rs:164:5
|
163 | pub(crate) struct ActorState {
| ---------- field in this struct
164 | registered_at: chrono::DateTime<chrono::Utc>,
| ^^^^^^^^^^^^^
|
= note: `ActorState` has derived impls for the traits `Clone` and `Debug`, but these are intentionally ignored during dead code analysis
= note: `#[warn(dead_code)]` (part of `#[warn(unused)]`) on by default

warning: method `registered_at` is never used
--> crates/botticelli_actor/src/server.rs:169:19
|
167 | impl ActorState {
| --------------- method in this implementation
168 | /// Get the registration timestamp
169 | pub(crate) fn registered_at(&self) -> &chrono::DateTime<chrono::Utc> {
| ^^^^^^^^^^^^^

warning: field `state` is never read
--> crates/botticelli_actor/src/skills/rate_limiting.rs:11:5
|
9 | pub struct RateLimitingSkill {
| ----------------- field in this struct
10 | name: String,
11 | state: HashMap<String, usize>,
| ^^^^^

warning: method `state_mut` is never used
--> crates/botticelli_actor/src/skills/rate_limiting.rs:16:8
|
14 | impl RateLimitingSkill {
| ---------------------- method in this implementation
15 | /// Get mutable reference to rate limiting state
16 | fn state_mut(&mut self) -> &mut HashMap<String, usize> {
| ^^^^^^^^^

warning: `botticelli_actor` (lib) generated 4 warnings
Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.30s
✓ database + tui passed

Running clippy checks...
Testing: clippy no-default-features
Checking botticelli*narrative v0.2.0 (/home/erik/repos/botticelli/crates/botticelli_narrative)
error: empty lines after doc comment
--> crates/botticelli_narrative/src/storage_actor.rs:542:1
|
542 | / /// Formats schema for LLM prompts (reserved for Phase 2 improved prompts)
543 | |
544 | |
| |*^
545 | /// Converts a JSON value to SQL literal based on column type with best-effort coercion
546 | fn json_value_to_sql(value: &JsonValue, col_type: &str) -> String {
| -------------------- the comment documents this function
|
= help: for further information visit https://rust-lang.github.io/rust-clippy/rust-1.91.0/index.html#empty_line_after_doc_comments
= note: `-D clippy::empty-line-after-doc-comments` implied by `-D warnings`
= help: to override `-D warnings` add `#[allow(clippy::empty_line_after_doc_comments)]`
= help: if the empty lines are unintentional, remove them
help: if the documentation should include the empty lines include them in the comment
|
543 + ///
544 + ///
|

error: could not compile `botticelli_narrative` (lib) due to 1 previous error
✗ clippy no-default-features failed
error: Recipe `check-features` failed on line 254 with exit code 1
