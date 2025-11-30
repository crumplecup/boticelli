//! Tests for narrative composition loading and context preservation.
//!
//! These tests verify that NarrativeSource correctly detects composition acts
//! and preserves MultiNarrative context when needed.

use botticelli_narrative::{NarrativeProvider, NarrativeSource};

#[test]
fn test_single_narrative_file_loads_as_single() {
    // Single-narrative TOML file should always load as Single variant
    let toml = r#"
[narrative]
name = "simple_test"
description = "A simple test narrative"

[toc]
order = ["greet"]

[acts.greet]
model = "gemini-2.0-flash-exp"
temperature = 0.7
max_tokens = 100

[[acts.greet.input]]
type = "text"
content = "Say hello"
"#;

    let temp_dir = tempfile::tempdir().unwrap();
    let file_path = temp_dir.path().join("simple.toml");
    std::fs::write(&file_path, toml).unwrap();

    let source = NarrativeSource::from_file(&file_path, None).unwrap();

    assert!(!source.has_composition_context());

    match &source {
        NarrativeSource::Single(narrative) => {
            assert_eq!(narrative.name(), "simple_test");
        }
        NarrativeSource::MultiWithContext { .. } => {
            panic!("Expected Single, got MultiWithContext");
        }
    }
}

#[test]
fn test_multi_narrative_without_composition_extracts_single() {
    // Multi-narrative file where the specified narrative has no composition
    // should extract as Single (no context overhead)
    let toml = r#"
[narratives.first]
description = "First narrative without composition"
toc = ["act1"]

[narratives.second]
description = "Second narrative"
toc = ["act2"]

[acts.act1]
model = "gemini-2.0-flash-exp"
temperature = 0.7
max_tokens = 100

[[acts.act1.input]]
type = "text"
content = "Do something"

[acts.act2]
model = "gemini-2.0-flash-exp"

[[acts.act2.input]]
type = "text"
content = "Do something else"
"#;

    let temp_dir = tempfile::tempdir().unwrap();
    let file_path = temp_dir.path().join("multi_no_composition.toml");
    std::fs::write(&file_path, toml).unwrap();

    let source = NarrativeSource::from_file(&file_path, Some("first")).unwrap();

    assert!(!source.has_composition_context());

    match &source {
        NarrativeSource::Single(narrative) => {
            assert_eq!(narrative.name(), "first");
        }
        NarrativeSource::MultiWithContext { .. } => {
            panic!("Expected Single, got MultiWithContext");
        }
    }
}

#[test]
fn test_multi_narrative_with_composition_preserves_context() {
    // Multi-narrative file where the specified narrative uses composition
    // should preserve full MultiWithContext
    let toml = r#"
[acts.call_worker]
narrative_ref = "worker"

[acts.work]
model = "gemini-2.0-flash-exp"
temperature = 0.5
max_tokens = 50

[[acts.work.input]]
type = "text"
content = "Do the work"

[narrative.orchestrator]
name = "orchestrator"
description = "Main narrative that composes others"
toc = ["call_worker"]

[narrative.worker]
name = "worker"
description = "Worker narrative"
toc = ["work"]
"#;

    let temp_dir = tempfile::tempdir().unwrap();
    let file_path = temp_dir.path().join("composition.toml");
    std::fs::write(&file_path, toml).unwrap();

    let source = NarrativeSource::from_file(&file_path, Some("orchestrator")).unwrap();

    assert!(source.has_composition_context());

    match &source {
        NarrativeSource::MultiWithContext {
            multi,
            execute_name,
        } => {
            assert_eq!(execute_name, "orchestrator");

            // Verify both narratives are accessible
            assert!(multi.get_narrative("orchestrator").is_some());
            assert!(multi.get_narrative("worker").is_some());
        }
        NarrativeSource::Single(_) => {
            panic!("Expected MultiWithContext, got Single");
        }
    }
}

#[test]
fn test_composition_with_multiple_references() {
    // Narrative with multiple composition acts should preserve context
    let toml = r#"
[acts.step1]
narrative_ref = "sub1"

[acts.step2]
narrative_ref = "sub2"

[acts.step3]
narrative_ref = "sub3"

[acts.work]
model = "gemini-2.0-flash-exp"

[[acts.work.input]]
type = "text"
content = "Sub1 work"

[narrative.main]
name = "main"
description = "Main with multiple composition acts"
toc = ["step1", "step2", "step3"]

[narrative.sub1]
name = "sub1"
toc = ["work"]

[narrative.sub2]
name = "sub2"
toc = ["work"]

[narrative.sub3]
name = "sub3"
toc = ["work"]
"#;

    let temp_dir = tempfile::tempdir().unwrap();
    let file_path = temp_dir.path().join("multi_ref.toml");
    std::fs::write(&file_path, toml).unwrap();

    let source = NarrativeSource::from_file(&file_path, Some("main")).unwrap();

    assert!(source.has_composition_context());

    if let NarrativeSource::MultiWithContext { multi, .. } = source {
        // All referenced narratives should be accessible
        assert!(multi.get_narrative("main").is_some());
        assert!(multi.get_narrative("sub1").is_some());
        assert!(multi.get_narrative("sub2").is_some());
        assert!(multi.get_narrative("sub3").is_some());
    } else {
        panic!("Expected MultiWithContext");
    }
}

