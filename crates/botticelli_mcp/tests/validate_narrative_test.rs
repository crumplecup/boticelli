//! Tests for validate_narrative tool.

use botticelli_mcp::tools::{McpTool, ToolRegistry, ValidateNarrativeTool};
use serde_json::json;

#[tokio::test]
async fn test_validate_valid_narrative() {
    let tool = ValidateNarrativeTool;

    let input = json!({
        "content": r#"
[narrative]
name = "test"
description = "Test narrative"

[toc]
order = ["act1"]

[acts]
act1 = "Hello world"
        "#
    });

    let result = tool.execute(input).await.unwrap();
    assert_eq!(result["valid"], true);
    assert_eq!(result["errors"].as_array().unwrap().len(), 0);
    assert_eq!(result["warnings"].as_array().unwrap().len(), 0);
}

#[tokio::test]
async fn test_validate_invalid_syntax() {
    let tool = ValidateNarrativeTool;

    let input = json!({
        "content": r#"
[narrative]
name = "test"

[toc]
order = ["act1"]

[[acts]]
name = "act1"
prompt = "Hello"
        "#
    });

    let result = tool.execute(input).await.unwrap();
    assert_eq!(result["valid"], false);
    assert!(!result["errors"].as_array().unwrap().is_empty());

    let error_msg = result["errors"][0]["message"].as_str().unwrap();
    assert!(error_msg.contains("[[acts]]"));
}

#[tokio::test]
async fn test_validate_unknown_model() {
    let tool = ValidateNarrativeTool;

    let input = json!({
        "content": r#"
[narrative]
name = "test"
description = "Test"
model = "gpt-5-turbo"

[toc]
order = ["act1"]

[acts]
act1 = "Hello"
        "#,
        "validate_models": true
    });

    let result = tool.execute(input).await.unwrap();
    assert_eq!(result["valid"], true); // Warnings don't fail validation
    assert!(!result["warnings"].as_array().unwrap().is_empty());

    let warning_msg = result["warnings"][0]["message"].as_str().unwrap();
    assert!(warning_msg.contains("gpt-5-turbo"));
    assert!(warning_msg.contains("gpt-4-turbo"));
}

#[tokio::test]
async fn test_validate_unused_resources() {
    let tool = ValidateNarrativeTool;

    let input = json!({
        "content": r#"
[narrative]
name = "test"

[bots.unused]
platform = "discord"
command = "test"

[toc]
order = ["act1"]

[acts]
act1 = "Hello"
        "#,
        "warn_unused": true
    });

    let result = tool.execute(input).await.unwrap();
    assert_eq!(result["valid"], true);
    assert!(!result["warnings"].as_array().unwrap().is_empty());

    let warning_msg = result["warnings"][0]["message"].as_str().unwrap();
    assert!(warning_msg.contains("unused"));
}

#[tokio::test]
async fn test_validate_circular_dependency() {
    let tool = ValidateNarrativeTool;

    let input = json!({
        "content": r#"
[narratives.first]
description = "First"
toc = ["step1"]

[narratives.first.acts]
step1 = "narrative.second"

[narratives.second]
description = "Second"
toc = ["step2"]

[narratives.second.acts]
step2 = "narrative.first"
        "#
    });

    let result = tool.execute(input).await.unwrap();
    assert_eq!(result["valid"], false);
    assert!(!result["errors"].as_array().unwrap().is_empty());

    let error_msg = result["errors"][0]["message"].as_str().unwrap();
    assert!(error_msg.contains("Circular dependency"));
}

#[tokio::test]
async fn test_validate_strict_mode() {
    let tool = ValidateNarrativeTool;

    let input = json!({
        "content": r#"
[narrative]
name = "test"
model = "unknown-model"

[toc]
order = ["act1"]

[acts]
act1 = "Hello"
        "#,
        "strict": true
    });

    let result = tool.execute(input).await.unwrap();
    // Strict mode treats warnings as errors
    assert_eq!(result["valid"], false);
    assert!(!result["warnings"].as_array().unwrap().is_empty());
}

#[tokio::test]
async fn test_tool_registry_includes_validator() {
    let registry = ToolRegistry::default();
    let tool = registry.get("validate_narrative");
    assert!(tool.is_some());
    assert_eq!(tool.unwrap().name(), "validate_narrative");
}
