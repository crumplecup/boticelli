# Discord API Coverage Analysis

## Current Implementation (54 commands)

### Server Management (READ)
- ✅ `server.get_stats` - Get server statistics (members, description, icon, etc.)

### Channels (READ + WRITE)
- ✅ `channels.list` - List all channels in a server
- ✅ `channels.get` - Get specific channel details
- ✅ `channels.create` - Create a new channel (WRITE - secured)
- ✅ `channels.edit` - Edit channel properties (WRITE - secured)
- ✅ `channels.delete` - Delete a channel (WRITE - secured)
- ✅ `channels.create_invite` - Create invite link (WRITE - secured)
- ✅ `channels.typing` - Trigger typing indicator (WRITE - secured, low-risk)

### Roles (READ + WRITE)
- ✅ `roles.list` - List all roles in a server
- ✅ `roles.get` - Get specific role details
- ✅ `roles.create` - Create a new role (WRITE - secured)
- ✅ `roles.assign` - Assign role to member (WRITE - secured)
- ✅ `roles.remove` - Remove role from member (WRITE - secured)
- ✅ `roles.edit` - Edit role properties (WRITE - secured)
- ✅ `roles.delete` - Delete a role (WRITE - secured)

### Members (READ + WRITE)
- ✅ `members.list` - List server members
- ✅ `members.get` - Get specific member details
- ✅ `members.ban` - Ban a member (WRITE - secured)
- ✅ `members.kick` - Kick a member (WRITE - secured)
- ✅ `members.timeout` - Timeout a member (WRITE - secured)
- ✅ `members.unban` - Unban a member (WRITE - secured)
- ✅ `members.edit` - Edit member properties (WRITE - secured)
- ✅ `members.remove_timeout` - Remove timeout (WRITE - secured)

### Messages (READ + WRITE)
- ✅ `messages.get` - Get a specific message (READ)
- ✅ `messages.list` - Get channel message history (READ)
- ✅ `messages.send` - Send a message to a channel (WRITE - secured)
- ✅ `messages.edit` - Edit an existing message (WRITE - secured)
- ✅ `messages.delete` - Delete a message (WRITE - secured)
- ✅ `messages.bulk_delete` - Delete multiple messages (WRITE - secured)
- ✅ `messages.pin` - Pin a message (WRITE - secured)
- ✅ `messages.unpin` - Unpin a message (WRITE - secured)

### Reactions (READ + WRITE)
- ✅ `reactions.list` - List users who reacted with emoji (READ)
- ✅ `reactions.add` - Add reaction to message (WRITE - secured, low-risk)
- ✅ `reactions.remove` - Remove reaction from message (WRITE - secured, low-risk)
- ✅ `reactions.clear` - Clear all reactions from message (WRITE - secured)
- ✅ `reactions.clear_emoji` - Clear specific emoji reactions (WRITE - secured)

### Moderation (READ)
- ✅ `bans.list` - List banned users

### Threads (READ + WRITE)
- ✅ `threads.create` - Create a new thread (WRITE - secured)
- ✅ `threads.list` - List active threads (READ)
- ✅ `threads.get` - Get thread details (READ)
- ✅ `threads.edit` - Edit thread properties (WRITE - secured)
- ✅ `threads.delete` - Delete a thread (WRITE - secured)
- ✅ `threads.join` - Join a thread (WRITE - low-risk)
- ✅ `threads.leave` - Leave a thread (WRITE - low-risk)
- ✅ `threads.add_member` - Add member to thread (WRITE - secured)
- ✅ `threads.remove_member` - Remove member from thread (WRITE - secured)

### Server Features (READ)
- ✅ `emojis.list` - List custom emojis
- ✅ `stickers.list` - List custom stickers
- ✅ `voice_regions.list` - List available voice regions
- ✅ `events.list` - List scheduled events
- ✅ `integrations.list` - List server integrations
- ✅ `invites.list` - List server invites
- ✅ `webhooks.list` - List webhooks

## Missing Serenity API Coverage

### High Priority - Core Discord Operations

