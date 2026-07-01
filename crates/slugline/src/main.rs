mod app;
mod cli;
mod keys;
mod ui;

use clap::Parser;
use iced::Task;

use slugline_core::config::{default_config_path, load_or_create};
use slugline_core::dates::today_iso;
use slugline_core::store::{NotesStore, ensure_writable_dir};

use crate::app::App;
use crate::cli::{Cli, resolve};

pub fn main() -> iced::Result {
    let cli = Cli::parse();

    let config_path = cli.config.clone().unwrap_or_else(default_config_path);
    let config = load_or_create(&config_path).unwrap_or_else(|e| {
        eprintln!("Failed to load config {}: {e}", config_path.display());
        std::process::exit(1);
    });

    let resolved = resolve(&cli, &config);
    if let Err(e) = ensure_writable_dir(&resolved.notes_dir) {
        eprintln!(
            "Notes directory {} is not usable: {e}",
            resolved.notes_dir.display()
        );
        std::process::exit(1);
    }

    let store = NotesStore::new(resolved.notes_dir);
    let date = today_iso();

    iced::application(App::title, App::update, App::view)
        .subscription(App::subscription)
        .run_with(move || (App::new(store.clone(), date.clone()), Task::none()))
}
