//! Integration test for Discord publish_welcome narrative

use botticelli::{
    establish_connection, GeminiClient, Narrative, NarrativeExecutor, ProcessorRegistry,
};
use botticelli_narrative::ContentGenerationProcessor;
use std::path::Path;
use std::sync::{Arc, Mutex};

#[tokio::test]
#[cfg_attr(not(feature = "api"), ignore)]
async fn test_publish_welcome_narrative() {
    // Load environment variables
    dotenvy::dotenv().ok();
    
    // Get required environment variables
    let test_guild_id = std::env::var("TEST_GUILD_ID")
        .expect("TEST_GUILD_ID must be set");
    let welcome_channel_id = std::env::var("WELCOME_CHANNEL_ID")
        .expect("WELCOME_CHANNEL_ID must be set");
    
    println!("Test Guild ID: {}", test_guild_id);
    println!("Welcome Channel ID: {}", welcome_channel_id);
    
    // Load narrative
    let narrative_path = Path::new("crates/botticelli_narrative/narratives/discord/publish_welcome.toml");
    let mut conn = establish_connection().expect("Failed to connect to database");
    let narrative = Narrative::from_file_with_db(narrative_path, &mut conn)
        .expect("Failed to load narrative");
    
    println!("Loaded narrative: {}", narrative.metadata.name);
    println!("Acts: {:?}", narrative.toc.order);
    
    // Create Gemini client
    let client = GeminiClient::new().expect("Failed to create Gemini client");
    
    // Create content generation processor
    let conn = establish_connection().expect("Failed to connect to database");
    let processor = ContentGenerationProcessor::new(Arc::new(Mutex::new(conn)));
    
    let mut registry = ProcessorRegistry::new();
    registry.register(Box::new(processor));
    
    // Create executor with processors
    let executor = NarrativeExecutor::with_processors(client, registry);
    
    // Execute the narrative
    let result = executor.execute(&narrative).await;
    
    match result {
        Ok(execution) => {
            println!("✅ Narrative executed successfully!");
            println!("Narrative: {}", execution.narrative_name);
            println!("\nAct executions:");
            for (i, act_exec) in execution.act_executions.iter().enumerate() {
                println!("\n  Act {}: {}", i + 1, act_exec.act_name);
                if let Some(model) = &act_exec.model {
                    println!("    Model: {}", model);
                }
                println!("    Response: {}", act_exec.response);
            }
        }
        Err(e) => {
            panic!("❌ Narrative execution failed: {}", e);
        }
    }
}
