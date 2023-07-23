extern crate encoding_rs;
extern crate encoding_rs_io;

use anyhow::Context;

use anyhow::__private::kind::TraitKind;
use anyhow::bail;
use anyhow::ensure;
use anyhow::Result;
use thiserror::Error;

use std::str;

use itertools::Itertools;
use rayon::prelude::*;

use regex::Regex;

use std::fs::File;

use clap::Parser;

use std::fs;
use std::fs::*;
use std::path::Path;
use std::path::PathBuf;
use std::string::String;

mod glob_walk;

use glob_walk::GlobWalkerBuilder;

use std::io::{BufRead, BufReader, Read};

use encoding_rs_io::DecodeReaderBytesBuilder;

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

fn walk(
    globs: Vec<String>,
    max_depth: usize,
    search_string: Option<String>,
    replacer_string: Option<String>,
    replace_filenames: bool,
    replace_contents: bool,
    b_dry: bool,
    b_bin: bool,
    b_verbose: bool,
) -> Result<(), Box<dyn std::error::Error>> {
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
    println!("Replace regex: {}", replacer_string.clone());

    walker.into_iter().par_bridge().for_each(|source_path| {
        if (b_verbose) {
            println!("reading file: {:?}", source_path.path());
        }
        let _source_path = source_path.path().to_str().unwrap();
        if _source_path == "."
            || _source_path == ".."
            || _source_path == "./"
            || _source_path == "../"
            || _source_path == ""
            || _source_path == " "
        {
            println!("skipping: {:?}", source_path.path());
            return;
        }

        let mut b_contents = b_contents;

        let metadata: Metadata;
        let file_size;

        if source_path.path().is_file() {
            let _reader = File::open(source_path.path()).unwrap();
            metadata = _reader.metadata().unwrap();
            file_size = metadata.len();
        } else {
            b_contents = false;
        }
        //let mut reader = BufReader::new(_reader);

        if b_contents && search_string.is_some() {
            let content_info = do_contents(
                source_path.path(),
                &(search_string.as_ref().unwrap()),
                &replacer_string,
                b_dry,
                b_bin,
            );
            match content_info {
                Ok(replacement_infos) => {
                    for replacement_info in replacement_infos {
                        println!("{:?}", replacement_info.path);
                        for replacement in replacement_info.replacements {
                            println!(
                                "    {} lines {}:{} = {:?} -> {:?}",
                                if !replacement_info.did_change {
                                    "<dry>"
                                } else {
                                    ""
                                },
                                replacement.start,
                                replacement.end,
                                replacement.original,
                                replacement.new
                            );
                        }
                    }
                }
                Err(e) => match e.downcast_ref() {
                    Some(DoContentError::NoMatchesFound) => {
                        if b_verbose {
                            println!("No matches found: {:?}", e);
                        }
                    }
                    Some(DoContentError::EmptyFile) => {
                        if (b_verbose) {
                            println!("Empty file: {:?}", e);
                        }
                    }
                    Some(DoContentError::ReadDecodingError(_path)) => {
                        if (b_verbose) {
                            println!("Read decoding error: {:?}", e);
                        }
                    },
                    Some(DoContentError::HexDecodeError(_path, _)) => {
                         println!("Hex decode error: {:?}", e);
                         std::process::exit(1);
                    }
                    _ => {
                        panic!("Critical error: {}", e)
                    }
                },
            }
        }

        if b_names {
            let names_info = do_names(
                source_path.path(),
                search_string.clone(),
                &replacer_string,
                b_dry,
            );
            match names_info {
                Ok(replacement_infos) => {
                    for replacement_info in replacement_infos {
                        println!(
                            "{}, {:?} -> {:?}",
                            if !replacement_info.did_change {
                                "<dry>"
                            } else {
                                ""
                            },
                            replacement_info.old_name,
                            replacement_info.new_name
                        );
                    }
                }
                Err(err) => {
                    println!("Names error: {:?}", err);
                }
            }
        }
    });
    Ok(())
}

