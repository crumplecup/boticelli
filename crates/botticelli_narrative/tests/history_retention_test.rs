//! Tests for conversation history retention functionality.

use botticelli_core::{HistoryRetention, Input, TableFormat};
use botticelli_narrative::{apply_retention_to_inputs, should_auto_summarize, summarize_input};

#[test]
fn test_summarize_table_input() {
    let input = Input::Table {
        table_name: "large_table".to_string(),
        columns: None,
        where_clause: None,
        limit: Some(10),
        offset: None,
        order_by: None,
        alias: None,
        format: TableFormat::Json,
        sample: None,
        history_retention: HistoryRetention::Summary,
    };

    let summary = summarize_input(&input);
    assert!(summary.contains("Table: large_table"));
    assert!(summary.contains("10 rows queried"));
    assert!(summary.len() < 100);
}

#[test]
fn test_summarize_table_with_offset() {
    let input = Input::Table {
        table_name: "test_table".to_string(),
        columns: None,
        where_clause: None,
        limit: Some(5),
        offset: Some(20),
        order_by: None,
        alias: None,
        format: TableFormat::Json,
        sample: None,
        history_retention: HistoryRetention::Summary,
    };

    let summary = summarize_input(&input);
    assert!(summary.contains("Table: test_table"));
    assert!(summary.contains("5 rows queried"));
    assert!(summary.contains("offset 20"));
}

#[test]
fn test_summarize_table_no_limit() {
    let input = Input::Table {
        table_name: "all_rows_table".to_string(),
        columns: None,
        where_clause: None,
        limit: None,
        offset: None,
        order_by: None,
        alias: None,
        format: TableFormat::Json,
        sample: None,
        history_retention: HistoryRetention::Summary,
    };

    let summary = summarize_input(&input);
    assert!(summary.contains("Table: all_rows_table"));
    assert!(summary.contains("all rows"));
}

#[test]
fn test_summarize_large_text() {
    let large_text = "a".repeat(5000);
    let input = Input::Text(large_text);

    let summary = summarize_input(&input);
    assert!(summary.contains("[Text:"));
    assert!(summary.contains("KB]"));
    assert!(summary.len() < 50);
}

#[test]
fn test_summarize_small_text_unchanged() {
    let small_text = "This is a short text.".to_string();
    let input = Input::Text(small_text.clone());

    let summary = summarize_input(&input);
    assert_eq!(summary, small_text);
}

#[test]
fn test_summarize_bot_command() {
    let input = Input::BotCommand {
        platform: "discord".to_string(),
        command: "server.get_stats".to_string(),
        args: std::collections::HashMap::new(),
        required: false,
        cache_duration: None,
        history_retention: HistoryRetention::Summary,
    };

    let summary = summarize_input(&input);
    assert_eq!(summary, "[Bot command: discord.server.get_stats]");
}

#[test]
fn test_summarize_narrative() {
    let input = Input::Narrative {
        name: "content_generation".to_string(),
        path: None,
        history_retention: HistoryRetention::Summary,
    };

    let summary = summarize_input(&input);
    assert_eq!(summary, "[Nested narrative: content_generation]");
}

#[test]
fn test_should_auto_summarize_large_text() {
    let large_text = "a".repeat(15000);
    let input = Input::Text(large_text);

    assert!(should_auto_summarize(&input));
}

#[test]
fn test_should_auto_summarize_small_text() {
    let small_text = Input::Text("small".to_string());

    assert!(!should_auto_summarize(&small_text));
}

#[test]
fn test_apply_retention_full_keeps_input() {
    let input = Input::Text("Keep this text".to_string());
    let inputs = vec![input.clone()];

    let result = apply_retention_to_inputs(&inputs);

    assert_eq!(result.len(), 1);
    assert_eq!(result[0], input);
}

#[test]
fn test_apply_retention_summary_replaces_large_input() {
    let input = Input::Table {
        table_name: "test".to_string(),
        columns: None,
        where_clause: None,
        limit: Some(100),
        offset: None,
        order_by: None,
        alias: None,
        format: TableFormat::Json,
        sample: None,
        history_retention: HistoryRetention::Summary,
    };
    let inputs = vec![input];

    let result = apply_retention_to_inputs(&inputs);

    assert_eq!(result.len(), 1);
    match &result[0] {
        Input::Text(text) => {
            assert!(text.contains("[Table:"));
            assert!(text.len() < 100);
        }
        _ => panic!("Expected Text variant with summary"),
    }
}

