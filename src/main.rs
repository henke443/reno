extern crate encoding_rs;
extern crate encoding_rs_io;

mod glob_walk;
mod replace;

use clap::{ColorChoice, Parser, crate_version, crate_name, crate_authors, crate_description};
use replace::*;
use std::path::PathBuf;
use std::str;
use std::string::String;

const DEFAULT_MAX_DEPTH: &'static str = "4294967294";

#[derive(Parser)]
#[command(name = crate_name!())]
#[command(author = crate_authors!())]
#[command(version = crate_version!())]
#[command(about = crate_description!(), long_about = None)]
#[command(next_line_help = true)]
#[command(color = ColorChoice::Auto)]
struct Cli {
    /// Search regex or binary sequence if --bin is passed.
    ///
    /// In the binary mode, the search string should be a binary sequence with optional wildcards (e.g.: "\x22\x??\x??\x44\x22\x01\x69\x55" or "22 ?? ?? 44 22 01 69 55"))
    search: String,

    /// Regex (e.g.: "Hello ${1}") in the normal mode.
    ///
    /// **IMPORTANT**: Even though capture groups without curly braces (for example just $1 instead of ${1}) mostly work, I strongly advise using them as unexpected results can occur otherwise.
    ///
    /// Be sure to always run --dry before you actually replace anything.
    ///
    /// A binary sequence (e.g.: "\x22\x01\xD5\x44\x22\x01\x69\x55") in binary mode.
    ///
    /// Dry mode if left empty.
    replace: Option<String>,

    #[arg(long)]
    ///Don't modify files, just show what would happen.
    dry: bool,

    #[arg(long, short, default_value = "**")]
    /// Filename glob patterns, defaults to: "*"
    globs: Vec<PathBuf>,

    #[arg(long = "bin", short)]
    /// Binary search and replace mode
    binary: bool,

    #[arg(long, short)]
    /// Only search and replace file contents
    contents: bool,

    #[arg(long, short)]
    /// Only search and replace file names
    names: bool,

    #[arg(long, short, default_value = DEFAULT_MAX_DEPTH)]
    /// Max depth of directory traversal.
    /// 0 means only current directory.
    depth: usize,

    #[arg(long, short)]
    /// Prints (very) verbosely
    verbose: bool,
}

fn main() {
    let cli = Cli::parse();

    let globs: Vec<String> = cli
        .globs
        .into_iter()
        .map(|p| p.into_os_string().into_string().unwrap())
        .collect::<Vec<String>>();

    let max_depth = cli.depth + 1;

    walk(
        globs,
        max_depth,
        cli.search.clone(),
        cli.replace.clone(),
        cli.names,
        cli.contents,
        cli.dry,
        cli.binary,
        cli.verbose,
    )
    .unwrap();
}

#[cfg(test)]
mod test;
