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
    #[arg(short = 'i', long)]
    pub input: PathBuf,

    /// Output CSV file path
    #[arg(short = 'o', long)]
    pub output: PathBuf,

    /// Coordinate format
    #[arg(short = 'c', long, value_enum)]
    pub coord_format: CoordinateFormat,

    /// Input format
    #[arg(short = 'f', long, value_enum)]
    pub input_format: InputFormat,

    /// Strict mode: stop on first error
    #[arg(short = 's', long)]
    pub strict: bool,
}

// Supported coordinate formats.
#[derive(Copy, Clone, Debug, ValueEnum)]
pub enum CoordinateFormat {
    /// degree decimal
    Dd,
    /// degree minute second
    Dms,
    /// degree decimal minute
    Ddm,
}

// Supported input formats.
#[derive(Copy, Clone, Debug, ValueEnum)]
pub enum InputFormat {
    /// Csv file
    Csv,
    /// Json file
    Json,
    /// Json lines file
    Jsonl,
}
