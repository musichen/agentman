use std::path::PathBuf;

use agentman::{AgentKind, Capability, Session, ranked_sessions};

fn session(title: &str, id: &str, project: &str) -> Session {
    Session::new(
        AgentKind::Codex,
        id,
        title,
        Some(PathBuf::from(project)),
        PathBuf::from("/tmp/session.jsonl"),
        [Capability::Resume],
    )
}

#[test]
fn fuzzy_search_matches_title_id_and_project() {
    let sessions = vec![session("Fix session picker", "abc-123", "/work/acryl")];
    assert_eq!(ranked_sessions(&sessions, "picker"), vec![0]);
    assert_eq!(ranked_sessions(&sessions, "abc"), vec![0]);
    assert_eq!(ranked_sessions(&sessions, "acryl"), vec![0]);
}

#[test]
fn blank_query_preserves_session_order() {
    let sessions = vec![
        session("First", "one", "/work/one"),
        session("Second", "two", "/work/two"),
    ];
    assert_eq!(ranked_sessions(&sessions, ""), vec![0, 1]);
}
