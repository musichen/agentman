use std::{
    fs,
    io::Write,
    path::{Path, PathBuf},
    time::SystemTime,
};

use anyhow::Result;
use serde_json::Value;

use crate::{AgentKind, Capability, Session};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SessionAction {
    Resume,
    Fork,
    YoloResume,
    YoloFork,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CommandSpec {
    pub program: String,
    pub args: Vec<String>,
}

#[must_use]
pub fn launch_command(session: &Session, action: SessionAction) -> Option<CommandSpec> {
    let id = &session.id;
    let (program, mut args) = match (session.agent, action) {
        (AgentKind::Codex, SessionAction::Resume) => ("codex", vec!["resume".into(), id.clone()]),
        (AgentKind::Codex, SessionAction::Fork) => ("codex", vec!["fork".into(), id.clone()]),
        (AgentKind::Codex, SessionAction::YoloResume) => (
            "codex",
            vec![
                "resume".into(),
                "--dangerously-bypass-approvals-and-sandbox".into(),
                id.clone(),
            ],
        ),
        (AgentKind::Codex, SessionAction::YoloFork) => (
            "codex",
            vec![
                "fork".into(),
                "--dangerously-bypass-approvals-and-sandbox".into(),
                id.clone(),
            ],
        ),
        (AgentKind::ClaudeCode, SessionAction::Resume) => {
            ("claude", vec!["--resume".into(), id.clone()])
        }
        (AgentKind::ClaudeCode, SessionAction::YoloResume) => (
            "claude",
            vec![
                "--resume".into(),
                id.clone(),
                "--dangerously-skip-permissions".into(),
            ],
        ),
        (AgentKind::OpenClaude, SessionAction::Resume) => {
            ("openclaude", vec!["--resume".into(), id.clone()])
        }
        (AgentKind::OpenClaude, SessionAction::Fork) => (
            "openclaude",
            vec!["--resume".into(), id.clone(), "--fork-session".into()],
        ),
        (AgentKind::OpenClaude, SessionAction::YoloResume) => (
            "openclaude",
            vec!["--resume".into(), id.clone(), "--yolo".into()],
        ),
        (AgentKind::OpenClaude, SessionAction::YoloFork) => (
            "openclaude",
            vec![
                "--resume".into(),
                id.clone(),
                "--fork-session".into(),
                "--yolo".into(),
            ],
        ),
        (AgentKind::Pi, SessionAction::Resume) => ("pi", vec!["--session".into(), id.clone()]),
        (AgentKind::Pi, SessionAction::Fork) => ("pi", vec!["--fork".into(), id.clone()]),
        (AgentKind::Codewhale, SessionAction::Resume) => {
            ("codewhale", vec!["resume".into(), id.clone()])
        }
        (AgentKind::Codewhale, SessionAction::Fork) => {
            ("codewhale", vec!["fork".into(), id.clone()])
        }
        (AgentKind::Acryl, SessionAction::Resume) => ("acryl", vec!["--resume".into(), id.clone()]),
        _ => return None,
    };
    Some(CommandSpec {
        program: program.into(),
        args: std::mem::take(&mut args),
    })
}

/// Rename the first recognized native title field atomically.
///
/// # Errors
/// Returns an error when the file cannot be read/written or no title field is recognized.
pub fn rename_session(session: &Session, new_title: &str) -> Result<()> {
    let source = fs::read_to_string(&session.path)?;
    let mut changed = false;
    let rewritten = source
        .lines()
        .map(|line| {
            let Ok(mut value) = serde_json::from_str::<Value>(line) else {
                return line.to_owned();
            };
            if !changed && set_title(&mut value, new_title) {
                changed = true;
                serde_json::to_string(&value).unwrap_or_else(|_| line.to_owned())
            } else {
                line.to_owned()
            }
        })
        .collect::<Vec<_>>()
        .join("\n");
    anyhow::ensure!(changed, "this session has no editable native title");
    let temporary = session.path.with_extension("agentman.tmp");
    let mut file = fs::File::create(&temporary)?;
    file.write_all(rewritten.as_bytes())?;
    file.sync_all()?;
    fs::rename(temporary, &session.path)?;
    Ok(())
}

fn set_title(value: &mut Value, new_title: &str) -> bool {
    let Value::Object(map) = value else {
        return false;
    };
    for key in ["title", "summary", "name"] {
        if let Some(Value::String(title)) = map.get_mut(key) {
            new_title.clone_into(title);
            return true;
        }
    }
    map.values_mut().any(|value| set_title(value, new_title))
}

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

/// Discover session records under all supported agent roots.
///
/// # Errors
/// Returns an error when a supported directory cannot be read.
pub fn discover_all(home: &Path) -> Result<Vec<Session>> {
    let mut sessions = Vec::new();
    for &(agent, relative) in ROOTS {
        let root = home.join(relative);
        if root.is_dir() {
            discover_root(agent, &root, &mut sessions)?;
        }
    }
    sessions.sort_by_key(|session| std::cmp::Reverse(session.modified));
    Ok(sessions)
}

fn discover_root(agent: AgentKind, root: &Path, sessions: &mut Vec<Session>) -> Result<()> {
    for entry in fs::read_dir(root)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            discover_root(agent, &path, sessions)?;
        } else if matches!(
            path.extension().and_then(|extension| extension.to_str()),
            Some("json" | "jsonl")
        ) {
            sessions.push(parse_session(agent, path));
        }
    }
    Ok(())
}

fn parse_session(agent: AgentKind, path: PathBuf) -> Session {
    let modified = fs::metadata(&path)
        .and_then(|meta| meta.modified())
        .unwrap_or(SystemTime::UNIX_EPOCH);
    let fallback_id = path
        .file_stem()
        .and_then(|name| name.to_str())
        .unwrap_or("unknown")
        .to_owned();
    let parsed = fs::read_to_string(&path).ok().and_then(|source| {
        source
            .lines()
            .find_map(|line| serde_json::from_str::<Value>(line).ok())
    });
    let Some(value) = parsed else {
        let mut session = Session::new(
            agent,
            fallback_id,
            "Unreadable session",
            None,
            path,
            [Capability::ReadOnly, Capability::Trash],
        );
        session.modified = modified;
        session.diagnostic = Some("Could not read session metadata".to_owned());
        return session;
    };
    let id = find_string(&value, &["sessionId", "session_id", "id"]).unwrap_or(fallback_id);
    let title = find_string(&value, &["title", "summary", "name"])
        .unwrap_or_else(|| format!("{} session", agent.label()));
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
        AgentKind::Codex | AgentKind::OpenClaude => result.extend([
            Capability::Fork,
            Capability::YoloResume,
            Capability::YoloFork,
        ]),
        AgentKind::ClaudeCode => result.extend([Capability::YoloResume]),
        AgentKind::Pi | AgentKind::Codewhale => result.push(Capability::Fork),
        AgentKind::Dsh | AgentKind::Acryl => {}
    }
    result
}
