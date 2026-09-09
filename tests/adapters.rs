use std::fs;

use agentman::{AgentKind, discover_all};
use tempfile::tempdir;

#[test]
fn discovers_all_agent_roots_from_metadata_only() {
    let home = tempdir().unwrap();
    let cases = [
        (".codex/sessions/a.jsonl", r#"{"type":"session_meta","payload":{"id":"codex-1","title":"Codex title","cwd":"/work/codex"}}"#),
        (".claude/projects/project/claude-1.jsonl", r#"{"type":"summary","sessionId":"claude-1","summary":"Claude title","cwd":"/work/claude"}"#),
        (".openclaude/projects/project/open-1.replay.json", r#"{"sessionId":"open-1","summary":"Open title","cwd":"/work/open"}"#),
        (".pi/agent/sessions/project/pi-1.jsonl", r#"{"type":"session","id":"pi-1","cwd":"/work/pi","name":"Pi title"}"#),
        (".codewhale/sessions/whale-1.json", r#"{"metadata":{"id":"whale-1","title":"Whale title","cwd":"/work/whale"}}"#),
        (".dsh/sessions/project/dsh-1.json", r#"{"id":"dsh-1","title":"DSH title","cwd":"/work/dsh"}"#),
        (".acryl/agent/sessions/acryl-1.jsonl", r#"{"type":"session","id":"acryl-1","cwd":"/work/acryl","name":"ACRYL title"}"#),
    ];
    for (relative, content) in cases {
        let path = home.path().join(relative);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, content).unwrap();
    }

    let sessions = discover_all(home.path()).unwrap();
    assert_eq!(sessions.len(), 7);
    assert!(sessions.iter().any(|session| session.agent == AgentKind::Codex && session.id == "codex-1"));
    assert!(sessions.iter().any(|session| session.agent == AgentKind::Acryl && session.title == "ACRYL title"));
}
