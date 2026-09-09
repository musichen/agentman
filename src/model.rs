use std::{collections::BTreeSet, path::PathBuf, time::SystemTime};

use fuzzy_matcher::{FuzzyMatcher, skim::SkimMatcherV2};

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum AgentKind {
    Codex,
    ClaudeCode,
    OpenClaude,
    Pi,
    Codewhale,
    Dsh,
    Acryl,
}

impl AgentKind {
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Codex => "Codex",
            Self::ClaudeCode => "Claude Code",
            Self::OpenClaude => "OpenClaude",
            Self::Pi => "Pi",
            Self::Codewhale => "Codewhale",
            Self::Dsh => "DSH",
            Self::Acryl => "ACRYL",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum Capability {
    Resume,
    Fork,
    YoloResume,
    YoloFork,
    Rename,
    Trash,
    ReadOnly,
}

#[derive(Clone, Debug)]
pub struct Session {
    pub agent: AgentKind,
    pub id: String,
    pub title: String,
    pub project: Option<PathBuf>,
    pub path: PathBuf,
    pub modified: SystemTime,
    pub capabilities: BTreeSet<Capability>,
    pub diagnostic: Option<String>,
}

impl Session {
    #[must_use]
    pub fn new(
        agent: AgentKind,
        id: impl Into<String>,
        title: impl Into<String>,
        project: Option<PathBuf>,
        path: PathBuf,
        capabilities: impl IntoIterator<Item = Capability>,
    ) -> Self {
        Self {
            agent,
            id: id.into(),
            title: title.into(),
            project,
            path,
            modified: SystemTime::UNIX_EPOCH,
            capabilities: capabilities.into_iter().collect(),
            diagnostic: None,
        }
    }
}

#[must_use]
pub fn ranked_sessions(sessions: &[Session], query: &str) -> Vec<usize> {
    let query = query.trim();
    if query.is_empty() {
        return (0..sessions.len()).collect();
    }
    let matcher = SkimMatcherV2::default();
    let mut matches: Vec<(usize, i64)> = sessions
        .iter()
        .enumerate()
        .filter_map(|(index, session)| {
            let project = session
                .project
                .as_ref()
                .map_or_else(String::new, |path| path.to_string_lossy().into_owned());
            let haystack = format!("{} {} {}", session.title, session.id, project);
            matcher.fuzzy_match(&haystack, query).map(|score| (index, score))
        })
        .collect();
    matches.sort_by(|left, right| right.1.cmp(&left.1).then(left.0.cmp(&right.0)));
    matches.into_iter().map(|(index, _)| index).collect()
}
