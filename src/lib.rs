#![forbid(unsafe_code)]

mod model;
mod adapters;

pub use model::{AgentKind, Capability, Session, ranked_sessions};
pub use adapters::discover_all;
