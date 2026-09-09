#![forbid(unsafe_code)]

mod adapters;
pub mod app;
mod model;
pub mod ui;

pub use adapters::{CommandSpec, SessionAction, discover_all, launch_command, rename_session};
pub use model::{AgentKind, Capability, Session, ranked_sessions};