#### Messages (WRITE)
- ✅ `messages.edit` - Edit an existing message (IMPLEMENTED)
- ✅ `messages.delete` - Delete a message (IMPLEMENTED)
- ✅ `messages.get` - Get a specific message (IMPLEMENTED)
- ✅ `messages.list` - Get channel message history (IMPLEMENTED)
- ✅ `messages.pin` - Pin a message (IMPLEMENTED)
- ✅ `messages.unpin` - Unpin a message (IMPLEMENTED)
- ✅ `messages.bulk_delete` - Delete multiple messages (IMPLEMENTED)

#### Reactions (WRITE)
- ✅ `reactions.add` - Add reaction to message (IMPLEMENTED)
- ✅ `reactions.remove` - Remove reaction from message (IMPLEMENTED)

#### Channels (WRITE)
- ✅ `channels.edit` - Modify channel settings (IMPLEMENTED)
- ✅ `channels.create_invite` - Create an invite link (IMPLEMENTED)
- ✅ `channels.typing` - Trigger typing indicator (IMPLEMENTED)

#### Roles (WRITE)
- ✅ `roles.create` - Create a new role (IMPLEMENTED)
- ✅ `roles.edit` - Modify role properties (IMPLEMENTED)
- ✅ `roles.delete` - Delete a role (IMPLEMENTED)
- ✅ `roles.assign` - Assign role to member (IMPLEMENTED)
- ✅ `roles.remove` - Remove role from member (IMPLEMENTED)

#### Members (WRITE)
- ✅ `members.kick` - Kick a member (IMPLEMENTED)
- ✅ `members.unban` - Unban a member (IMPLEMENTED)
- ✅ `members.timeout` - Timeout a member (IMPLEMENTED)
- ✅ `members.edit` - Modify member (nickname, roles, mute, deafen) (IMPLEMENTED)
- ✅ `members.remove_timeout` - Remove timeout from member (IMPLEMENTED)

#### Threads
- ✅ `threads.create` - Create a thread from message or in forum (IMPLEMENTED)
- ✅ `threads.list` - List active/archived threads (IMPLEMENTED)
- ✅ `threads.get` - Get thread details (IMPLEMENTED)
- ✅ `threads.edit` - Modify thread settings (IMPLEMENTED)
- ✅ `threads.delete` - Delete a thread (IMPLEMENTED)
- ✅ `threads.join` - Join a thread (IMPLEMENTED)
- ✅ `threads.leave` - Leave a thread (IMPLEMENTED)
- ✅ `threads.add_member` - Add member to thread (IMPLEMENTED)
- ✅ `threads.remove_member` - Remove member from thread (IMPLEMENTED)

### Medium Priority - Rich Features

#### Reactions
- ✅ `reactions.add` - Add reaction to message (IMPLEMENTED)
- ✅ `reactions.remove` - Remove reaction from message (IMPLEMENTED)
- ✅ `reactions.list` - List users who reacted with emoji (IMPLEMENTED)
- ✅ `reactions.clear` - Clear all reactions from message (IMPLEMENTED)
- ✅ `reactions.clear_emoji` - Clear specific emoji reactions (IMPLEMENTED)

#### Voice Channels
- ❌ `voice.connect` - Connect bot to voice channel (requires gateway)
- ❌ `voice.disconnect` - Disconnect from voice channel

#### Stage Channels
- ❌ `stage.create_instance` - Create stage instance
- ❌ `stage.edit_instance` - Edit stage instance
- ❌ `stage.delete_instance` - Delete stage instance

#### Forum Channels
- ❌ `forum.create_post` - Create forum post
- ❌ `forum.list_posts` - List forum posts
- ❌ `forum.get_post` - Get forum post details

#### Scheduled Events
- ❌ `events.create` - Create scheduled event
- ❌ `events.edit` - Edit scheduled event
- ❌ `events.delete` - Delete scheduled event
- ❌ `events.get` - Get event details
- ❌ `events.list_users` - List users interested in event

#### Server Settings (WRITE)
- ❌ `server.edit` - Modify server settings
- ❌ `server.leave` - Leave server (bot)
- ❌ `server.get_widget` - Get server widget settings
- ❌ `server.edit_widget` - Modify server widget

#### Emojis (WRITE)
- ❌ `emojis.create` - Create custom emoji
- ❌ `emojis.edit` - Edit emoji name
- ❌ `emojis.delete` - Delete emoji

