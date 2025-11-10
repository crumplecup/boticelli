use async_trait::async_trait;
use boticelli::{
    BoticelliDriver, BoticelliResult, GenerateRequest, GenerateResponse, Narrative,
    NarrativeExecutor, Output,
};

/// Mock LLM driver for testing that echoes the prompt with a prefix.
struct MockDriver {
    response_prefix: String,
}

impl MockDriver {
    fn new(response_prefix: &str) -> Self {
        Self {
            response_prefix: response_prefix.to_string(),
        }
    }
}

#[async_trait]
impl BoticelliDriver for MockDriver {
    async fn generate(&self, req: &GenerateRequest) -> BoticelliResult<GenerateResponse> {
        // Extract the last user message (current prompt)
        let last_message = req
            .messages
            .iter()
            .rev()
            .find(|m| matches!(m.role, boticelli::Role::User));

        let response_text = if let Some(msg) = last_message {
            // Extract text from the message
            let texts: Vec<String> = msg
                .content
                .iter()
                .filter_map(|input| {
                    if let boticelli::Input::Text(text) = input {
                        Some(text.clone())
                    } else {
                        None
                    }
                })
                .collect();

            format!("{}: {}", self.response_prefix, texts.join(" "))
        } else {
            format!("{}: (no prompt)", self.response_prefix)
        };

        Ok(GenerateResponse {
            outputs: vec![Output::Text(response_text)],
        })
    }

    fn provider_name(&self) -> &'static str {
        "mock"
    }

    fn model_name(&self) -> &str {
        "mock-model-v1"
    }
}

#[tokio::test]
async fn test_execute_simple_narrative() {
    let toml_content = r#"
        [narration]
        name = "test_narrative"
        description = "A simple test narrative"

        [toc]
        order = ["act1", "act2"]

        [acts]
        act1 = "First prompt"
        act2 = "Second prompt"
    "#;

    let narrative: Narrative = toml_content.parse().expect("Failed to parse narrative");
    let driver = MockDriver::new("Response");
    let executor = NarrativeExecutor::new(driver);

    let result = executor
        .execute(&narrative)
        .await
        .expect("Execution failed");

    assert_eq!(result.narrative_name, "test_narrative");
    assert_eq!(result.act_executions.len(), 2);

    // Check first act
    let act1 = &result.act_executions[0];
    assert_eq!(act1.act_name, "act1");
    assert_eq!(act1.prompt, "First prompt");
    assert_eq!(act1.response, "Response: First prompt");
    assert_eq!(act1.sequence_number, 0);

    // Check second act
    let act2 = &result.act_executions[1];
    assert_eq!(act2.act_name, "act2");
    assert_eq!(act2.prompt, "Second prompt");
    assert_eq!(act2.response, "Response: Second prompt");
    assert_eq!(act2.sequence_number, 1);
}

#[tokio::test]
async fn test_execute_single_act_narrative() {
    let toml_content = r#"
        [narration]
        name = "single_act"
        description = "A narrative with just one act"

        [toc]
        order = ["only_act"]

        [acts]
        only_act = "The only prompt"
    "#;

    let narrative: Narrative = toml_content.parse().expect("Failed to parse narrative");
    let driver = MockDriver::new("Result");
    let executor = NarrativeExecutor::new(driver);

    let result = executor
        .execute(&narrative)
        .await
        .expect("Execution failed");

    assert_eq!(result.narrative_name, "single_act");
    assert_eq!(result.act_executions.len(), 1);

    let act = &result.act_executions[0];
    assert_eq!(act.act_name, "only_act");
    assert_eq!(act.prompt, "The only prompt");
    assert_eq!(act.response, "Result: The only prompt");
    assert_eq!(act.sequence_number, 0);
}

#[tokio::test]
async fn test_context_passing_between_acts() {
    // Create a custom mock that records all messages it receives
    struct ContextTrackingDriver {
        call_count: std::sync::Arc<std::sync::Mutex<usize>>,
    }

    #[async_trait]
    impl BoticelliDriver for ContextTrackingDriver {
        async fn generate(&self, req: &GenerateRequest) -> BoticelliResult<GenerateResponse> {
            let mut count = self.call_count.lock().unwrap();
            *count += 1;
            let call_num = *count;

            // Verify that each subsequent call has more messages (conversation history)
            let num_messages = req.messages.len();

            // First call should have 1 message (just the prompt)
            // Second call should have 3 messages (user, assistant, user)
            // Third call should have 5 messages (user, assistant, user, assistant, user)
            let expected_messages = call_num * 2 - 1;
            assert_eq!(
                num_messages, expected_messages,
                "Act {} should have {} messages, but had {}",
                call_num, expected_messages, num_messages
            );

            Ok(GenerateResponse {
                outputs: vec![Output::Text(format!("Response {}", call_num))],
            })
        }

        fn provider_name(&self) -> &'static str {
            "context_tracking"
        }

        fn model_name(&self) -> &str {
            "context-tracker-v1"
        }
    }

    let toml_content = r#"
        [narration]
        name = "context_test"
        description = "Test context passing"

        [toc]
        order = ["act1", "act2", "act3"]

        [acts]
        act1 = "First"
        act2 = "Second"
        act3 = "Third"
    "#;

    let narrative: Narrative = toml_content.parse().expect("Failed to parse narrative");
    let driver = ContextTrackingDriver {
        call_count: std::sync::Arc::new(std::sync::Mutex::new(0)),
    };
    let executor = NarrativeExecutor::new(driver);

    let result = executor
        .execute(&narrative)
        .await
        .expect("Execution failed");

    assert_eq!(result.act_executions.len(), 3);
    assert_eq!(result.act_executions[0].response, "Response 1");
    assert_eq!(result.act_executions[1].response, "Response 2");
    assert_eq!(result.act_executions[2].response, "Response 3");
}

#[tokio::test]
async fn test_executor_driver_access() {
    let driver = MockDriver::new("Test");
    let executor = NarrativeExecutor::new(driver);

    // Verify we can access the driver
    assert_eq!(executor.driver().provider_name(), "mock");
    assert_eq!(executor.driver().model_name(), "mock-model-v1");
}
