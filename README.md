Name is short for *Renovate*. A simple but powerful command-line batch file editor. Enables you to use regex search and replace on both filenames and contents, efficiently (multi-threaded) and cross-platform.

To see it in action run
`cargo run -- -S "test(\.md|\.txt)" -R "changed_test$1"` and then check the filenames and contents of hello henrik and test.md.

More examples:
`reno -G *test.* -R "changed_test" --names` - hello henrik becomes changed_test and test.md becomes changed_test, one of them will be overwritten depending on which thread wins.
Is somewhat of a bug and I'll do some check for it but in the meantime be aware of that.

`reno -S "^(FolderPrefix?)([^\.]*)$" -G **/** -R $2 --names` - Recursively removes the string FolderPrefix in the beginning of all folder names
```
A small CLI utility written in Rust that helps with searching and replacing filenames and file contents recursively using regex and glob patterns.

Usage: reno.exe [OPTIONS]

Options:
  -G, --glob-patterns <GLOB_PATTERNS>
          Filename glob patterns, defaults to: "./*.*"
  -S <SEARCH_REGEX>
          Search regex
  -R <REPLACE_REGEX>
          Replace regex
  -d, --dry
          Don't modify files, just show what would happen
  -c, --contents
          Only search and replace file contents
  -n, --names
          Only search and replace file names
  -m, --max-depth <MAX_DEPTH>
          Max depth of directory traversal, unlimited by default. 0 means only the current directory
  -h, --help
          Print help
  -V, --version
          Print version
```
