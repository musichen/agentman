use agentman::update_message;

#[test]
fn update_message_reports_newer_release() {
    assert_eq!(
        update_message("0.1.1", "0.1.2"),
        "Update available: agentman 0.1.1 -> 0.1.2"
    );
}

#[test]
fn update_message_reports_current_release() {
    assert_eq!(
        update_message("0.1.1", "0.1.1"),
        "agentman 0.1.1 is up to date."
    );
}
