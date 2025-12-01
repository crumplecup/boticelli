use botticelli_core::{GenerateRequest, Input, Message, Role};
use botticelli_interface::BotticelliDriver;
use botticelli_models::AnthropicClient;
use std::env;

#[tokio::test]
#[cfg_attr(not(feature = "api"), ignore)]
async fn test_anthropic_simple_generation() {
    let api_key =
        env::var("ANTHROPIC_API_KEY").expect("ANTHROPIC_API_KEY must be set for API tests");

    let client = AnthropicClient::new(api_key, "claude-3-5-sonnet-20241022");

    let message = Message::builder()
        .role(Role::User)
        .content(vec![Input::Text(
            "Say 'test' and nothing else.".to_string(),
        )])
        .build()
        .expect("Valid message");

    let request = GenerateRequest::builder()
        .messages(vec![message])
        .build()
        .expect("Valid request");

    let response = client.generate(&request).await.expect("API call succeeded");

    assert!(!response.outputs().is_empty());
    println!("Response: {:?}", response.outputs());
}

#[tokio::test]
#[cfg_attr(not(feature = "api"), ignore)]
async fn test_anthropic_with_temperature() {
    let api_key =
        env::var("ANTHROPIC_API_KEY").expect("ANTHROPIC_API_KEY must be set for API tests");

    let client = AnthropicClient::new(api_key, "claude-3-5-sonnet-20241022");

    let message = Message::builder()
        .role(Role::User)
        .content(vec![Input::Text("Count to 3.".to_string())])
        .build()
        .expect("Valid message");

    let request = GenerateRequest::builder()
        .messages(vec![message])
        .temperature(Some(0.5))
        .build()
        .expect("Valid request");

    let response = client.generate(&request).await.expect("API call succeeded");

    assert!(!response.outputs().is_empty());
    println!("Response with temperature: {:?}", response.outputs());
}
