extern crate encoding_rs;
extern crate encoding_rs_io;

mod glob_walk;
mod replace;

use replace::*;

use std::str;

use clap::Parser;

use std::path::PathBuf;
use std::string::String;

const DEFAULT_MAX_DEPTH: &'static str = "4294967294";

#[derive(Parser)]
#[command(name = "reno")]
#[command(author = "henke443")]
#[command(version = "0.0.1")]
#[command(about = "A small CLI utility written in Rust that helps with searching and replacing filenames and file contents recursively using regex and glob patterns.", long_about = None)]
#[command(next_line_help = true)]
struct Cli {
    #[arg(short = 'S')]
    /// Search regex or binary sequence if --bin is passed.
    /// Can only be omitted if --names is present
    search: Option<String>,

    #[arg(short = 'R')]
    /// Either regex (e.g.: "Hello $1") in the normal mode,
    /// or a binary sequence (e.g.: "\x22\x01\xD5\x44\x22\x01\x69\x55") in binary mode.
    /// Dry mode if left empty
    replace: Option<String>,

    #[arg(long)]
    ///Don't modify files, just show what would happen.
    dry: bool,

    #[arg(long, short = 'G', default_value = "*")]
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
    /// Defaults to unlimited.
    depth: usize,

    #[arg(long, short)]
    /// Only search and replace file names
    verbose: bool,
}

fn main() {
    let cli = Cli::parse();

    let default_globs_s = String::from("*");
    let _default_globs = vec![default_globs_s.clone()];

    let globs: Vec<String> = cli
        .globs
        .into_iter()
        .map(|p| {
            p.into_os_string()
                .into_string()
                .unwrap_or_else(|_| default_globs_s.clone())
        })
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
