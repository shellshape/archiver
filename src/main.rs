use crate::dirtree::DirTree;
use crate::errors::Error;
use anyhow::Result;
use chrono::{DateTime, Datelike};
use clap::{Parser, command};
use console::Term;
use std::fs::{self, DirEntry, File};
use std::io;
use std::path::{Path, PathBuf};
use std::time::UNIX_EPOCH;
use yansi::Paint;

mod dirtree;
mod errors;

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

    let mut dir_tree = DirTree::default();

    let mut failed = 0;

    for entry in fs::read_dir(cli.source_dir).map_err(Error::ReadingSourceDir)? {
        let entry = match entry {
            Ok(e) => e,
            Err(err) => {
                println!(
                    "{} failed getting directory entry: {}",
                    "error:".red().bold(),
                    err
                );
                failed += 1;
                continue;
            }
        };

        print!(
            "{} {} ...",
            "processing".cyan(),
            entry.file_name().to_string_lossy()
        );

        match process_entry(&mut dir_tree, &entry, &cli.target_dir, cli.mv) {
            Ok(_) => {
                Term::stdout().clear_line().ok();
                println!(
                    "\r{} {}",
                    "finished".bright_green(),
                    entry.file_name().to_string_lossy()
                );
            }
            Err(err) => {
                Term::stdout().clear_line().ok();
                println!(
                    "\r{} {} : {}",
                    "failed".red(),
                    entry.file_name().to_string_lossy(),
                    err
                );
                failed += 1;
            }
        }
    }

    if failed > 0 {
        return Err(anyhow::anyhow!("Processing failed for {failed} files."));
    }

    Ok(())
}

fn process_entry(
    dir_tree: &mut DirTree,
    entry: &DirEntry,
    target_dir: impl AsRef<Path>,
    mv: bool,
) -> Result<()> {
    let meta = entry.metadata().map_err(Error::Metadata)?;
    if meta.is_dir() {
        return Ok(());
    }

    let mtime = DateTime::from_timestamp_micros(
        meta.modified()
            .map_err(Error::ModifiedTime)?
            .duration_since(UNIX_EPOCH)
            .map_err(Error::ModifiedTimeDuration)?
            .as_micros() as i64,
    )
    .ok_or(Error::ModifiedTimeToChrono)?;

    let dir = target_dir
        .as_ref()
        .join(mtime.year().to_string())
        .join(mtime.month().to_string())
        .join(mtime.day().to_string());

    dir_tree
        .mkdir_all(&dir)
        .map_err(Error::CreatingTargetDirectory)?;

    let mut source_file = File::open(entry.path()).map_err(Error::OpeningSourceFile)?;
    let mut target_file =
        File::create(dir.join(entry.file_name())).map_err(Error::CreatingTargetFile)?;
    io::copy(&mut source_file, &mut target_file).map_err(Error::CopyingFileContents)?;

    if mv {
        fs::remove_file(entry.path()).map_err(Error::DeletingSourceFile)?;
    }

    Ok(())
}