#### Stickers (WRITE)
- ❌ `stickers.create` - Create custom sticker
- ❌ `stickers.edit` - Edit sticker
- ❌ `stickers.delete` - Delete sticker

#### Webhooks (WRITE)
- ❌ `webhooks.create` - Create webhook
- ❌ `webhooks.get` - Get webhook details
- ❌ `webhooks.edit` - Edit webhook
- ❌ `webhooks.delete` - Delete webhook
- ❌ `webhooks.execute` - Send message via webhook

#### Permissions
- ❌ `permissions.get_channel` - Get channel permission overwrites
- ❌ `permissions.edit_channel` - Edit channel permissions for role/member
- ❌ `permissions.delete_channel` - Delete permission overwrite

### Low Priority - Advanced Features

#### Auto Moderation
- ❌ `automod.list_rules` - List auto-moderation rules
- ❌ `automod.get_rule` - Get specific rule
- ❌ `automod.create_rule` - Create auto-mod rule
- ❌ `automod.edit_rule` - Edit auto-mod rule
- ❌ `automod.delete_rule` - Delete auto-mod rule

#### Application Commands (Slash Commands)
- ❌ `commands.list_global` - List global application commands
- ❌ `commands.create_global` - Create global command
- ❌ `commands.edit_global` - Edit global command
- ❌ `commands.delete_global` - Delete global command
- ❌ `commands.list_guild` - List guild-specific commands
- ❌ `commands.create_guild` - Create guild command
- ❌ `commands.edit_guild` - Edit guild command
- ❌ `commands.delete_guild` - Delete guild command

#### Audit Logs
- ❌ `audit.list` - Get audit log entries (who did what when)

#### Bans (WRITE)
- ❌ `bans.get` - Get specific ban details
- ❌ `bans.create` - Ban user with reason (duplicate of members.ban?)
- ❌ `bans.delete` - Unban user

#### Invites (WRITE)
- ❌ `invites.get` - Get invite details
- ❌ `invites.delete` - Revoke invite

#### Integrations (WRITE)
- ❌ `integrations.delete` - Remove integration

#### Prune
- ❌ `prune.count` - Count members who would be pruned
- ❌ `prune.begin` - Prune inactive members

#### Templates
- ❌ `templates.list` - List server templates
- ❌ `templates.get` - Get template
- ❌ `templates.create` - Create template from server
- ❌ `templates.sync` - Sync template with server
- ❌ `templates.edit` - Edit template
- ❌ `templates.delete` - Delete template

#### Welcome Screen
- ❌ `welcome.get` - Get welcome screen
- ❌ `welcome.edit` - Edit welcome screen

## Implementation Priorities for Feature Parity

### Phase 2.5 - Essential Write Operations (COMPLETED ✅)
1. ✅ Message sending (implemented)
2. ✅ Basic moderation (ban/kick implemented)
3. ✅ Channel management (create/delete implemented)
4. ✅ Message editing/deletion (implemented)
5. ✅ Role management (create implemented)
6. ✅ Extended role management (edit/delete/assign/remove implemented)
7. ✅ Extended member management (timeout/unban implemented)
8. ✅ Reaction support (add/remove implemented)
9. ✅ Channel editing (implemented)

### Phase 3 - Rich Messaging
1. Message history/retrieval
2. Reactions (add/remove/list/clear)
3. Pins
4. Bulk operations
5. Embeds and attachments

### Phase 4 - Modern Discord Features
1. Threads (create/manage/member operations)
2. Forum channels
3. Stage channels
4. Scheduled events (full CRUD)

### Phase 5 - Advanced Administration
1. Permission management
2. Audit logs
3. Auto-moderation
4. Server settings
5. Webhooks
6. Application commands (slash commands)

### Phase 6 - Edge Cases
1. Templates
2. Welcome screens
3. Widgets
4. Prune operations
5. Integration management

## Security Considerations

All WRITE operations must:
1. Be gated by `PermissionChecker`
2. Have appropriate `ActionType` and `ResourceType` mappings
3. Support allowlist/denylist filtering
4. Log security decisions
5. Provide clear error messages on denial

READ operations should:
1. Generally be allowed (information gathering)
2. Support optional permission checking for sensitive data
3. Be rate-limited appropriately

## Testing Requirements

