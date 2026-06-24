use std::net::Ipv4Addr;
use std::process;
use std::sync::Arc;

use clap::Parser;

use slugline::app::{build_router, AppState};
use slugline::cli::{resolve, Cli};
use slugline::config::{default_config_path, load_or_create};
use slugline::store::{ensure_writable_dir, NotesStore};

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    let config_path = cli.config.clone().unwrap_or_else(default_config_path);
    let config = match load_or_create(&config_path) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Failed to load config {}: {e}", config_path.display());
            process::exit(1);
        }
    };

    let resolved = resolve(&cli, &config);

    if let Err(e) = ensure_writable_dir(&resolved.notes_dir) {
        eprintln!(
            "Notes directory {} is not usable: {e}",
            resolved.notes_dir.display()
        );
        process::exit(1);
    }

    let state = Arc::new(AppState {
        store: NotesStore::new(resolved.notes_dir.clone()),
        config_path: config_path.clone(),
    });
    let app = build_router(state);

    let listener = match tokio::net::TcpListener::bind((Ipv4Addr::LOCALHOST, resolved.port)).await {
        Ok(l) => l,
        Err(e) => {
            eprintln!("Failed to bind 127.0.0.1:{}: {e}", resolved.port);
            process::exit(1);
        }
    };

    let url = format!("http://127.0.0.1:{}", resolved.port);
    println!(
        "Slugline serving at {url}  (notes: {})",
        resolved.notes_dir.display()
    );

    if resolved.auto_open {
        let _ = open::that(&url);
    }

    if let Err(e) = axum::serve(listener, app).await {
        eprintln!("Server error: {e}");
        process::exit(1);
    }
}
