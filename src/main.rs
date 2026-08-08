use clap::Parser;
use gitnest::cli::{handle_command, Cli};
use gitnest::config::ConfigManager;
use gitnest::logger::init_logger;
use std::process::exit;

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    let config_mgr = match ConfigManager::new() {
        Ok(mgr) => mgr,
        Err(e) => {
            eprintln!("Error initializing config manager: {}", e);
            exit(1);
        }
    };

    if config_mgr.is_initialized() {
        if let Ok(cfg) = config_mgr.load_config() {
            let _ = init_logger(&config_mgr.logs_dir(), &cfg.log_level);
        }
    }

    match cli.command {
        Some(cmd) => {
            if let Err(err) = handle_command(cmd, config_mgr).await {
                eprintln!("\nError: {}\n", err);
                exit(1);
            }
        }
        None => {
            if let Err(e) = gitnest::ui::run_tui_dashboard(&config_mgr).await {
                eprintln!("Error running TUI dashboard: {}", e);
                gitnest::cli::handlers::render_dashboard(&config_mgr).await;
            }
        }
    }
}
