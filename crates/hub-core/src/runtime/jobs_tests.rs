use std::time::Duration;

use super::{Internal, Jobs};

#[tokio::test(start_paused = true)]
async fn completed_job_returns_its_id_and_result() {
    let mut jobs = Jobs::new();
    jobs.spawn(
        "weather".into(),
        Box::pin(async { Ok(Box::new(42_u32) as Box<dyn std::any::Any + Send>) }),
        None,
        Duration::from_secs(5),
    );
    let Internal::JobDone(id, result, _) = jobs.next().await else {
        panic!("expected completion")
    };
    assert_eq!(id, "weather");
    assert_eq!(*result.unwrap().downcast::<u32>().unwrap(), 42);
}

#[tokio::test(start_paused = true)]
async fn timed_out_job_reports_timeout() {
    let mut jobs = Jobs::new();
    jobs.spawn(
        "slow".into(),
        Box::pin(std::future::pending()),
        None,
        Duration::from_secs(3),
    );
    assert!(matches!(jobs.next().await, Internal::Timeout(None)));
}

#[tokio::test(start_paused = true)]
async fn replacement_discards_already_queued_old_result() {
    let mut jobs = Jobs::new();
    jobs.spawn(
        "weather".into(),
        Box::pin(async { Ok(Box::new(1_u32) as Box<dyn std::any::Any + Send>) }),
        None,
        Duration::from_secs(5),
    );
    tokio::task::yield_now().await;
    jobs.spawn(
        "weather".into(),
        Box::pin(async { Ok(Box::new(2_u32) as Box<dyn std::any::Any + Send>) }),
        None,
        Duration::from_secs(5),
    );
    let Internal::JobDone(id, result, _) = jobs.next().await else {
        panic!("expected completion")
    };
    assert_eq!(id, "weather");
    assert_eq!(*result.unwrap().downcast::<u32>().unwrap(), 2);
}
