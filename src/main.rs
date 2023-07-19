extern crate encoding_rs_io;
extern crate encoding_rs;

use anyhow::Error;
use encoding_rs_io::DecodeReaderBytes;
use encoding_rs_io::*;

use rayon::prelude::*;
use itertools::Itertools;
use rayon::string;
use std::borrow::Cow;
use std::collections::VecDeque;
use std::fs::File;
use std::io::BufReader;
use std::io::Seek;
use std::io::SeekFrom;
use std::iter::once;
use regex::Regex;



use clap::Parser;
use glob::glob_with;
use glob::MatchOptions;
use std::fs;
use std::fs::*;
use std::path::PathBuf;

mod glob_walk;
use glob_walk::GlobWalker;
use glob_walk::GlobWalkerBuilder;

#[derive(Parser)]
#[command(name = "reno")]
#[command(author = "Henrik Franzén")]
#[command(version = "1.0")]
#[command(about = "A small CLI utility written in Rust that helps with searching and replacing filenames and file contents recursively using regex and glob patterns.", long_about = None)]
#[command(next_line_help = true)]
struct Cli {
    #[arg(long, short='G')]
    /// Filename glob patterns, defaults to: "*"
    Glob_patterns: Vec<PathBuf>,

    #[arg(short='S')]
    /// Search regex
    Search_regex: Option<String>,

    #[arg(short='R')]
    /// Replace regex
    Replace_regex: Option<String>,
    
    #[arg(long, short)]
    /// (NOT IMPLEMENTED) Only search and replace file contents
    contents: bool,

    #[arg(long, short)]
    /// (NOT IMPLEMENTED) Only search and replace file names
    names: bool,

    #[arg(long, short)]
    /// (NOT IMPLEMENTED) Don't modify files, just show what would happen.
    dry: bool,

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

    
    let globs: Vec<String> = if cli.Glob_patterns.len() > 0 {
        cli.Glob_patterns
        .into_iter()
        .map(|p| p.into_os_string().into_string().unwrap_or_else(|_| String::from("*")))
        .collect::<Vec<String>>()
    } else {
        vec!["*".to_string()]
    };

    let max_depth = cli.max_depth.unwrap_or(usize::MAX);
    
    
    println!("Contents: {:?}", cli.contents);
    println!("File names: {:?}", cli.names);
    println!("Search regex: {}", cli.Search_regex.clone().unwrap_or(String::from("none")));
    println!("Replace regex: {}", cli.Replace_regex.clone().unwrap_or(String::from("none")));


    walk(globs, max_depth, 
        cli.Search_regex.clone(), 
        cli.Replace_regex.clone()).unwrap();


}

fn walk(globs: Vec<String>, max_depth: usize, search_string: Option<String>, replacer_string: Option<String>) -> Result<(), Box<dyn std::error::Error>>{
    let base_dir = ".";
    let walker = GlobWalkerBuilder::from_patterns(
        base_dir,
        &globs,
    )
        .max_depth(max_depth)
        .follow_links(true)
        .build()?
        .filter_map(Result::ok);

    walker.into_iter().par_bridge().for_each(|source_path| {
        let reader = File::open(source_path.path()).unwrap();
        let metadata  = reader.metadata().unwrap();
        let search_string = search_string.clone().unwrap();

        let mut should_replace = false;

        let replacer_string = match &replacer_string {
            Some(s) => { should_replace = true; s },
            None => {should_replace = false; ""},
        };

        let mut reader = BufReader::new(reader);

        let stringContents = 
            fs::read_to_string(source_path.clone().path())
            .unwrap();

        if !stringContents.is_empty() {
            let re = Regex::new(&search_string).unwrap();
            let search = re.find_iter(&stringContents);
            //let mut found_count = 0;
            for search_match in search {
                print!("{:?}: {}-{} ", source_path.path(), search_match.start(), search_match.end());
                //found_count += 1;
            } 


            if should_replace {
                    
                let result = re.replace_all(&stringContents, replacer_string);
                fs::write(source_path.path(), result.as_bytes()).unwrap();
            }
        }
        
    });

    Ok(())
        // write to file
}