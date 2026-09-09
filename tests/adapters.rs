use std::fs;

use agentman::{
    AgentKind, Capability, Session, SessionAction, discover_all, launch_command, rename_session,
};
use tempfile::tempdir;

#[test]
fn discovers_all_agent_roots_from_metadata_only() {
    let home = tempdir().unwrap();
    let cases = [
        (
            ".codex/sessions/a.jsonl",
            r#"{"type":"session_meta","payload":{"id":"codex-1","title":"Codex title","cwd":"/work/codex"}}"#,
        ),
        (
            ".claude/projects/project/claude-1.jsonl",
            r#"{"type":"summary","sessionId":"claude-1","summary":"Claude title","cwd":"/work/claude"}"#,
        ),
        (
            ".openclaude/projects/project/open-1.replay.json",
            r#"{"sessionId":"open-1","summary":"Open title","cwd":"/work/open"}"#,
        ),
        (
            ".pi/agent/sessions/project/pi-1.jsonl",
            r#"{"type":"session","id":"pi-1","cwd":"/work/pi","name":"Pi title"}"#,
        ),
        (
            ".codewhale/sessions/whale-1.json",
            r#"{"metadata":{"id":"whale-1","title":"Whale title","cwd":"/work/whale"}}"#,
        ),
        (
            ".dsh/sessions/project/dsh-1.json",
            r#"{"id":"dsh-1","title":"DSH title","cwd":"/work/dsh"}"#,
        ),
        (
            ".acryl/agent/sessions/acryl-1.jsonl",
            r#"{"type":"session","id":"acryl-1","cwd":"/work/acryl","name":"ACRYL title"}"#,
        ),
    ];
    for (relative, content) in cases {
        let path = home.path().join(relative);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, content).unwrap();
    }

    let sessions = discover_all(home.path()).unwrap();
    assert_eq!(sessions.len(), 7);
    assert!(
        sessions
            .iter()
            .any(|session| session.agent == AgentKind::Codex && session.id == "codex-1")
    );
    assert!(
        sessions
            .iter()
            .any(|session| session.agent == AgentKind::Acryl && session.title == "ACRYL title")
    );
}

#[test]
fn codex_yolo_fork_uses_the_real_subcommand_and_flag() {
    let session = Session::new(
        AgentKind::Codex,
        "id",
        "title",
        None,
        "/tmp/x".into(),
        [Capability::Fork],
    );
    let command = launch_command(&session, SessionAction::YoloFork).unwrap();
    assert_eq!(command.program, "codex");
    assert_eq!(
        command.args,
        ["fork", "--dangerously-bypass-approvals-and-sandbox", "id"]
    );
}

#[test]
fn openclaude_yolo_fork_uses_its_documented_flags() {
    let session = Session::new(
        AgentKind::OpenClaude,
        "id",
        "title",
        None,
        "/tmp/x".into(),
        [Capability::Fork],
    );
    let command = launch_command(&session, SessionAction::YoloFork).unwrap();
    assert_eq!(command.args, ["--resume", "id", "--fork-session", "--yolo"]);
}

#[test]
fn rename_changes_metadata_without_touching_message_content() {
    let directory = tempdir().unwrap();
    let path = directory.path().join("session.json");
    fs::write(
        &path,
        r#"{"title":"Old","messages":[{"content":"do not touch"}]}"#,
    )
    .unwrap();
    let session = Session::new(
        AgentKind::Codewhale,
        "id",
        "Old",
        None,
        path.clone(),
        [Capability::Rename],
    );
    rename_session(&session, "New title").unwrap();
    let changed: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(path).unwrap()).unwrap();
    assert_eq!(changed["title"], "New title");
    assert_eq!(changed["messages"][0]["content"], "do not touch");
}
