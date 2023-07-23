extern crate encoding_rs;
extern crate encoding_rs_io;

use anyhow::Context;
use anyhow::Error;
use anyhow::bail;
use anyhow::ensure;
use thiserror::Error;
use anyhow::Result;

use encoding_rs_io::DecodeReaderBytes;
use encoding_rs_io::*;
use std::str;
use bytes::{Buf, BufMut};


use itertools::Itertools;
use rayon::prelude::*;
use rayon::string;
use regex::Regex;
use std::borrow::Cow;
use std::collections::VecDeque;
use std::ffi::OsStr;
use std::fs::File;
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
use std::string::String;

mod glob_walk;
use glob_walk::GlobWalker;
use glob_walk::GlobWalkerBuilder;

use std::io::{BufReader, BufRead, Read};

use encoding_rs::WINDOWS_1252;
use encoding_rs_io::DecodeReaderBytesBuilder;


#[derive(Parser)]
#[command(name = "reno")]
#[command(author = "henke443")]
#[command(version = "0.0.1")]
#[command(about = "A small CLI utility written in Rust that helps with searching and replacing filenames and file contents recursively using regex and glob patterns.", long_about = None)]
#[command(next_line_help = true)]


struct Cli {
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

    #[arg(long, short)]
    /// Binary search and replace mode
    binary: bool,

    #[arg(long, short)]
    /// Only search and replace file contents
    contents: bool,

    #[arg(long, short)]
    /// Only search and replace file names
    names: bool,

    #[arg(long, short, default_value = "0")]
    /// Max depth of directory traversal. 
    /// 0 means only the current directory.
    depth: Option<usize>,
}

