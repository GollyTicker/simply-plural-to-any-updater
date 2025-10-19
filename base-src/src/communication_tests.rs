use crate::communication::{DefaultAlwaysImmediateSend, OnlyChangesImmediateSend, fire_and_forget_channel};

#[tokio::test]
async fn test_fire_and_forget_channel_default_config() {
    let mut channel = fire_and_forget_channel::<i32, DefaultAlwaysImmediateSend>();
    let mut receiver = channel.subscribe();

    // Test sending a value
    channel.send(42);
    assert_eq!(receiver.recv().await, Some(42));
    assert_eq!(channel.most_recent_value, Some(42));

    // Test sending multiple values, receiver only gets the last one
    channel.send(1);
    channel.send(2);
    channel.send(3);
    assert_eq!(receiver.recv().await, Some(3));
    assert_eq!(channel.most_recent_value, Some(3));

    // Test sending the same value again
    channel.send(3);
    assert_eq!(receiver.recv().await, Some(3));
    assert_eq!(channel.most_recent_value, Some(3));
}

#[tokio::test]
async fn test_fire_and_forget_channel_only_changes_config() {
    let mut channel = fire_and_forget_channel::<i32, OnlyChangesImmediateSend>();
    let mut receiver = channel.subscribe();

    // Test sending a new value
    assert_eq!(channel.send(42), Some(1));
    assert_eq!(receiver.recv().await, Some(42));
    assert_eq!(channel.most_recent_value, Some(42));

    // Test sending the same value again (should not send)
    assert_eq!(channel.send(42), None);
    assert_eq!(channel.most_recent_value, Some(42));

    // Test sending a different value
    assert_eq!(channel.send(43), Some(1));
    assert_eq!(receiver.recv().await, Some(43));
    assert_eq!(channel.most_recent_value, Some(43));
}

#[tokio::test]
async fn test_channel_closing() {
    let mut channel = fire_and_forget_channel::<i32, DefaultAlwaysImmediateSend>();
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
    let mut channel = fire_and_forget_channel::<i32, DefaultAlwaysImmediateSend>();
    let mut receiver1 = channel.subscribe();
    let mut receiver2 = channel.subscribe();

    channel.send(10);

    assert_eq!(receiver1.recv().await, Some(10));
    assert_eq!(receiver2.recv().await, Some(10));
}
