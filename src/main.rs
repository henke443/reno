extern crate encoding_rs;
extern crate encoding_rs_io;

use anyhow::Error;
use encoding_rs_io::DecodeReaderBytes;
use encoding_rs_io::*;

use itertools::Itertools;
use rayon::prelude::*;
use rayon::string;
use regex::Regex;
use std::borrow::Cow;
use std::collections::VecDeque;
use std::ffi::OsStr;
use std::fs::File;
use std::io::BufReader;
use std::io::Seek;
use std::io::SeekFrom;
use std::iter::once;

use clap::Parser;
use glob::glob_with;
use glob::MatchOptions;
use std::fs;
use std::fs::*;
use std::path::Path;
use std::path::PathBuf;

mod glob_walk;
use glob_walk::GlobWalker;
use glob_walk::GlobWalkerBuilder;

#[derive(Parser)]
#[command(name = "reno")]
#[command(author = "Henrik Franzén")]
#[command(version = "0.0.1")]
#[command(about = "A small CLI utility written in Rust that helps with searching and replacing filenames and file contents recursively using regex and glob patterns.", long_about = None)]
#[command(next_line_help = true)]


struct Cli {
    #[arg(short = 'S')]
    /// Search regex, can only be omitted if --names is present
    Search_regex: Option<String>,


    #[arg(long, short = 'G')]
    /// Filename glob patterns, defaults to: "./*.*"
    Glob_patterns: Option<Vec<PathBuf>>,

    #[arg(short = 'R')]
    /// Replace regex
    Replace_regex: Option<String>,

    #[arg(long, short)]
    ///Don't modify files, just show what would happen.
    dry: bool,

    #[arg(long, short)]
    /// Only search and replace file contents
    contents: bool,

    #[arg(long, short)]
    /// Only search and replace file names
    names: bool,

    #[arg(long, short)]
    /// Max depth of directory traversal, unlimited by default. 0 means only the current directory.
    max_depth: Option<usize>,

    #[arg(long, short)]
    /// (NOT IMPLEMENTED) Prepends the replacement to the start of all matched strings
    prefix: bool,

    #[arg(long, short)]
    /// (NOT IMPLEMENTED) Appends the replacement to the end of all matched strings
    suffix: bool,
}

fn main() {
    let cli = Cli::parse();

    let default_globs_s = String::from("*.*");
    let default_globs = vec![default_globs_s.clone()];

    let globs: Vec<String> = match cli.Glob_patterns {
        Some(G) => {
            if G.len() > 0 {
                G.into_iter()
                    .map(|p| {
                        p.into_os_string()
                            .into_string()
                            .unwrap_or_else(|_| default_globs_s.clone())
                    })
                    .collect::<Vec<String>>()
            } else {
                default_globs.clone()
            }
        }
        None => default_globs.clone(),
    };

    let max_depth = cli.max_depth.unwrap_or(usize::MAX);

    println!("Contents: {:?}", cli.contents);
    println!("File names: {:?}", cli.names);
    println!(
        "Search regex: {}",
        cli.Search_regex.clone().unwrap_or(String::from("none"))
    );
    println!(
        "Replace regex: {}",
        cli.Replace_regex.clone().unwrap_or(String::from("none"))
    );

    walk(
        globs,
        max_depth,
        cli.Search_regex.clone(),
        cli.Replace_regex.clone(),
        cli.names,
        cli.contents,
        cli.dry,
    )
    .unwrap();
}

fn walk(
    globs: Vec<String>,
    max_depth: usize,
    search_string: Option<String>,
    replacer_string: Option<String>,
    replace_filenames: bool,
    replace_contents: bool,
    b_dry: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let verbose = true;

    let mut b_names = replace_filenames;
    let mut b_contents = replace_contents;
    let mut b_replace = !b_dry;

    if (!b_names && !b_contents && b_replace) {
        b_names = true;
        b_contents = true;
    }

    let replacer_string: &str = match &replacer_string {
        Some(s) => s,
        None => {
            b_replace = false;
            println!("No replacer string provided, dry run.");
            ""
        }
    };

    let base_dir = ".";

    let walker = GlobWalkerBuilder::from_patterns(base_dir, &globs)
        .max_depth(max_depth)
        .follow_links(true)
        .build()
        .unwrap()
        .filter_map(Result::ok);

    println!("Globs: {:?}", globs);
    
    match search_string {
        Some(_) => {},
        None => {
            if (!b_names) {
                println!("No search string provided and no --names, dry run.");
                b_replace = false;
            }
        }
    }
    let search_string = search_string.clone().unwrap();
    let re = Regex::new(&search_string).unwrap();

    println!("Dry run? {}", !b_replace);

    walker.into_iter().par_bridge().for_each(|source_path| {
        println!("reading file: {:?}", source_path.path());
        //

        let mut b_contents = b_contents;

        let mut metadata: Metadata;
        let mut file_size;

        if source_path.path().is_file() {
            let _reader = File::open(source_path.path()).unwrap();
            metadata = _reader.metadata().unwrap();
            file_size = metadata.len();
        } else {
            b_contents = false;
        }
        //let mut reader = BufReader::new(_reader);

        if b_contents {
            println!("should do contents");

            let string_contents = fs::read_to_string(source_path.clone().path()).unwrap();

            if !string_contents.is_empty() {
                let search_result = re.find_iter(&string_contents);

                for search_match in search_result {
                    print!(
                        "(contents) {:?}: {}-{} ",
                        source_path.path(),
                        search_match.start(),
                        search_match.end()
                    );
                }
                let result = re.replace_all(&string_contents, replacer_string);
                println!(": {}", result);
                if b_replace {
                    fs::write(source_path.path(), result.as_bytes()).unwrap();
                }
            }
        }

        if b_names {
            let old_path = source_path.into_path(); // no more
            let old_name = old_path.file_name().unwrap(); // no more
            let old_name_str = old_name.clone().to_string_lossy();
            let new_name = re.replace_all(&old_name_str, replacer_string);
            let new_path = old_path.with_file_name(PathBuf::from(new_name.clone().into_owned()));

            if (verbose) {
                println!("(not changing filename) {} ", old_name.to_string_lossy(),);
            }
            if (new_path.file_name().unwrap() != old_path.file_name().unwrap()) {
                println!(
                    "(replacing filename) {} -> {} ",
                    old_name.to_string_lossy(),
                    new_name
                );

                if b_replace {
                    fs::rename(&old_path, &new_path).unwrap();
                }
            }
        }
    });
    // write to file
    Ok(())
}
