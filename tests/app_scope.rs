use agentman::app::App;
use agentman::{AgentKind, Capability, Session};

fn session(agent: AgentKind, title: &str) -> Session {
    Session::new(
        agent,
        title,
        title,
        None,
        "/tmp/x".into(),
        [Capability::Resume],
    )
}

#[test]
fn selecting_an_agent_limits_the_session_domain() {
    let mut app = App::new(vec![
        session(AgentKind::Codex, "Codex one"),
        session(AgentKind::Pi, "Pi one"),
    ]);
    app.select_agent(Some(AgentKind::Codex));
    assert_eq!(app.visible_titles(), vec!["Codex one"]);
    app.select_agent(Some(AgentKind::Pi));
    assert_eq!(app.visible_titles(), vec!["Pi one"]);
}
