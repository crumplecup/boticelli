use async_trait::async_trait;
use botticelli_actor::{ActorBuilder, ActorError, Skill, SkillContext, SkillOutput, SkillOutputBuilder};
use botticelli_core::{GenerateRequestBuilder, Input, MessageBuilder, Role};
use std::sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
};

struct CountingSkill {
    counter: Arc<AtomicUsize>,
    name: String,
}

#[async_trait]
impl Skill for CountingSkill {
    fn name(&self) -> &str {
        &self.name
    }
    
    fn description(&self) -> &str {
        "A skill that counts executions"
    }
    
    async fn execute(&self, _ctx: &SkillContext) -> Result<SkillOutput, ActorError> {
        self.counter.fetch_add(1, Ordering::SeqCst);
        Ok(SkillOutputBuilder::default()
            .message(format!("{} executed", self.name))
            .build()
            .expect("Valid output"))
    }
}

#[tokio::test]
async fn test_full_bot_pipeline() {
    let counter = Arc::new(AtomicUsize::new(0));

    let mut actor = ActorBuilder::default()
        .name("pipeline_test")
        .skill(Box::new(CountingSkill {
            counter: counter.clone(),
            name: "stage1".to_string(),
        }))
        .skill(Box::new(CountingSkill {
            counter: counter.clone(),
            name: "stage2".to_string(),
        }))
        .skill(Box::new(CountingSkill {
            counter: counter.clone(),
            name: "stage3".to_string(),
        }))
        .build()
        .expect("Valid actor");

    let request = GenerateRequestBuilder::default()
        .messages(vec![
            MessageBuilder::default()
                .role(Role::User)
                .content(vec![Input::text("test pipeline")])
                .build()
                .expect("Valid message"),
        ])
        .build()
        .expect("Valid request");

    actor
        .execute(&request)
        .await
        .expect("Pipeline should succeed");

    // All three stages should have executed
    assert_eq!(
        counter.load(Ordering::SeqCst),
        3,
        "All pipeline stages should execute"
    );
}

#[tokio::test]
async fn test_concurrent_bot_execution() {
    let counter = Arc::new(AtomicUsize::new(0));

    let mut handles = vec![];

    for i in 0..5 {
        let counter_clone = counter.clone();
        let handle = tokio::spawn(async move {
            let mut actor = ActorBuilder::default()
                .name(&format!("concurrent_{}", i))
                .skill(Box::new(CountingSkill {
                    counter: counter_clone,
                    name: format!("bot_{}", i),
                }))
                .build()
                .expect("Valid actor");

            let request = GenerateRequestBuilder::default()
                .messages(vec![
                    MessageBuilder::default()
                        .role(Role::User)
                        .content(vec![Input::text("concurrent test")])
                        .build()
                        .expect("Valid message"),
                ])
                .build()
                .expect("Valid request");

            actor.execute(&request).await
        });
        handles.push(handle);
    }

    for handle in handles {
        handle
            .await
            .expect("Task should complete")
            .expect("Actor should succeed");
    }

    assert_eq!(
        counter.load(Ordering::SeqCst),
        5,
        "All actors should execute"
    );
}

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

#[tokio::test]
async fn test_actor_failure_isolation() {
    let success_counter = Arc::new(AtomicUsize::new(0));

    let mut handles = vec![];

    // Spawn failing actor
    let fail_handle = tokio::spawn(async {
        let mut actor = ActorBuilder::default()
            .name("failing_actor")
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

        actor.execute(&request).await
    });
    handles.push(fail_handle);

    // Spawn successful actors
    for i in 0..3 {
        let counter = success_counter.clone();
        let handle = tokio::spawn(async move {
            let mut actor = ActorBuilder::default()
                .name(&format!("success_actor_{}", i))
                .skill(Box::new(CountingSkill {
                    counter,
                    name: format!("success_{}", i),
                }))
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

            actor.execute(&request).await
        });
        handles.push(handle);
    }

    let mut success_count = 0;
    let mut fail_count = 0;

    for handle in handles {
        match handle.await.expect("Task should complete") {
            Ok(_) => success_count += 1,
            Err(_) => fail_count += 1,
        }
    }

    assert_eq!(fail_count, 1, "One actor should fail");
    assert_eq!(success_count, 3, "Three actors should succeed");
    assert_eq!(
        success_counter.load(Ordering::SeqCst),
        3,
        "Successful actors should all execute"
    );
}
