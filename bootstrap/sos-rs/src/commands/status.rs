use anyhow::Result;
use std::fs;

pub fn run() -> Result<()> {
    match fs::read_to_string(".sos/state.toml") {
        Ok(raw) => {
            println!("─── .sos/state.toml ───");
            print!("{raw}");
            Ok(())
        }
        Err(_) => {
            println!("No .sos/state.toml — this isn't a Genesis-managed project (or run 'sos init').");
            Ok(())
        }
    }
}
