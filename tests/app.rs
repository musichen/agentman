use agentman::app::App;
use agentman::{AgentKind, Capability, Session};

fn session(title: &str) -> Session {
    Session::new(
        AgentKind::Codex,
        title,
        title,
        None,
        "/tmp/x".into(),
        [Capability::Resume],
    )
}

#[test]
fn slash_enters_search_and_filters_the_current_agent() {
    let mut app = App::new(vec![session("Build agentman"), session("Fix docs")]);
    app.key('/');
    app.key('a');
    assert!(app.searching());
    assert_eq!(app.visible_titles(), vec!["Build agentman"]);
}

#[test]
fn delete_needs_explicit_confirmation_before_emitting_trash_action() {
    let mut app = App::new(vec![session("Trash me")]);
    app.key('d');
    assert!(app.confirming_delete());
    assert!(app.take_action().is_none());
    app.key('y');
    assert!(app.take_action().is_some());
}
