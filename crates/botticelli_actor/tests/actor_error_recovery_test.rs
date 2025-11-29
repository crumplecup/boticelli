use async_trait::async_trait;
use botticelli_actor::{ActorBuilder, ActorError, Skill, SkillContext, SkillOutput, SkillOutputBuilder};
use botticelli_core::{GenerateRequestBuilder, Input, MessageBuilder, Role};

struct FailingSkill;

#[async_trait]
impl Skill for FailingSkill {
    fn name(&self) -> &str {
        "FailingSkill"
    }
    
    fn description(&self) -> &str {
        "A skill that always fails"
    }
    
    async fn execute(&self, _ctx: &SkillContext) -> Result<SkillOutput, ActorError> {
        Err(ActorError::execution_failed("Intentional failure", file!(), line!()))
    }
}

struct SuccessSkill;

#[async_trait]
impl Skill for SuccessSkill {
    fn name(&self) -> &str {
        "SuccessSkill"
    }
    
    fn description(&self) -> &str {
        "A skill that always succeeds"
    }
    
    async fn execute(&self, ctx: &SkillContext) -> Result<SkillOutput, ActorError> {
        Ok(SkillOutputBuilder::default()
            .message(format!("Success from {}", ctx.actor_name()))
            .build()
            .expect("Valid output"))
    }
}

#[tokio::test]
async fn test_actor_handles_skill_failure() {
    let mut actor = ActorBuilder::default()
        .name("error_test")
        .skill(Box::new(FailingSkill))
        .build()
        .expect("Valid actor");

    let request = GenerateRequestBuilder::default()
        .messages(vec![
            MessageBuilder::default()
                .role(Role::User)
                .content(vec![Input::text("test")])
                .build()
                .expect("Valid message"),
        ])
        .build()
        .expect("Valid request");

    // Actor should handle error gracefully without panicking
    let result = actor.execute(&request).await;
    assert!(result.is_err(), "Should return error from failing skill");
}

#[tokio::test]
async fn test_actor_recovers_after_failure() {
    let mut actor = ActorBuilder::default()
        .name("recovery_test")
        .skill(Box::new(FailingSkill))
        .skill(Box::new(SuccessSkill))
        .build()
        .expect("Valid actor");

    let request = GenerateRequestBuilder::default()
        .messages(vec![
            MessageBuilder::default()
                .role(Role::User)
                .content(vec![Input::text("test")])
                .build()
                .expect("Valid message"),
        ])
        .build()
        .expect("Valid request");

    // First execution fails
    let result1 = actor.execute(&request).await;
    assert!(result1.is_err(), "First execution should fail");

    // Actor should still be usable after failure
    assert_eq!(actor.name(), "recovery_test");
    assert_eq!(actor.skills().len(), 2, "Skills should still be available");
}