Each command needs:
1. Unit tests for argument parsing
2. Mock tests for error handling
3. Integration tests with real Discord API (gated behind `#[cfg(feature = "api")]`)
4. Security policy tests
5. Documentation examples

## Estimated Coverage

- Current: **42 commands** (up from 35)
- Serenity API: ~120+ endpoints
- Coverage: ~35% (up from 29%)

**Phase 2.5 Complete!** Essential bot operations now at ~90% coverage:
- ✅ Complete role management (list, get, create, edit, delete, assign, remove)
- ✅ Complete member moderation (list, get, ban, kick, timeout, unban, edit, remove_timeout)
- ✅ Complete channel CRUD (list, get, create, edit, delete, get_or_create)
- ✅ Complete message operations (get, list, send, edit, delete, pin, unpin, clear)
- ✅ Reaction support (add, remove)
- ✅ Channel utilities (create_invite, typing)
- ✅ Extended read operations (bans.list, emojis.list, stickers.list, events.list, integrations.list, invites.list, webhooks.list, voice_regions.list)

Target for "feature parity":
- Essential operations (High + Medium priority): ~80 commands
- Would achieve: ~70% coverage with threads, webhooks, and scheduled events
- Sufficient for most bot use cases

## Recommendations

1. **Focus on High Priority** - These are the commands users expect in a bot framework
2. **Implement in phases** - Don't try to do everything at once
3. **Test thoroughly** - Each command needs comprehensive tests
4. **Document well** - Examples for every command
5. **Consider Gateway** - Some features (voice, real-time events) require WebSocket gateway
6. **Rate limiting** - Monitor and respect Discord rate limits
7. **Permissions** - Ensure bot has required Discord permissions for each operation

## Next Steps (Updated 2025-11-22)

### Immediate Priorities (Phase 3)
1. ✅ Complete narrative state management for persistent channel IDs
2. ⏭️ **Thread Management** - Critical for modern Discord communities
   - `threads.create` - Create thread from message or in forum/channel
   - `threads.list` - List active/archived threads
   - `threads.join`/`leave` - Thread membership
   - `threads.add_member`/`remove_member` - Thread member management
   - `threads.edit` - Modify thread settings
3. ⏭️ **Webhook Execution** - Essential for automation
   - `webhooks.create` - Create webhook
   - `webhooks.execute` - Send message via webhook
   - `webhooks.get`/`edit`/`delete` - Webhook management
4. ⏭️ **Scheduled Events** - Community engagement
   - `events.create` - Create scheduled event
   - `events.edit` - Edit event details
   - `events.delete` - Cancel event
   - `events.get` - Get event details
   - `events.list_users` - Get RSVPs

### Medium Term (Phase 4)
1. **Forum Channel Posts** - Q&A and discussion
2. **Stage Channel Instances** - Community talks/AMAs
3. **Advanced Reaction Management** - Clear reactions, list reactors
4. **Audit Logs** - Moderation and security tracking

### Long Term (Phase 5)
1. **Auto Moderation Rules** - Safety and spam prevention
2. **Permission Overwrites** - Fine-grained access control
3. **Emoji/Sticker Management** - Custom branding
4. **Server Templates** - Community replication

## Implementation Progress Update

### Phase 3: Forum Channels and Scheduled Events (IN PROGRESS)

**Status**: Implementation complete, but code needs structural refactoring

**Completed**:
- ✅ Added forum.create_post, forum.list_posts, forum.get_post commands
- ✅ Added events.create, events.edit, events.delete, events.get commands
- ✅ Added command documentation and help text
- ✅ Updated supported_commands list
- ✅ Added necessary helper functions (missing_arg_error, parse_channel_id, parse_guild_id, parse_event_id)
- ✅ Added necessary Serenity imports (CreateForumPost, CreateScheduledEvent, etc.)

**Issue Found**: 
- Helper methods (messages_bulk_delete, threads_*, reactions_*, members_*, roles_*, etc.) are currently inside the trait impl block but should be in the struct impl block
- This causes ~119 compilation errors

**Next Steps**:
1. Move all helper methods from inside trait impl (after line 4772) to before trait impl (before line 4251)
2. Ensure all methods are in DiscordCommandExecutor impl block, not BotCommandExecutor trait impl
3. Run cargo check to verify compilation
4. Test the new commands with actual narratives

