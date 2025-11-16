use boticelli::{Input, MediaSource, Narrative, NarrativeErrorKind};

#[test]
fn test_load_mint_narrative() {
    let narrative = Narrative::from_file("narrations/mint.toml").expect("Failed to load mint.toml");

    // Verify metadata
    assert_eq!(narrative.metadata.name, "mint");
    assert!(narrative.metadata.description.contains("MINT"));

    // Verify toc
    assert_eq!(narrative.toc.order, vec!["act1", "act2", "act3", "act4"]);

    // Verify acts exist
    assert!(narrative.acts.contains_key("act1"));
    assert!(narrative.acts.contains_key("act2"));
    assert!(narrative.acts.contains_key("act3"));
    assert!(narrative.acts.contains_key("act4"));

    // Verify ordered_acts returns correct order
    let ordered = narrative.ordered_acts();
    assert_eq!(ordered.len(), 4);
    assert_eq!(ordered[0].0, "act1");
    assert_eq!(ordered[1].0, "act2");
    assert_eq!(ordered[2].0, "act3");
    assert_eq!(ordered[3].0, "act4");
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

#[test]
fn test_multimodal_toml_parsing() {
    let toml_content = r#"
[narration]
name = "multimodal_test"
description = "Test multimodal act parsing"

[toc]
order = ["simple", "vision", "mixed"]

[acts]
simple = "Simple text act"

[acts.vision]
model = "gemini-pro-vision"
temperature = 0.3
max_tokens = 500

[[acts.vision.input]]
type = "text"
content = "Analyze this image"

[[acts.vision.input]]
type = "image"
mime = "image/png"
url = "https://example.com/image.png"

[acts.mixed]
model = "claude-3-opus"
temperature = 0.5

[[acts.mixed.input]]
type = "text"
content = "Review these materials"

[[acts.mixed.input]]
type = "image"
url = "https://example.com/chart.png"

[[acts.mixed.input]]
type = "document"
mime = "application/pdf"
url = "https://example.com/report.pdf"
filename = "report.pdf"
"#;

    let narrative: Narrative = toml_content.parse().expect("Should parse multimodal narrative");

    // Check metadata
    assert_eq!(narrative.metadata.name, "multimodal_test");
    assert_eq!(narrative.toc.order.len(), 3);
    assert_eq!(narrative.acts.len(), 3);

    // Check simple text act
    let simple_act = narrative.acts.get("simple").expect("simple act should exist");
    assert_eq!(simple_act.inputs.len(), 1);
    assert!(matches!(&simple_act.inputs[0], Input::Text(_)));
    assert_eq!(simple_act.model, None);

    // Check vision act
    let vision_act = narrative.acts.get("vision").expect("vision act should exist");
    assert_eq!(vision_act.inputs.len(), 2);
    assert!(matches!(&vision_act.inputs[0], Input::Text(_)));
    assert!(matches!(&vision_act.inputs[1], Input::Image { .. }));
    assert_eq!(vision_act.model, Some("gemini-pro-vision".to_string()));
    assert_eq!(vision_act.temperature, Some(0.3));
    assert_eq!(vision_act.max_tokens, Some(500));

    // Verify image source is URL
    if let Input::Image { source, mime } = &vision_act.inputs[1] {
        assert!(matches!(source, MediaSource::Url(_)));
        assert_eq!(mime, &Some("image/png".to_string()));
    } else {
        panic!("Expected Image input");
    }

    // Check mixed act
    let mixed_act = narrative.acts.get("mixed").expect("mixed act should exist");
    assert_eq!(mixed_act.inputs.len(), 3);
    assert!(matches!(&mixed_act.inputs[0], Input::Text(_)));
    assert!(matches!(&mixed_act.inputs[1], Input::Image { .. }));
    assert!(matches!(&mixed_act.inputs[2], Input::Document { .. }));
    assert_eq!(mixed_act.model, Some("claude-3-opus".to_string()));
    assert_eq!(mixed_act.temperature, Some(0.5));

    // Verify document has filename
    if let Input::Document { filename, .. } = &mixed_act.inputs[2] {
        assert_eq!(filename, &Some("report.pdf".to_string()));
    } else {
        panic!("Expected Document input");
    }
}

#[test]
fn test_load_showcase_narrative() {
    // This test verifies that the comprehensive showcase.toml example parses successfully
    let narrative = Narrative::from_file("narratives/showcase.toml")
        .expect("Failed to load showcase.toml");

    assert_eq!(narrative.metadata.name, "product_presentation_analysis");
    assert_eq!(narrative.toc.order.len(), 8);

    // Verify simple text act
    let initial = narrative.acts.get("initial_brief").expect("initial_brief should exist");
    assert_eq!(initial.inputs.len(), 1);
    assert!(matches!(&initial.inputs[0], Input::Text(_)));

    // Verify vision act with image
    let slides = narrative.acts.get("analyze_slides").expect("analyze_slides should exist");
    assert_eq!(slides.model, Some("gemini-pro-vision".to_string()));
    assert_eq!(slides.temperature, Some(0.3));
    assert_eq!(slides.inputs.len(), 2);

    // Verify audio act
    let pitch = narrative.acts.get("transcribe_pitch").expect("transcribe_pitch should exist");
    assert_eq!(pitch.model, Some("whisper-large-v3".to_string()));
    assert!(matches!(&pitch.inputs[1], Input::Audio { .. }));

    // Verify video act
    let video = narrative.acts.get("analyze_demo_video").expect("analyze_demo_video should exist");
    assert!(matches!(&video.inputs[1], Input::Video { .. }));

    // Verify document act
    let plan = narrative.acts.get("review_business_plan").expect("review_business_plan should exist");
    assert!(matches!(&plan.inputs[1], Input::Document { .. }));
}
