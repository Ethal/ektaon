// src/cls.rs

use clap::Parser;
use clap::ValueEnum;
use std::path::PathBuf;

/* ---------------- CLI ---------------- */

// Command-line interface definition.
#[derive(Parser, Debug)]
#[command(author, version, about)]
pub struct Cli {
    /// Input CSV file path
    #[arg(short, long)]
    pub input: PathBuf,

    /// Output CSV file path
    #[arg(short, long)]
    pub output: PathBuf,

    /// Coordinate input format
    #[arg(short = 'f', long, value_enum)]
    pub input_format: InputFormat,

    /// Strict mode: stop on first error
    #[arg(long)]
    pub strict: bool,
}

// Supported coordinate formats.
#[derive(Copy, Clone, Debug, ValueEnum)]
pub enum InputFormat {
    Dd,
    Dms,
    Ddm,
}
