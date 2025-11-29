use botticelli_database::{format_data_type, is_content_focus};

#[test]
fn test_is_content_focus_short() {
    let short = "Create a welcoming creative community for artists and musicians.";
    assert!(is_content_focus(short));
}

#[test]
fn test_is_content_focus_long_with_keywords() {
    let explicit = r#"
        Create a JSON object with the following schema:
        
        **Required Fields:**
        - id: bigint
        - name: varchar(100)
        "#;
    assert!(!is_content_focus(explicit));
}

#[test]
fn test_is_content_focus_critical_keyword() {
    let explicit = "Some text... **CRITICAL OUTPUT REQUIREMENTS:** ...";
    assert!(!is_content_focus(explicit));
}

#[test]
fn test_format_data_type_varchar() {
    assert_eq!(
        format_data_type("character varying", Some(100)),
        "text (max 100 chars)"
    );
}

#[test]
fn test_format_data_type_bigint() {
    assert_eq!(format_data_type("bigint", None), "64-bit integer");
}
