use crate::dirtree::DirTree;
use crate::errors::Error;
use anyhow::Result;
use chrono::{DateTime, Datelike};
use clap::{Parser, command};
use indicatif::ProgressStyle;
use std::fs::{self, DirEntry, File};
use std::io;
use std::path::{Path, PathBuf};
use std::time::UNIX_EPOCH;
use yansi::Paint;

mod dirtree;
mod errors;

const PB_TEMPLATE: &str = "{spinner:.green} [{wide_bar:.cyan/dim}] {pos}/{len} ({per_sec}, {eta})";

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

    let files = fs::read_dir(cli.source_dir)
        .map_err(Error::ReadingSourceDir)?
        .collect::<Result<Vec<_>, _>>()?;

    let pb_style = ProgressStyle::with_template(PB_TEMPLATE)
        .expect("bar template")
        .progress_chars("=>-");
    let bar = indicatif::ProgressBar::new(files.len() as u64).with_style(pb_style);

    let mut failed = vec![];
    let mut skipped = vec![];

    for entry in bar.wrap_iter(files.iter()) {
        match process_entry(&mut dir_tree, entry, &cli.target_dir, cli.mv, cli.force) {
            Ok(Status::Transferrred) => (),
            Ok(Status::SkippedExists | Status::SkippedIsDir) => skipped.push(entry.file_name()),
            Err(err) => failed.push((entry.file_name(), err)),
        }
    }

    bar.finish_and_clear();

    let n_processed = files.len() - failed.len() - skipped.len();
    println!(
        "{} {}",
        n_processed.to_string().green().bold(),
        "files have been processed.".green()
    );

    if !skipped.is_empty() {
        println!(
            "{} {}",
            skipped.len().to_string().yellow().bold(),
            "files have been skipped.".yellow()
        );
    }

    if !failed.is_empty() {
        println!(
            "{} {} {}",
            "Failed processing for".red(),
            failed.len().to_string().red().bold(),
            "files:".red()
        );

        for (filename, err) in &failed {
            println!(
                "{}",
                format_args!(" - {}: {}", filename.to_string_lossy(), err).red()
            );
        }

        return Err(anyhow::anyhow!(
            "Processing failed for {} files.",
            failed.len()
        ));
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
