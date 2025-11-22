# Narrative Test Fixes Needed

## Overview
Multiple test narrative files are using the old `[[act]]` array syntax instead of the current `[toc]` + `[acts]` + friendly syntax pattern.

## Files Needing Conversion

1. test_channel_commands.toml - NEEDS FIX
2. test_members.toml - NEEDS FIX  
3. test_message_commands.toml - NEEDS FIX
4. test_messages.toml - NEEDS FIX
5. test_reaction_commands.toml - NEEDS FIX
6. test_roles.toml - NEEDS FIX
7. test_server_commands.toml - NEEDS FIX
8. test_server_stats.toml - NEEDS FIX

## Already Fixed

- test_role_commands.toml ✅
- test_member_commands.toml ✅
- test_channels_list.toml ✅ (already correct)
- test_guilds_get.toml ✅ (already correct)
- test_members_list.toml ✅ (already correct)
- test_messages_send.toml ✅ (already correct)
- test_roles_list.toml ✅ (already correct)

## Conversion Pattern

### OLD (deprecated):
```toml
[[act]]
name = "my_act"
prompt = "Do something"

[[act.input]]
type = "bot_command"
platform = "discord"
command = "roles.list"

[act.input.args]
guild_id = "${TEST_GUILD_ID}"
```

### NEW (current):
```toml
[toc]
order = ["my_act"]

[bots.my_cmd]
platform = "discord"
command = "roles.list"
guild_id = "${TEST_GUILD_ID}"

[acts]
my_act = "bots.my_cmd"
```

## Next Steps
1. Fix each file one by one following the pattern
2. Run tests to verify each fix
3. Update TEST_FIXES_NEEDED when complete
