use crate::communication::{
    DefaultAlwaysImmediateSend, OnlyChangesImmediateSend, RateLimitedMostRecentSend,
    fire_and_forget_channel, fire_and_forget_channel_with,
};

#[tokio::test]
async fn test_fire_and_forget_channel_default_config() {
    let mut channel = fire_and_forget_channel::<i32, DefaultAlwaysImmediateSend<_>>();
    let mut receiver = channel.subscribe();

    // Test sending a value
    channel.send(42);
    assert_eq!(receiver.recv().await, Some(42));
    assert_eq!(channel.most_recent_sent_value(), Some(42));

    // Test sending multiple values, receiver only gets the last one
    channel.send(1);
    channel.send(2);
    channel.send(3);
    assert_eq!(receiver.recv().await, Some(3));
    assert_eq!(channel.most_recent_sent_value(), Some(3));

    // Test sending the same value again
    channel.send(3);
    assert_eq!(receiver.recv().await, Some(3));
    assert_eq!(channel.most_recent_sent_value(), Some(3));
}

#[tokio::test]
async fn test_fire_and_forget_channel_only_changes_config() {
    let mut channel = fire_and_forget_channel::<i32, OnlyChangesImmediateSend<_>>();
    let mut receiver = channel.subscribe();

    // Test sending a new value
    assert_eq!(channel.send(42), Some(1));
    assert_eq!(receiver.recv().await, Some(42));
    assert_eq!(channel.most_recent_sent_value(), Some(42));

    // Test sending the same value again (should not send)
    assert_eq!(channel.send(42), None);
    assert_eq!(channel.most_recent_sent_value(), Some(42));

    // Test sending a different value
    assert_eq!(channel.send(43), Some(1));
    assert_eq!(receiver.recv().await, Some(43));
    assert_eq!(channel.most_recent_sent_value(), Some(43));
}

#[tokio::test]
async fn test_channel_closing() {
    let mut channel = fire_and_forget_channel::<i32, DefaultAlwaysImmediateSend<_>>();
    let mut receiver = channel.subscribe();

    channel.send(1);
    assert_eq!(receiver.recv().await, Some(1));

    // Drop the channel (sender)
    drop(channel);

    // Receiver should now get None
    assert_eq!(receiver.recv().await, None);
}

#[tokio::test]
async fn test_multiple_receivers() {
    let mut channel = fire_and_forget_channel::<i32, DefaultAlwaysImmediateSend<_>>();
    let mut receiver1 = channel.subscribe();
    let mut receiver2 = channel.subscribe();

    channel.send(10);

    assert_eq!(receiver1.recv().await, Some(10));
    assert_eq!(receiver2.recv().await, Some(10));
}

#[tokio::test]
async fn test_fire_and_forget_channel_rate_limited() {
    env_logger::builder()
        .filter_level(log::LevelFilter::Trace)
        .default_format()
        .init();

    let wait_increment = chrono::Duration::milliseconds(100);
    let wait_max = chrono::Duration::milliseconds(800);
    let duration_to_count_over = chrono::Duration::seconds(1);

    let mut channel = fire_and_forget_channel_with(RateLimitedMostRecentSend::<i32>::new(
        "test".to_owned(),
        wait_increment,
        wait_max,
        duration_to_count_over,
    ));
    let mut receiver = channel.subscribe();

    // Test 1: Single send
    let start = tokio::time::Instant::now();
    channel.send(1);
    assert_eq!(receiver.recv().await, Some(1));
    let elapsed = start.elapsed();
    assert!(
        elapsed >= wait_increment.to_std().unwrap(),
        "Elapsed: {:?}",
        elapsed
    );

    // Test 2: Multiple sends, only last one is received
    tokio::time::sleep(duration_to_count_over.to_std().unwrap()).await; // reset counter
    let start = tokio::time::Instant::now();
    channel.send(2); // causes wait_increment * 1^2 = 100ms
    assert_eq!(receiver.recv().await, Some(2));
    let elapsed = start.elapsed();
    channel.send(3); // causes second send with wait_inc * 2^2 = 400ms
    let expected_wait = wait_increment.to_std().unwrap();
    assert!(
        elapsed >= expected_wait,
        "Elapsed: {:?}, Expected: {:?}",
        elapsed,
        expected_wait
    );

    assert_eq!(receiver.recv().await, Some(3));
    let elapsed = start.elapsed();
    let expected_wait = wait_increment.to_std().unwrap() * 4;
    assert!(
        elapsed >= expected_wait,
        "Elapsed: {:?}, Expected: {:?}",
        elapsed,
        expected_wait
    );

    // Test 3: Wait time does not exceed wait_max
    tokio::time::sleep(duration_to_count_over.to_std().unwrap()).await; // reset counter
    for i in 4..=9 {
        channel.send(i);
    }
    assert_eq!(receiver.recv().await, Some(9));
    let start = tokio::time::Instant::now();
    channel.send(10);
    assert_eq!(receiver.recv().await, Some(10));
    let elapsed = start.elapsed();
    let min = wait_max.to_std().unwrap();
    let max = wait_max.to_std().unwrap().as_secs_f64() * 1.05;
    assert!(
        elapsed >= min && elapsed.as_secs_f64() < max,
        "Elapsed: {:?}, Elapsed: {:?}(s), Min: {min:?}, Max: {max:?}",
        elapsed,
        elapsed.as_secs_f64()
    );

    // // Test 4: most_recent_sent_value is updated after send
    tokio::time::sleep(duration_to_count_over.to_std().unwrap()).await; // reset counter
    assert_eq!(channel.most_recent_sent_value(), Some(10)); // from previous test
    channel.send(11);
    assert_eq!(channel.most_recent_sent_value(), Some(10)); // still old value
    tokio::time::sleep(wait_increment.to_std().unwrap() * 2).await;
    assert_eq!(channel.most_recent_sent_value(), Some(11));
}