#[test]
fn test_apply_retention_drop_removes_input() {
    let input = Input::Table {
        table_name: "drop_me".to_string(),
        columns: None,
        where_clause: None,
        limit: Some(10),
        offset: None,
        order_by: None,
        alias: None,
        format: TableFormat::Json,
        sample: None,
        history_retention: HistoryRetention::Drop,
    };
    let inputs = vec![input];

    let result = apply_retention_to_inputs(&inputs);

    assert_eq!(result.len(), 0);
}

#[test]
fn test_apply_retention_mixed_policies() {
    let inputs = vec![
        Input::Text("Keep this".to_string()), // Default: Full
        Input::Table {
            table_name: "summarize_me".to_string(),
            columns: None,
            where_clause: None,
            limit: Some(10),
            offset: None,
            order_by: None,
            alias: None,
            format: TableFormat::Json,
            sample: None,
            history_retention: HistoryRetention::Summary,
        },
        Input::Table {
            table_name: "drop_me".to_string(),
            columns: None,
            where_clause: None,
            limit: Some(5),
            offset: None,
            order_by: None,
            alias: None,
            format: TableFormat::Json,
            sample: None,
            history_retention: HistoryRetention::Drop,
        },
    ];

    let result = apply_retention_to_inputs(&inputs);

    // First input kept, second summarized, third dropped
    assert_eq!(result.len(), 2);

    // First input should be unchanged
    match &result[0] {
        Input::Text(text) => assert_eq!(text, "Keep this"),
        _ => panic!("Expected Text variant"),
    }

    // Second input should be summarized
    match &result[1] {
        Input::Text(text) => {
            assert!(text.contains("[Table: summarize_me"));
        }
        _ => panic!("Expected Text variant with summary"),
    }
}

#[test]
fn test_apply_retention_auto_summarize_large_full_input() {
    let large_text = "a".repeat(15000);
    let input = Input::Text(large_text);
    let inputs = vec![input];

    let result = apply_retention_to_inputs(&inputs);

    // Should be auto-summarized even though retention is Full (default)
    assert_eq!(result.len(), 1);
    match &result[0] {
        Input::Text(text) => {
            assert!(text.contains("[Text:"));
            assert!(text.contains("KB]"));
            assert!(text.len() < 50);
        }
        _ => panic!("Expected Text variant with summary"),
    }
}

#[test]
fn test_history_retention_default_is_full() {
    let input = Input::Text("test".to_string());
    assert_eq!(input.history_retention(), HistoryRetention::Full);
}

#[test]
fn test_with_history_retention_modifies_table() {
    let input = Input::Table {
        table_name: "test".to_string(),
        columns: None,
        where_clause: None,
        limit: Some(10),
        offset: None,
        order_by: None,
        alias: None,
        format: TableFormat::Json,
        sample: None,
        history_retention: HistoryRetention::Full,
    };

    let modified = input.with_history_retention(HistoryRetention::Summary);

    assert_eq!(modified.history_retention(), HistoryRetention::Summary);
}

#[test]
fn test_with_history_retention_modifies_bot_command() {
    let input = Input::BotCommand {
        platform: "discord".to_string(),
        command: "test".to_string(),
        args: std::collections::HashMap::new(),
        required: false,
        cache_duration: None,
        history_retention: HistoryRetention::Full,
    };

    let modified = input.with_history_retention(HistoryRetention::Drop);

    assert_eq!(modified.history_retention(), HistoryRetention::Drop);
}

#[test]
fn test_with_history_retention_modifies_narrative() {
    let input = Input::Narrative {
        name: "test".to_string(),
        path: None,
        history_retention: HistoryRetention::Full,
    };

    let modified = input.with_history_retention(HistoryRetention::Summary);

    assert_eq!(modified.history_retention(), HistoryRetention::Summary);
}

#[test]
fn test_with_history_retention_ignores_text_input() {
    let input = Input::Text("test".to_string());

    let modified = input.with_history_retention(HistoryRetention::Summary);

    // Text inputs don't support history_retention, should still be Full
    assert_eq!(modified.history_retention(), HistoryRetention::Full);
}
