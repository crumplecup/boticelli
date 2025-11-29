use botticelli_database::{generate_create_table_sql, map_data_type, ColumnInfo, TableSchema};

#[test]
fn test_map_data_type() {
    assert_eq!(map_data_type("bigint"), "BIGINT");
    assert_eq!(map_data_type("text"), "TEXT");
    assert_eq!(map_data_type("boolean"), "BOOLEAN");
    assert_eq!(map_data_type("unknown_type"), "TEXT");
}

#[test]
fn test_generate_create_table_sql() {
    let schema = TableSchema {
        table_name: "test_source".to_string(),
        columns: vec![
            ColumnInfo {
                name: "id".to_string(),
                data_type: "bigint".to_string(),
                is_nullable: "NO".to_string(),
                character_maximum_length: None,
                column_default: Some("nextval('seq'::regclass)".to_string()),
            },
            ColumnInfo {
                name: "name".to_string(),
                data_type: "character varying".to_string(),
                is_nullable: "NO".to_string(),
                character_maximum_length: Some(100),
                column_default: None,
            },
            ColumnInfo {
                name: "guild_id".to_string(),
                data_type: "bigint".to_string(),
                is_nullable: "NO".to_string(),
                character_maximum_length: None,
                column_default: None,
            },
        ],
    };

    let sql = generate_create_table_sql("test_target", &schema);

    assert!(sql.contains("CREATE TABLE test_target"));
    assert!(sql.contains("id BIGINT NOT NULL"));
    assert!(sql.contains("name VARCHAR(100) NOT NULL"));
    assert!(sql.contains("guild_id BIGINT NULL")); // FK made nullable
    assert!(sql.contains("generated_at TIMESTAMP NOT NULL DEFAULT NOW()"));
    assert!(sql.contains("source_narrative TEXT"));
    assert!(sql.contains("review_status TEXT DEFAULT 'pending'"));
}
