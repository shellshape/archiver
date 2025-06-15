use anyhow::Result;
use chrono::{DateTime, Datelike};
use clap::{Parser, command};
use std::fs;
use std::path::PathBuf;
use std::time::UNIX_EPOCH;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    source_dir: PathBuf,
    target_dir: PathBuf,

    #[arg(long)]
    mv: bool,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    for entry in fs::read_dir(cli.source_dir)? {
        let entry = entry?;

        let mtime = DateTime::from_timestamp_micros(
            entry
                .metadata()?
                .modified()?
                .duration_since(UNIX_EPOCH)?
                .as_micros() as i64,
        )
        .ok_or_else(|| anyhow::anyhow!("could not get mtime as chrono::DateTime"))?;

        println!("{}/{}/{}", mtime.year(), mtime.month(), mtime.day());
    }

    Ok(())
}
