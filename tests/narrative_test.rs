use boticelli::{Narrative, NarrativeErrorKind};

#[test]
fn test_load_mint_narrative() {
    let narrative = Narrative::from_file("narrations/mint.toml").expect("Failed to load mint.toml");

    // Verify metadata
    assert_eq!(narrative.metadata.name, "mint");
    assert!(narrative.metadata.description.contains("MINT"));

    // Verify toc
    assert_eq!(narrative.toc.order, vec!["act1", "act2", "act3"]);

    // Verify acts exist
    assert!(narrative.acts.contains_key("act1"));
    assert!(narrative.acts.contains_key("act2"));
    assert!(narrative.acts.contains_key("act3"));

    // Verify ordered_acts returns correct order
    let ordered = narrative.ordered_acts();
    assert_eq!(ordered.len(), 3);
    assert_eq!(ordered[0].0, "act1");
    assert_eq!(ordered[1].0, "act2");
    assert_eq!(ordered[2].0, "act3");
}

#[test]
fn test_narrative_validation_empty_toc() {
    let toml_content = r#"
[narration]
name = "test"
description = "Test narrative"

[toc]
order = []

[acts]
act1 = "First prompt"
"#;

    let result: Result<Narrative, _> = toml_content.parse();
    assert!(result.is_err());
    if let Err(e) = result {
        assert!(matches!(e.kind, NarrativeErrorKind::EmptyToc));
    }
}

#[test]
fn test_narrative_validation_missing_act() {
    let toml_content = r#"
[narration]
name = "test"
description = "Test narrative"

[toc]
order = ["act1", "act2"]

[acts]
act1 = "First prompt"
"#;

    let result: Result<Narrative, _> = toml_content.parse();
    assert!(result.is_err());
    if let Err(e) = result {
        assert!(matches!(e.kind, NarrativeErrorKind::MissingAct(_)));
    }
}

#[test]
fn test_narrative_validation_empty_prompt() {
    let toml_content = r#"
[narration]
name = "test"
description = "Test narrative"

[toc]
order = ["act1"]

[acts]
act1 = "   "
"#;

    let result: Result<Narrative, _> = toml_content.parse();
    assert!(result.is_err());
    if let Err(e) = result {
        assert!(matches!(e.kind, NarrativeErrorKind::EmptyPrompt(_)));
    }
}

#[test]
fn test_narrative_valid() {
    let toml_content = r#"
[narration]
name = "test"
description = "Test narrative"

[toc]
order = ["act1", "act2"]

[acts]
act1 = "First prompt"
act2 = "Second prompt"
"#;

    let narrative: Narrative = toml_content.parse().expect("Should parse valid narrative");
    assert_eq!(narrative.metadata.name, "test");
    assert_eq!(narrative.toc.order.len(), 2);
    assert_eq!(narrative.acts.len(), 2);
}
