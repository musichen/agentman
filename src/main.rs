fn main() {
    if std::env::args().any(|argument| argument == "--version" || argument == "-V") {
        println!("agentman {}", env!("CARGO_PKG_VERSION"));
        return;
    }
    println!("agentman: interactive session management is starting up");
}