#[test]
fn test_mixed_acts_with_composition() {
    // Narrative with both regular acts and composition acts should preserve context
    let toml = r#"
[acts.regular1]
model = "gemini-2.0-flash-exp"

[[acts.regular1.input]]
type = "text"
content = "Regular act 1"

[acts.composed]
narrative_ref = "helper"

[acts.regular2]
model = "gemini-2.0-flash-exp"

[[acts.regular2.input]]
type = "text"
content = "Regular act 2"

[acts.help]
model = "gemini-2.0-flash-exp"

[[acts.help.input]]
type = "text"
content = "Helper work"

[narrative.mixed]
name = "mixed"
description = "Has both regular and composition acts"
toc = ["regular1", "composed", "regular2"]

[narrative.helper]
name = "helper"
toc = ["help"]
"#;

    let temp_dir = tempfile::tempdir().unwrap();
    let file_path = temp_dir.path().join("mixed.toml");
    std::fs::write(&file_path, toml).unwrap();

    let source = NarrativeSource::from_file(&file_path, Some("mixed")).unwrap();

    // Should preserve context because of the composed act
    assert!(source.has_composition_context());
}

#[test]
fn test_narrative_source_get_narrative() {
    let toml = r#"
[narrative.test]
name = "test_narrative"
description = "Test"
toc = ["act"]

[acts.act]
model = "gemini-2.0-flash-exp"
[[acts.act.input]]
type = "text"
content = "Test"
"#;

    let temp_dir = tempfile::tempdir().unwrap();
    let file_path = temp_dir.path().join("test.toml");
    std::fs::write(&file_path, toml).unwrap();

    let source = NarrativeSource::from_file(&file_path, Some("test")).unwrap();

    let narrative = source.get_narrative().unwrap();
    assert_eq!(narrative.name(), "test_narrative");
}

#[test]
fn test_narrative_source_name() {
    let toml = r#"
[narrative]
name = "my_narrative"
description = "Test"

[toc]
order = ["act"]

[acts.act]
model = "gemini-2.0-flash-exp"
[[acts.act.input]]
type = "text"
content = "Test"
"#;

    let temp_dir = tempfile::tempdir().unwrap();
    let file_path = temp_dir.path().join("name_test.toml");
    std::fs::write(&file_path, toml).unwrap();

    let source = NarrativeSource::from_file(&file_path, None).unwrap();

    assert_eq!(source.name(), "my_narrative");
}

#[test]
fn test_multi_narrative_requires_name() {
    let toml = r#"
[acts.act]
model = "gemini-2.0-flash-exp"

[[acts.act.input]]
type = "text"
content = "Test"

[narrative.first]
name = "first"
toc = ["act"]

[narrative.second]
name = "second"
toc = ["act"]
"#;

    let temp_dir = tempfile::tempdir().unwrap();
    let file_path = temp_dir.path().join("multi_no_name.toml");
    std::fs::write(&file_path, toml).unwrap();

    // Should fail without narrative_name
    let result = NarrativeSource::from_file(&file_path, None);
    assert!(result.is_err());

    let err_msg = result.unwrap_err().to_string();
    assert!(
        err_msg.contains("Multiple narratives found"),
        "Error message: {}",
        err_msg
    );
}

#[test]
fn test_get_multi_context_returns_none_for_single() {
    let toml = r#"
[narrative]
name = "single"
description = "Single narrative"

[toc]
order = ["act"]

[acts.act]
model = "gemini-2.0-flash-exp"
[[acts.act.input]]
type = "text"
content = "Test"
"#;

    let temp_dir = tempfile::tempdir().unwrap();
    let file_path = temp_dir.path().join("single.toml");
    std::fs::write(&file_path, toml).unwrap();

    let source = NarrativeSource::from_file(&file_path, None).unwrap();

    assert!(source.get_multi_context().is_none());
}

#[test]
fn test_get_multi_context_returns_some_for_composition() {
    let toml = r#"
[acts.comp]
narrative_ref = "sub"

[acts.work]
model = "gemini-2.0-flash-exp"

[[acts.work.input]]
type = "text"
content = "Work"

[narrative.main]
name = "main"
toc = ["comp"]

[narrative.sub]
name = "sub"
toc = ["work"]
"#;

    let temp_dir = tempfile::tempdir().unwrap();
    let file_path = temp_dir.path().join("comp.toml");
    std::fs::write(&file_path, toml).unwrap();

    let source = NarrativeSource::from_file(&file_path, Some("main")).unwrap();

    assert!(source.get_multi_context().is_some());
}
