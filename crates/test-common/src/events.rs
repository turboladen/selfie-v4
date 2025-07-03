//! Event stream processing helpers to eliminate duplication in async service tests.

use selfie::package::event::{EventStream, OperationResult, PackageEvent};

/// Collects all events from a stream for testing verification.
/// This is the most common pattern for testing event streams in service tests.
///
/// # Example
/// ```rust
/// let stream = service.check("test-package").await;
/// let events = collect_events(stream).await;
/// ```
pub async fn collect_events(mut stream: EventStream) -> Vec<PackageEvent> {
    let mut events = Vec::new();
    while let Some(event) = futures::StreamExt::next(&mut stream).await {
        events.push(event);
    }
    events
}

/// Extracts the operation result from collected events.
/// Returns the final result of the operation if found.
///
/// # Example
/// ```rust
/// let events = collect_events(stream).await;
/// let result = get_operation_result(&events);
/// assert!(matches!(result, Some(OperationResult::Success(_))));
/// ```
#[must_use]
pub fn get_operation_result(events: &[PackageEvent]) -> Option<&OperationResult> {
    for event in events {
        if let PackageEvent::Completed { result, .. } = event {
            return Some(result);
        }
    }
    None
}

/// Counts events of a specific type for verification.
/// Useful for asserting expected event sequences.
///
/// # Example
/// ```rust
/// let progress_count = count_events_of_type(&events, |e| {
///     matches!(e, PackageEvent::Progress { .. })
/// });
/// assert_eq!(progress_count, 3);
/// ```
pub fn count_events_of_type<F>(events: &[PackageEvent], predicate: F) -> usize
where
    F: Fn(&PackageEvent) -> bool,
{
    events.iter().filter(|e| predicate(e)).count()
}

/// Helper to verify standard event sequence for successful operations.
/// Checks for exactly one started event, at least one progress event,
/// exactly one completed event, and that the result is successful.
///
/// # Panics
/// Panics if the event sequence doesn't match expected successful operation pattern.
///
/// # Example
/// ```rust
/// let events = collect_events(stream).await;
/// assert_successful_operation(&events);
/// ```
pub fn assert_successful_operation(events: &[PackageEvent]) {
    // Should have exactly one started event
    assert_eq!(
        count_events_of_type(events, |e| matches!(e, PackageEvent::Started { .. })),
        1,
        "Should have exactly one started event"
    );

    // Should have at least one progress event
    assert!(
        count_events_of_type(events, |e| matches!(e, PackageEvent::Progress { .. })) > 0,
        "Should have at least one progress event"
    );

    // Should have exactly one completed event
    assert_eq!(
        count_events_of_type(events, |e| matches!(e, PackageEvent::Completed { .. })),
        1,
        "Should have exactly one completed event"
    );

    // Result should be successful
    let result = get_operation_result(events).expect("Should have an operation result");
    assert!(
        matches!(result, OperationResult::Success(_)),
        "Operation should be successful, got: {result:?}"
    );
}

/// Helper to verify standard event sequence for failed operations.
/// Checks for at least one error event and that the final result (if present) is a failure.
///
/// # Panics
/// Panics if the event sequence doesn't match expected failed operation pattern.
///
/// # Example
/// ```rust
/// let events = collect_events(stream).await;
/// assert_failed_operation(&events);
/// ```
pub fn assert_failed_operation(events: &[PackageEvent]) {
    // Should have at least one error event
    assert!(
        count_events_of_type(events, |e| matches!(e, PackageEvent::Error { .. })) > 0,
        "Should have at least one error event for failed operation"
    );

    // If there's a completion result, it should be a failure
    if let Some(result) = get_operation_result(events) {
        assert!(
            matches!(result, OperationResult::Failure(_)),
            "Operation result should be failure, got: {result:?}"
        );
    }
}

/// Helper to verify that events contain specific progress steps.
/// Useful for testing that operations report expected progress stages.
///
/// # Example
/// ```rust
/// let events = collect_events(stream).await;
/// assert_has_progress_steps(&events, &["Loading package", "Running check", "Complete"]);
/// ```
pub fn assert_has_progress_steps(events: &[PackageEvent], expected_steps: &[&str]) {
    let progress_messages: Vec<String> = events
        .iter()
        .filter_map(|e| {
            if let PackageEvent::Progress { message, .. } = e {
                Some(message.clone())
            } else {
                None
            }
        })
        .collect();

    for expected_step in expected_steps {
        assert!(
            progress_messages
                .iter()
                .any(|msg| msg.contains(expected_step)),
            "Expected progress step '{expected_step}' not found in messages: {progress_messages:?}"
        );
    }
}

/// Helper to extract all error messages from events.
/// Useful for testing specific error conditions and messages.
///
/// # Example
/// ```rust
/// let events = collect_events(stream).await;
/// let errors = get_error_messages(&events);
/// assert!(errors.iter().any(|msg| msg.contains("Package not found")));
/// ```
#[must_use]
pub fn get_error_messages(events: &[PackageEvent]) -> Vec<String> {
    events
        .iter()
        .filter_map(|e| {
            if let PackageEvent::Error { message, .. } = e {
                Some(message.clone())
            } else {
                None
            }
        })
        .collect()
}

/// Helper to assert that no error events occurred.
/// Useful for testing that operations completed cleanly without any errors.
///
/// # Example
/// ```rust
/// let events = collect_events(stream).await;
/// assert_no_errors(&events);
/// ```
pub fn assert_no_errors(events: &[PackageEvent]) {
    let error_count = count_events_of_type(events, |e| matches!(e, PackageEvent::Error { .. }));
    assert_eq!(
        error_count,
        0,
        "Expected no error events, but found {} errors: {:?}",
        error_count,
        get_error_messages(events)
    );
}
