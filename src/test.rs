use glob_walk::*;
use std::path::Path;

use super::*;
use std::fs::{create_dir_all, File};
use tempfile::TempDir;

fn touch(dir: &TempDir, names: &[&str]) {
    for name in names {
        let name = normalize_path_sep(name);
        File::create(dir.path().join(name)).expect("Failed to create a test file");
    }
}

fn normalize_path_sep<S: AsRef<str>>(s: S) -> String {
    s.as_ref()
        .replace("[/]", if cfg!(windows) { "\\" } else { "/" })
}

fn equate_to_expected(g: GlobWalker, mut expected: Vec<String>, dir_path: &Path) {
    for matched_file in g.into_iter().filter_map(Result::ok) {
        let path = matched_file
            .path()
            .strip_prefix(dir_path)
            .unwrap()
            .to_str()
            .unwrap();
        let path = normalize_path_sep(path);

        let del_idx = if let Some(idx) = expected.iter().position(|n| &path == n) {
            idx
        } else {
            panic!("Iterated file is unexpected: {}", path);
        };

        expected.remove(del_idx);
    }

    // Not equating `.len() == 0` so that the assertion output
    // will contain the extra files
    let empty: &[&str] = &[][..];
    assert_eq!(expected, empty);
}

// TODO
#[test]
fn test_1() {
    let dir = TempDir::new().expect("Failed to create temporary folder");
    let dir_path = dir.path();
    create_dir_all(dir_path.join("src/some_mod")).expect("");
    create_dir_all(dir_path.join("tests")).expect("");
    create_dir_all(dir_path.join("contrib")).expect("");

    touch(
        &dir,
        &[
            "a.rs",
            "b.rs",
            "avocado.rs",
            "lib.c",
            "src[/]hello.rs",
            "src[/]world.rs",
            "src[/]some_mod[/]unexpected.rs",
            "src[/]cruel.txt",
            "contrib[/]README.md",
            "contrib[/]README.rst",
            "contrib[/]lib.rs",
        ][..],
    );

    let expected: Vec<_> = [
        "src[/]some_mod[/]unexpected.rs",
        "src[/]world.rs",
        "src[/]hello.rs",
        "lib.c",
        "contrib[/]lib.rs",
        "contrib[/]README.md",
        "contrib[/]README.rst",
    ]
    .iter()
    .map(normalize_path_sep)
    .collect();

    let patterns = ["src/**/*.rs", "*.c", "**/lib.rs", "**/*.{md,rst}"];
    let glob = GlobWalkerBuilder::from_patterns(dir_path, &patterns)
        .build()
        .unwrap();

    //equate_to_expected(glob, expected, dir_path);

    ()
}
