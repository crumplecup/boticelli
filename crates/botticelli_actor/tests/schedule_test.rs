//! Tests for schedule configuration and execution logic.

use botticelli_server::{Schedule, ScheduleType};
use chrono::{Datelike, Duration, Timelike, Utc};

#[test]
fn test_immediate_schedule() {
    let schedule = ScheduleType::Immediate;
    
    // Should be ready immediately on first check
    let check = schedule.check(None);
    assert!(check.should_run);
    assert!(check.next_run.is_none());
    
    // Should not be ready after execution
    let now = Utc::now();
    let check = schedule.check(Some(now));
    assert!(!check.should_run);
}

#[test]
fn test_once_schedule() {
    let future_time = Utc::now() + Duration::hours(1);
    let schedule = ScheduleType::Once { at: future_time };
    
    // Should not be ready before scheduled time
    let check = schedule.check(None);
    assert!(!check.should_run);
    assert!(check.next_run.is_some());
    
    // Should be ready after scheduled time
    let past_time = Utc::now() - Duration::hours(2);
    let schedule = ScheduleType::Once { at: past_time };
    let check = schedule.check(None);
    assert!(check.should_run);
}

#[test]
fn test_interval_schedule() {
    let schedule = ScheduleType::Interval { seconds: 3600 }; // 1 hour
    
    // Should be ready on first check
    let check = schedule.check(None);
    assert!(check.should_run);
    
    // Should not be ready immediately after execution
    let now = Utc::now();
    let check = schedule.check(Some(now));
    assert!(!check.should_run);
    assert!(check.next_run.is_some());
    
    // Should be ready after interval has passed
    let past = Utc::now() - Duration::hours(2);
    let check = schedule.check(Some(past));
    assert!(check.should_run);
}

#[test]
fn test_cron_schedule() {
    // Every day at 9 AM
    let schedule = ScheduleType::Cron {
        expression: "0 0 9 * * * *".to_string(),
    };
    
    // Should have a next execution time
    let next = schedule.next_execution(Utc::now());
    assert!(next.is_some());
}

#[test]
fn test_cron_schedule_invalid() {
    let schedule = ScheduleType::Cron {
        expression: "invalid cron".to_string(),
    };
    
    // Should handle invalid cron gracefully
    let next = schedule.next_execution(Utc::now());
    assert!(next.is_none());
}

#[test]
fn test_schedule_next_execution() {
    let now = Utc::now();
    
    // Immediate: no next execution
    let schedule = ScheduleType::Immediate;
    assert!(schedule.next_execution(now).is_none());
    
    // Once: next is the scheduled time
    let future = now + Duration::hours(1);
    let schedule = ScheduleType::Once { at: future };
    let next = schedule.next_execution(now);
    assert!(next.is_some());
    assert_eq!(next.unwrap(), future);
    
    // Interval: next is now + interval
    let schedule = ScheduleType::Interval { seconds: 3600 };
    let next = schedule.next_execution(now);
    assert!(next.is_some());
    let diff = next.unwrap() - now;
    assert!(diff.num_seconds() >= 3599 && diff.num_seconds() <= 3601);
}

#[test]
fn test_schedule_check_with_next_run() {
    let now = Utc::now();
    let past = now - Duration::hours(1);
    
    let schedule = ScheduleType::Interval { seconds: 3600 };
    
    // Last run in past, should be ready
    let check = schedule.check(Some(past));
    assert!(check.should_run);
    
    // Last run just now, should not be ready
    let check = schedule.check(Some(now));
    assert!(!check.should_run);
    assert!(check.next_run.is_some());
    assert!(check.next_run.unwrap() > now);
}

#[test]
fn test_cron_daily_schedule() {
    // Every day at 9:00 AM
    let schedule = ScheduleType::Cron {
        expression: "0 0 9 * * * *".to_string(),
    };
    
    let now = Utc::now();
    let next = schedule.next_execution(now);
    assert!(next.is_some());
    
    let next_time = next.unwrap();
    assert!(next_time > now);
    assert_eq!(next_time.hour(), 9);
    assert_eq!(next_time.minute(), 0);
    assert_eq!(next_time.second(), 0);
}

#[test]
fn test_cron_weekday_schedule() {
    // Every weekday at 9:30 AM
    let schedule = ScheduleType::Cron {
        expression: "0 30 9 * * Mon-Fri *".to_string(),
    };
    
    let now = Utc::now();
    let next = schedule.next_execution(now);
    assert!(next.is_some());
    
    let next_time = next.unwrap();
    assert!(next_time > now);
    
    // Should be a weekday (Monday = 1, Friday = 5)
    let weekday = next_time.weekday().number_from_monday();
    assert!(weekday >= 1 && weekday <= 5);
}

#[test]
fn test_interval_zero_seconds() {
    // Zero interval should still work (runs as fast as possible)
    let schedule = ScheduleType::Interval { seconds: 0 };
    
    let check = schedule.check(None);
    assert!(check.should_run);
    
    let now = Utc::now();
    let check = schedule.check(Some(now));
    // With zero interval, should be ready immediately
    assert!(check.should_run);
}

#[test]
fn test_once_schedule_already_executed() {
    let past = Utc::now() - Duration::hours(1);
    let schedule = ScheduleType::Once { at: past };
    
    // Should be ready if never executed
    let check = schedule.check(None);
    assert!(check.should_run);
    
    // Should not be ready if already executed
    let check = schedule.check(Some(past - Duration::minutes(1)));
    assert!(!check.should_run);
}
