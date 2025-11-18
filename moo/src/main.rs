mod engine;
mod games;
mod ui;

use anyhow::Result;
use engine::EngineApp;
use engine::core::EngineConfig;
use games::SandboxGame;

fn main() -> Result<()> {
    init_tracing();

    let config = EngineConfig::default();
    let app = EngineApp::new(config, SandboxGame::new());
    app.run()
}

fn init_tracing() {
    let subscriber = tracing_subscriber::FmtSubscriber::builder()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .finish();

    if tracing::subscriber::set_global_default(subscriber).is_err() {
        eprintln!("tracing subscriber already set");
    }
}
