# archiver

Simple CLI app to archive media files into different directories by creation dates

## Usage

```
$ archiver --help
Simple CLI app to archive media files into different directories by creation dates

Usage: archiver.exe [OPTIONS] <SOURCE_DIR> <TARGET_DIR>

Arguments:
  <SOURCE_DIR>  Source directory
  <TARGET_DIR>  Target directory

Options:
  -m, --mv       Move files instead of copying them
  -f, --force    Overwrite existing files
  -h, --help     Print help
  -V, --version  Print version
```



## Install

You can either download the latest release builds form the [Releases page](https://github.com/shellshape/archiver/releases) or you can install it using cargo install.
```
cargo install --git https://github.com/shellshape/archiver
```