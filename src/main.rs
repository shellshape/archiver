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
    /// Source directory
    source_dir: PathBuf,

    /// Target directory
    target_dir: PathBuf,

    /// Move files instead of copying them
    #[arg(short, long)]
    mv: bool,

    /// Overwrite existing files
    #[arg(short, long)]
    force: bool,
}

enum Status {
    SkippedExists,
    SkippedIsDir,
    Transferrred,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    let mut dir_tree = DirTree::new(&cli.target_dir);

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

        match process_entry(&mut dir_tree, &entry, &cli.target_dir, cli.mv, cli.force) {
            Ok(Status::Transferrred) => {
                Term::stdout().clear_line().ok();
                println!(
                    "\r{} {}",
                    "finished".bright_green(),
                    entry.file_name().to_string_lossy()
                );
            }
            Ok(Status::SkippedExists | Status::SkippedIsDir) => {
                Term::stdout().clear_line().ok();
                println!(
                    "\r{} {}",
                    "skipped".yellow(),
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
    force: bool,
) -> Result<Status> {
    let meta = entry.metadata().map_err(Error::Metadata)?;
    if meta.is_dir() {
        return Ok(Status::SkippedIsDir);
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
        .join(format!("{:0>2}", mtime.month()))
        .join(format!("{:0>2}", mtime.day()));

    dir_tree
        .mkdir_all(&dir)
        .map_err(Error::CreatingTargetDirectory)?;

    let target_dir = dir.join(entry.file_name());
    if !force && target_dir.exists() {
        let source_meta = entry
            .path()
            .metadata()
            .map_err(Error::GettingSourceFileMeta)?;
        let target_meta = target_dir
            .metadata()
            .map_err(Error::GettingTargetFileMeta)?;

        if source_meta.len() == target_meta.len() {
            return Ok(Status::SkippedExists);
        }
    }

    let mut source_file = File::open(entry.path()).map_err(Error::OpeningSourceFile)?;
    let mut target_file = File::create(target_dir).map_err(Error::CreatingTargetFile)?;
    io::copy(&mut source_file, &mut target_file).map_err(Error::CopyingFileContents)?;

    if mv {
        fs::remove_file(entry.path()).map_err(Error::DeletingSourceFile)?;
    }

    Ok(Status::Transferrred)
}
