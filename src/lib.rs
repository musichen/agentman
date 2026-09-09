#![forbid(unsafe_code)]

mod model;
mod adapters;
pub mod app;
pub mod ui;

pub use model::{AgentKind, Capability, Session, ranked_sessions};
pub use adapters::{CommandSpec, SessionAction, discover_all, launch_command, rename_session};
