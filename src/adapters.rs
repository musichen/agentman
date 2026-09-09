use std::{fs, path::{Path, PathBuf}, time::SystemTime};

use anyhow::Result;
use serde_json::Value;

use crate::{AgentKind, Capability, Session};

const ROOTS: &[(AgentKind, &str)] = &[
    (AgentKind::Codex, ".codex/sessions"),
    (AgentKind::ClaudeCode, ".claude/projects"),
    (AgentKind::OpenClaude, ".openclaude/projects"),
    (AgentKind::Pi, ".pi/agent/sessions"),
    (AgentKind::Codewhale, ".codewhale/sessions"),
    (AgentKind::Dsh, ".dsh/sessions"),
    (AgentKind::Acryl, ".acryl/agent/sessions"),
    (AgentKind::Acryl, ".acryl/.dsh/sessions"),
];

pub fn discover_all(home: &Path) -> Result<Vec<Session>> {
    let mut sessions = Vec::new();
    for &(agent, relative) in ROOTS {
        let root = home.join(relative);
        if root.is_dir() {
            discover_root(agent, &root, &mut sessions)?;
        }
    }
    sessions.sort_by(|left, right| right.modified.cmp(&left.modified));
    Ok(sessions)
}

fn discover_root(agent: AgentKind, root: &Path, sessions: &mut Vec<Session>) -> Result<()> {
    for entry in fs::read_dir(root)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            discover_root(agent, &path, sessions)?;
        } else if matches!(path.extension().and_then(|extension| extension.to_str()), Some("json" | "jsonl")) {
            sessions.push(parse_session(agent, path));
        }
    }
    Ok(())
}

fn parse_session(agent: AgentKind, path: PathBuf) -> Session {
    let modified = fs::metadata(&path).and_then(|meta| meta.modified()).unwrap_or(SystemTime::UNIX_EPOCH);
    let fallback_id = path.file_stem().and_then(|name| name.to_str()).unwrap_or("unknown").to_owned();
    let parsed = fs::read_to_string(&path)
        .ok()
        .and_then(|source| source.lines().find_map(|line| serde_json::from_str::<Value>(line).ok()));
    let Some(value) = parsed else {
        let mut session = Session::new(agent, fallback_id, "Unreadable session", None, path, [Capability::ReadOnly, Capability::Trash]);
        session.modified = modified;
        session.diagnostic = Some("Could not read session metadata".to_owned());
        return session;
    };
    let id = find_string(&value, &["sessionId", "session_id", "id"]).unwrap_or(fallback_id);
    let title = find_string(&value, &["title", "summary", "name"]).unwrap_or_else(|| format!("{} session", agent.label()));
    let project = find_string(&value, &["cwd", "project", "working_directory"]).map(PathBuf::from);
    let capabilities = capabilities(agent);
    let mut session = Session::new(agent, id, title, project, path, capabilities);
    session.modified = modified;
    session
}

fn find_string(value: &Value, keys: &[&str]) -> Option<String> {
    match value {
        Value::Object(map) => {
            for key in keys {
                if let Some(Value::String(value)) = map.get(*key) {
                    return Some(value.clone());
                }
            }
            map.values().find_map(|value| find_string(value, keys))
        }
        Value::Array(values) => values.iter().find_map(|value| find_string(value, keys)),
        _ => None,
    }
}

fn capabilities(agent: AgentKind) -> Vec<Capability> {
    let mut result = vec![Capability::Rename, Capability::Trash, Capability::Resume];
    match agent {
        AgentKind::Codex | AgentKind::OpenClaude => result.extend([Capability::Fork, Capability::YoloResume, Capability::YoloFork]),
        AgentKind::ClaudeCode => result.extend([Capability::YoloResume]),
        AgentKind::Pi | AgentKind::Codewhale => result.push(Capability::Fork),
        AgentKind::Dsh | AgentKind::Acryl => {}
    }
    result
}
