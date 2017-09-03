# shellwords

Crate **shellwords** provides utilities for parsing strings as they would be interpreted by the UNIX Bourne shell.

* [shellwords](https://crates.io/crates/shellwords) on crates.io
* [Documentation](https://docs.rs/shellwords) for the latest crates.io release

## Examples

Split a string into a vector of words in the same way the UNIX Bourne shell does:

``` rust
assert_eq!(split("here are \"two words\"").unwrap(), ["here", "are", "two words"]);
```

## Legal

shellwords is released under the MIT license.
See `LICENSE` for details.