#[derive(Error, Debug)]
enum DoContentError {
    #[error("Empty file\n")]
    EmptyFile,
    #[error("File \"{0}\" is not in a valid encoding, try running with --bin\n")]
    ReadDecodingError(String),
    #[error("IO Error\n")]
    NoMatchesFound,
    #[error("Error writing to file: {0}\n")]
    WriteError(String, #[source] std::io::Error),
    #[error("Could not decode string as hex: {0}\n")]
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
    did_change: bool,
    path: PathBuf,
    replacements: Vec<ContentReplacementInfo>,
}

// Instead of a Vec<u8> we will use a Vec<ByteMatcher> to allow for wildcards
struct ByteMatcher {
    value: u8,
    is_wildcard: bool,
}


fn do_contents(
    source_path: &Path,
    str_search: &str,
    str_replace: &str,
    b_dry: bool,
    b_bin: bool,
) -> Result<Vec<FileReplacementInfo>> {
    let file = File::open(source_path)?;

    let mut replacement_infos: Vec<FileReplacementInfo> = vec![];

    // Binary search and replace contents
    if b_bin {
        replacement_infos = do_contents_binary(source_path, file, str_search, str_replace, b_dry)?;
    }
    // Plain text search and replace contents
    else {
        replacement_infos = do_contents_plain(source_path, file, str_search, str_replace, b_dry)?;
    }

    Ok(replacement_infos)
}

fn do_contents_plain(source_path: &Path, file: File, str_search: &str, str_replace: &str, b_dry: bool) -> Result<Vec<FileReplacementInfo>> {
    let re = if !str_search.is_empty() {
        Regex::new(&str_search).unwrap()
    } else {
        Regex::new(".*").unwrap()
    };

    // make reader that does BOM sniffing using encoding_rs
    let mut reader = BufReader::new(DecodeReaderBytesBuilder::new().build(file));

    // read file to string
    let mut str_contents = String::new();
    reader
        .read_to_string(&mut str_contents)
        .or(Err(DoContentError::ReadDecodingError(String::from(
            source_path.to_string_lossy(),
        ))))?;

    // search and make sure we have matches

    let search_result = re.find_iter(&str_contents);

    let v_search_result = search_result.collect::<Vec<regex::Match>>();

    if v_search_result.len() == 0 {
        return Ok(vec![]);
    }

    // do replacement and write result

    let result = re.replace_all(&str_contents, str_replace);

    if (!b_dry) {
        let write_result = fs::write(source_path, result.as_bytes());
        ensure!(
            write_result.is_ok(),
            DoContentError::WriteError(String::from(result), write_result.unwrap_err())
        );
    }

    let mut file_replacement_infos: Vec<FileReplacementInfo> = vec![];

    // log changes, this will make us do re.replace_all twice, but it's fine for now.
    for search_match in v_search_result {
        file_replacement_infos.push(FileReplacementInfo {
            path: source_path.to_path_buf(),
            did_change: !b_dry,
            replacements: vec![ContentReplacementInfo {
                start: search_match.start(),
                end: search_match.end(),
                length: search_match.end() - search_match.start(),
                new: re
                    .replace_all(&search_match.as_str(), str_replace)
                    .to_string(),
                original: search_match.as_str().to_string(),
            }],
        });
    }
    return Ok(file_replacement_infos);
}

fn do_contents_binary(source_path: &Path, file: File, str_search: &str, str_replace: &str, b_dry: bool) -> Result<Vec<FileReplacementInfo>> {

    // decode string hex signature
    let decode_hex_bytes = |s: &str| -> Result<Vec<ByteMatcher>, DoContentError> {
        return s
            .split("\\x")
            .filter(|s| !s.is_empty())
            .map(|s| -> Result<ByteMatcher, DoContentError> {
                return u8::from_str_radix(s, 16)
                    .map(|x| ByteMatcher {
                        value: x,
                        is_wildcard: false,
                    })
                    .or_else(|err| {
                        if s == "*" || s == "??" || s == "?" {
                            return Ok(ByteMatcher {
                                value: 0,
                                is_wildcard: true,
                            });
                        }
                        Err(DoContentError::HexDecodeError(
                            String::from(str_search),
                            err,
                        ))
                    });
            })
            .try_collect();
    };

    let search_hex_bytes: Vec<ByteMatcher> = decode_hex_bytes(str_search)?;
    let replace_hex_bytes: Vec<ByteMatcher> = decode_hex_bytes(str_replace)?;

    // read file contents as binary and do replacement
    let reader = BufReader::new(file);
    let mut out_buffer: Vec<u8> = vec![];
    let i: usize = 0;

    let mut matches: Vec<(usize, usize)> = vec![];

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

    let mut file_replacement_infos: Vec<FileReplacementInfo> = vec![];

    // edit bytes with our full length matches
    for m in matches {
        for i in m.0..m.1 {
            // dont do anything if replacement is wildcard or if we shouldn't write changes
            if b_dry || replace_hex_bytes[i - m.0].is_wildcard {
                continue;
            }
            // otherwise write change
            out_buffer[i] = replace_hex_bytes[i - m.0].value;
        }

        file_replacement_infos.push(FileReplacementInfo {
            path: source_path.to_path_buf(),
            did_change: !b_dry,
            replacements: vec![ContentReplacementInfo {
                start: m.0,
                end: m.1,
                length: m.1 - m.0,
                new: str_replace.to_string(),
                original: str_search.to_string(),
            }],
        });
    }

    if !b_dry {
        fs::write(source_path, out_buffer.as_slice())?; // write changes to file
    }

    return Ok(file_replacement_infos);
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
    new_name: String,
}

fn do_names(
    source_path: &Path,
    str_search: Option<String>,
    str_replace: &str,
    b_dry: bool,
) -> Result<Vec<NameReplacementInfo>> {
    let re = if str_search.is_some() {
        Regex::new(&str_search.unwrap())?
    } else {
        Regex::new(".*")?
    };

    let old_path = source_path;
    let old_name = old_path
        .file_name()
        .context(DoNamesError::InvalidFilename(Box::from(old_path)))?
        .clone()
        .to_str()
        .context(DoNamesError::InvalidFilename(Box::from(old_path)))?;

    let new_name = re.replace_all(&old_name, str_replace);
    let new_path = old_path.with_file_name(PathBuf::from(new_name.clone().into_owned()));

    let mut name_replacements_info = vec![];

    if new_name != old_name {
        let mut replacement_was_ok = false;

        if !b_dry {
            fs::rename(&old_path, &new_path)
                .or_else(|err| {
                    Err(DoNamesError::RenameError(
                        new_name.clone().into_owned(),
                        err,
                    ))
                })
                .and_then(|_| {
                    replacement_was_ok = true;
                    Ok(())
                })?;
        }

        name_replacements_info.push(NameReplacementInfo {
            did_change: !b_dry && replacement_was_ok,
            path: old_path.to_path_buf(),
            old_name: old_name.to_string(),
            new_name: new_name.to_string(),
        });
    }

    Ok(name_replacements_info)
}

#[cfg(test)]
mod test;
