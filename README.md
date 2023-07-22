I started making this a couple hours ago as of writing this so I haven't come that far. Currently it searches and replaces string contents only but will be improved to also replace filenames and so the flag works.

A small CLI utility written in Rust that helps with searching and replacing filenames and file contents recursively using regex and glob patterns.

```
A small CLI utility written in Rust that helps with searching and replacing filenames and file contents recursively using regex and glob patterns.

Usage: reno.exe [OPTIONS]

Options:
  -G, --glob-patterns <GLOB_PATTERNS>
          Filename glob patterns, defaults to: "*"
  -S <SEARCH_REGEX>
          Search regex
  -R <REPLACE_REGEX>
          Replace regex
  -c, --contents
          (NOT IMPLEMENTED) Only search and replace file contents
  -n, --names
          (NOT IMPLEMENTED) Only search and replace file names
  -d, --dry
          (NOT IMPLEMENTED) Don't modify files, just show what would happen
  -m, --max-depth <MAX_DEPTH>
          Max depth of directory traversal, unlimited by default. 0 means only the current directory
  -p, --prefix
          (NOT IMPLEMENTED) Prepends the replacement to the start of all matched strings, <REPLACE_REGEX> will be treated like a normal string.
  -s, --suffix
          (NOT IMPLEMENTED) Appends the replacement to the end of all matched strings, <REPLACE_REGEX> will be treated like a normal string.
  -h, --help
          Print help
  -V, --version
          Print version
```
