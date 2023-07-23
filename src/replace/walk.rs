extern crate encoding_rs;
extern crate encoding_rs_io;

use crate::replace::contents::*;
use crate::replace::names::*;
use crate::glob_walk::GlobWalkerBuilder;

use anyhow::Result;
use std::str;
use rayon::prelude::*;
use std::fs::File;
use std::fs::*;
use std::string::String;

pub fn walk(
    globs: Vec<String>,
    max_depth: usize,
    search_string: String,
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
        search_string.clone()
    );
    println!("Replace regex: {}", replacer_string.clone());

    walker.into_iter().par_bridge().for_each(|source_path| {
        if b_verbose {
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

        if b_contents {
            let content_info = do_contents(
                source_path.path(),
                search_string.as_ref(),
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
                        if b_verbose {
                            println!("Empty file: {:?}", e);
                        }
                    }
                    Some(DoContentError::ReadDecodingError(_path)) => {
                        if b_verbose {
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
                search_string.as_ref(),
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