fn main() {
    let cli = Cli::parse();

    let default_globs_s = String::from("*");
    let default_globs = vec![default_globs_s.clone()];

    let globs: Vec<String> = cli.globs.into_iter()
        .map(|p| {
            p.into_os_string()
                .into_string()
                .unwrap_or_else(|_| default_globs_s.clone())
        })
        .collect::<Vec<String>>();

    let max_depth = cli.depth.unwrap_or(usize::MAX);

    walk(
        globs,
        max_depth,
        cli.search.clone(),
        cli.replace.clone(),
        cli.names,
        cli.contents,
        cli.dry,
        cli.binary,
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
    b_bin: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let verbose = true;

    let mut b_names = replace_filenames;
    let mut b_contents = replace_contents;

    if !b_names && !b_contents {
        b_names = true;
        b_contents = true;
    }

    let mut b_replace = !b_dry;

    let replacer_string: &str = match &replacer_string {
        Some(s) => s,
        None => {
            b_replace = false;
            println!("No replacer string provided, dry run.");
            ""
        }
    };

    if search_string.is_none() {
        if !b_names {
            println!("No search string provided and no --names, dry run.");
            b_replace = false;
        }
        b_contents = false;
    }

    let base_dir = ".";

    let walker = GlobWalkerBuilder::from_patterns(base_dir, &globs)
        .max_depth(max_depth)
        .follow_links(true)
        .build()
        .unwrap()
        .filter_map(Result::ok);

    println!("Globs: {:?}", globs);
    println!("Dry run? {}", !b_replace);

    
    println!("Contents: {:?}", b_contents);
    println!("File names: {:?}", b_names);
    println!(
        "Search regex: {}",
        search_string.clone().unwrap_or(String::from("none"))
    );
    println!(
        "Replace regex: {}",
        replacer_string.clone()
    );

    walker.into_iter().par_bridge().for_each(|source_path| {
        println!("reading file: {:?}", source_path.path());
        let _source_path = source_path.path().to_str().unwrap();
        if  _source_path == "." || _source_path == ".." {
            println!("skipping: {:?}", source_path.path());
            //return;
        }

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

        if b_contents && search_string.is_some() {
           // let content_info = do_contents(source_path.path(), &(search_string.as_ref().unwrap()), &replacer_string, b_dry, b_replace, b_bin);
           // println!("{:?}", content_info.unwrap());
        }

        if b_names {
           //let names_info = do_names(source_path.path(), search_string.clone(), &replacer_string, b_dry, b_replace);
           //println!("{:?}", names_info.unwrap());
        }
    });
    Ok(())
}


#[derive(Error, Debug)]
enum DoContentError {
    #[error("Empty file")]
    EmptyFile,
    #[error("File is binary, try running with --bin")]
    BinaryFile,
    #[error("IO Error")]
    IoError(#[from] std::io::Error),
    #[error("No matches found")]
    NoMatchesFound,
    #[error("Error writing to file: {0}")]
    WriteError(String, #[source] std::io::Error),
    #[error("Could not decode string as hex: {0}")]
    HexDecodeError(String, #[source] std::num::ParseIntError),
}

#[derive(Debug)]
struct ContentReplacementInfo {
    start: usize,
    end: usize,
    length: usize,
    new: String,
    original: String,
}

#[derive(Debug)]
struct FileReplacementInfo {
    path: PathBuf,
    replacements: Vec<ContentReplacementInfo>,
}

// Instead of a Vec<u8> we will use a Vec<ByteMatcher> to allow for wildcards
struct ByteMatcher {
    value: u8,
    is_wildcard: bool
}

fn do_contents(
    source_path: &Path, 
    str_search: &str,
    str_replace: &str, 
    b_dry: bool, 
    b_replace: bool, 
    b_bin: bool, 
) -> Result<Vec<FileReplacementInfo>> {

    let file = File::open(source_path)?;

    let mut replacement_infos: Vec<FileReplacementInfo> = vec![];


    // Binary search and replace contents
    if b_bin {        
        // decode string hex signature
        let decode_hex_bytes = |s: &str| -> Result<Vec<ByteMatcher>, DoContentError> {
            return s.split("\\x")
                .filter(|s| !s.is_empty())
                .map(|s| -> Result<ByteMatcher, DoContentError> {
                    return u8::from_str_radix(s, 16)
                    .map(|x| ByteMatcher { value: x, is_wildcard: false})
                    .or_else(|err| 
                        {
                            if s == "*" || s == "??" || s == "?" {
                                return Ok(ByteMatcher { value: 0, is_wildcard: true });
                            }
                            Err(
                                DoContentError::HexDecodeError(String::from(str_search), err),
                            )
                        }
                    );
                }).try_collect();
        };

        let search_hex_bytes: Vec<ByteMatcher> = decode_hex_bytes(str_search)?;
        let replace_hex_bytes: Vec<ByteMatcher> = decode_hex_bytes(str_replace)?;

        // read file contents as binary and do replacement
        let mut reader = BufReader::new(file);
        let mut out_buffer: Vec<u8> = vec![];
        let mut i: usize = 0;

        let mut matches: Vec<(usize,usize)> = vec![];

        let mut match_start: usize = 0;
        let mut match_end: usize = 0;
        for byte in reader.bytes() {
            let _byte = byte?;
            // if we have a full length match, save it for later
            if (match_end - match_start) == search_hex_bytes.len() {
                matches.push((match_start, match_end));
            }

            // get current match length
            match_end = i;
            if !search_hex_bytes[i].is_wildcard && search_hex_bytes[i].value != _byte {
                match_start += 1; // if it doesn't match we increment match_start
            }

            // always push original byte which we will edit later
            out_buffer.push(_byte);
        }

        // edit bytes with our full length matches
        for m in matches {
            for i in m.0..m.1 {
                // dont do anything if replacement is wildcard
                if replace_hex_bytes[i - m.0].is_wildcard {
                    continue;
                }
                // otherwise write change
                out_buffer[i] = replace_hex_bytes[i - m.0].value;
            }

            replacement_infos.push(FileReplacementInfo {
                path: source_path.to_path_buf(),
                replacements: vec![ContentReplacementInfo {
                    start: m.0,
                    end: m.1,
                    length: m.1 - m.0,
                    new: str_replace.to_string(),
                    original: str_search.to_string(),
                }],
            })
        }
    }
    // Plain text search and replace contents
    else if !b_bin {
        let re = if !str_search.is_empty() {
            Regex::new(&str_search).unwrap()
        } else {
            Regex::new(".*").unwrap()
        };

        // make reader that does BOM sniffing using encoding_rs, and check file validity
        let mut reader = BufReader::new(
             DecodeReaderBytesBuilder::new()
                .build(file));

        if reader.buffer().is_empty() {
            bail!(DoContentError::EmptyFile);
        }

        // read file to string

        let mut str_contents = String::new();

        reader.read_to_string(&mut str_contents)?;

        // search and make sure we have matches

        let search_result = re.find_iter(&str_contents);
        
        let v_search_result = search_result.collect::<Vec<regex::Match>>();
        if !v_search_result.len() > 0 {
            bail!(DoContentError::NoMatchesFound);
        }

        // do replacement and write result

        let result = re.replace_all(&str_contents, str_replace);

        let write_result = fs::write(source_path, result.as_bytes());

        ensure!(write_result.is_ok(), DoContentError::WriteError(String::from(result), write_result.unwrap_err()));


        // log changes, this will make us do re.replace_all twice, but it's fine for now.
        for search_match in v_search_result {
            replacement_infos.push(FileReplacementInfo {
                path: source_path.to_path_buf(),
                replacements: vec![ContentReplacementInfo {
                    start: search_match.start(),
                    end: search_match.end(),
                    length: search_match.end() - search_match.start(),
                    new: re.replace_all(&search_match.as_str(), str_replace).to_string(),
                    original: search_match.as_str().to_string(),
                }],
            })
        }
     }

     Ok(replacement_infos)
} 


#[derive(Error, Debug)]
enum DoNamesError {
    #[error("IO Error")]
    IoError(#[from] std::io::Error),
    #[error("Error renaming file: {0}")]
    RenameError(String, #[source] std::io::Error),
    #[error("Invalid filename: {0}")]
    InvalidFilename(Box<Path>),
}

#[derive(Debug)]
struct NameReplacementInfo {
    did_change: bool,
    path: PathBuf,
    old_name: String,
    new_name: String
}

fn do_names(
    source_path: &Path, 
    str_search: Option<String>, 
    str_replace: &str, 
    b_dry: bool, 
    b_replace: bool
) -> Result<Vec<NameReplacementInfo>> {

    let re = if str_search.is_some() {
        Regex::new(&str_search.unwrap())?
    } else {
        Regex::new(".*")?
    };

    let old_path = source_path;
    let old_name = old_path
    .file_name().context(DoNamesError::InvalidFilename(Box::from(old_path)))?.clone()
    .to_str().context(DoNamesError::InvalidFilename(Box::from(old_path)))?;

    let new_name = re.replace_all(&old_name, str_replace);
    let new_path = old_path.with_file_name(PathBuf::from(new_name.clone().into_owned()));

    let mut name_replacements_info = vec![];

    if new_path.file_name().unwrap() != old_path.file_name().unwrap() {

        let mut replacement_was_ok = false;

        if b_replace {
            fs::rename(&old_path, &new_path).or_else(|err| Err(DoNamesError::RenameError(
                new_name.clone().into_owned(),
                err,
            ))).and_then(|_| {
                replacement_was_ok = true;
                Ok(())
            })?;
        }
        
        name_replacements_info.push(NameReplacementInfo {
            did_change: b_replace && replacement_was_ok,
            path: old_path.to_path_buf(),
            old_name: old_name.to_string(),
            new_name: new_name.to_string()
        });
    }

    Ok(name_replacements_info)
}


#[cfg(test)]
mod test;
