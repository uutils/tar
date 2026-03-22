// This file is part of the uutils tar package.
//
// For the full copyright and license information, please view the LICENSE
// file that was distributed with this source code.

pub mod errors;
mod operations;

use clap::{arg, crate_version, ArgAction, Command};
use std::path::{Path, PathBuf};
use uucore::error::UResult;
use uucore::format_usage;

const ABOUT: &str = "an archiving utility";
const USAGE: &str = "tar key [FILE...]\n       tar {-c|-t|-x} [-v] -f ARCHIVE [FILE...]";

/// Determines whether a string looks like a POSIX tar keystring.
///
/// A valid keystring must not start with '-', must contain at least one
/// function letter (c, x, t, u, r), and every character must be a
/// recognised key character.
fn is_posix_keystring(s: &str) -> bool {
    if s.is_empty() || s.starts_with('-') {
        return false;
    }
    let valid_chars = "cxturvwfblmo";
    // function letters: c=create, x=extract, t=list, u=update, r=append
    // modifier letters: v=verbose, w=interactive, f=file, b=blocking-factor,
    //                   l=one-file-system, m=modification-time, o=no-same-owner
    s.chars().all(|c| valid_chars.contains(c)) && s.chars().any(|c| "cxtur".contains(c))
}

/// Expands a POSIX tar keystring at `args[1]` into flag-style arguments
/// suitable for clap.
///
/// Per the POSIX spec the key operand is a function letter optionally
/// followed by modifier letters.  Modifier letters `f` and `b` consume
/// the leading file operands (in the order they appear in the key).
/// GNU tar is more permissive and accepts non-standard ordering (for
/// example `fcv`/`vcf`), so we intentionally accept that compatibility mode.
// Keep argv as `OsString` so non-UTF-8/path-native arguments are preserved.
fn expand_posix_keystring(args: Vec<std::ffi::OsString>) -> Vec<std::ffi::OsString> {
    // Only expand when args[1] is valid UTF-8 and looks like a keystring
    let key = match args.get(1).and_then(|s| s.to_str()) {
        Some(s) if is_posix_keystring(s) => s.to_string(),
        _ => return args,
    };

    // args[2..] are the raw file operands (archive name, blocking factor, files)
    let file_operands = &args[2..];
    let mut result: Vec<std::ffi::OsString> = vec![args[0].clone()];
    let mut file_idx = 0; // how many file operands have been consumed

    for c in key.chars() {
        match c {
            'f' => {
                // Next file operand is the archive name
                result.push(std::ffi::OsString::from("-f"));
                if file_idx < file_operands.len() {
                    result.push(file_operands[file_idx].clone());
                    file_idx += 1;
                }
            }
            'b' => {
                // Preserve parity with dash-style parsing by forwarding '-b'
                // and its operand (when present). Since '-b' is currently
                // unsupported, clap will report it as an unknown argument.
                result.push(std::ffi::OsString::from("-b"));
                if file_idx < file_operands.len() {
                    result.push(file_operands[file_idx].clone());
                    file_idx += 1;
                }
            }
            other => {
                result.push(std::ffi::OsString::from(format!("-{other}")));
            }
        }
    }

    // Any remaining file operands are the files to archive/extract
    result.extend_from_slice(&file_operands[file_idx..]);
    result
}

#[uucore::main]
pub fn uumain(args: impl uucore::Args) -> UResult<()> {
    // Collect args - the test framework may add util_name as args[1], so skip it if present
    let args_vec: Vec<_> = args.collect();
    let util_name = uucore::util_name();

    // Skip duplicate util name if present (can be "tar" or "tarapp")
    let args_to_parse = if args_vec.len() > 1
        && (args_vec[1] == util_name || args_vec[1] == "tar" || args_vec[1] == "tarapp")
    {
        let mut result = vec![args_vec[0].clone()];
        result.extend_from_slice(&args_vec[2..]);
        result
    } else {
        args_vec
    };

    // Support POSIX keystring syntax: `tar cvf archive.tar files…`
    // where the first operand is a key rather than a flag-prefixed option.
    let args_to_parse = expand_posix_keystring(args_to_parse);

    let matches = match uu_app().try_get_matches_from(args_to_parse) {
        Ok(matches) => matches,
        Err(err) => {
            let kind = err.kind();
            match kind {
                clap::error::ErrorKind::DisplayHelp | clap::error::ErrorKind::DisplayVersion => {
                    let _ = err.print();
                    return Ok(());
                }
                _ => {
                    let code = match kind {
                        clap::error::ErrorKind::ArgumentConflict => 2,
                        clap::error::ErrorKind::UnknownArgument
                        | clap::error::ErrorKind::MissingRequiredArgument
                        | clap::error::ErrorKind::TooFewValues
                        | clap::error::ErrorKind::TooManyValues
                        | clap::error::ErrorKind::WrongNumberOfValues => 64,
                        clap::error::ErrorKind::InvalidValue
                        | clap::error::ErrorKind::ValueValidation => 2,
                        _ => 2,
                    };
                    return Err(uucore::error::USimpleError::new(code, err.to_string()));
                }
            }
        }
    };

    let verbose = matches.get_flag("verbose");

    // Handle extract operation
    if matches.get_flag("extract") {
        let archive_path = matches.get_one::<PathBuf>("file").ok_or_else(|| {
            uucore::error::USimpleError::new(64, "option requires an argument -- 'f'")
        })?;

        return operations::extract::extract_archive(archive_path, verbose);
    }

    // Handle create operation
    if matches.get_flag("create") {
        let archive_path = matches.get_one::<PathBuf>("file").ok_or_else(|| {
            uucore::error::USimpleError::new(64, "option requires an argument -- 'f'")
        })?;

        let files: Vec<&Path> = matches
            .get_many::<PathBuf>("files")
            .map(|v| v.map(|p| p.as_path()).collect())
            .unwrap_or_default();

        if files.is_empty() {
            return Err(uucore::error::USimpleError::new(
                2,
                "Cowardly refusing to create an empty archive",
            ));
        }

        return operations::create::create_archive(archive_path, &files, verbose);
    }

    // Handle list operation
    if matches.get_flag("list") {
        let archive_path = matches.get_one::<PathBuf>("file").ok_or_else(|| {
            uucore::error::USimpleError::new(64, "option requires an argument -- 'f'")
        })?;

        return operations::list::list_archive(archive_path, verbose);
    }

    // If no operation specified, show error
    Err(uucore::error::USimpleError::new(
        2,
        "You must specify one of the '-c', '-x', or '-t' options",
    ))
}

#[allow(clippy::cognitive_complexity)]
pub fn uu_app() -> Command {
    Command::new(uucore::util_name())
        .version(crate_version!())
        .about(ABOUT)
        .override_usage(format_usage(USAGE))
        .infer_long_args(true)
        .disable_help_flag(true)
        .args([
            // Main operation modes
            arg!(-c --create "Create a new archive").conflicts_with_all(["extract", "list"]),
            // arg!(-d --diff "Find differences between archive and file system").alias("compare"),
            // arg!(-r --append "Append files to end of archive"),
            arg!(-t --list "List contents of archive").conflicts_with_all(["create", "extract"]),
            // arg!(-u --update "Only append files newer than copy in archive"),
            arg!(-x --extract "Extract files from archive")
                .alias("get")
                .conflicts_with_all(["create", "list"]),
            // Archive file
            arg!(-f --file <ARCHIVE> "Use archive file or device ARCHIVE")
                .value_parser(clap::value_parser!(PathBuf)),
            // Compression options
            // arg!(-z --gzip "Filter through gzip"),
            // arg!(-j --bzip2 "Filter through bzip2"),
            // arg!(-J --xz "Filter through xz"),
            // Common options
            arg!(-v --verbose "Verbosely list files processed"),
            // arg!(-h --dereference "Follow symlinks"),
            // arg!(-p --"preserve-permissions" "Extract information about file permissions"),
            // arg!(-P --"absolute-names" "Don't strip leading '/' from file names"),
            // Help
            arg!(--help "Print help information").action(ArgAction::Help),
            // Files to process
            arg!([files]... "Files to archive or extract")
                .action(ArgAction::Append)
                .value_parser(clap::value_parser!(PathBuf)),
        ])
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- is_posix_keystring ---

    #[test]
    fn test_keystring_create() {
        assert!(is_posix_keystring("c"));
        assert!(is_posix_keystring("cf"));
        assert!(is_posix_keystring("cvf"));
        assert!(is_posix_keystring("cv"));
    }

    #[test]
    fn test_keystring_extract() {
        assert!(is_posix_keystring("x"));
        assert!(is_posix_keystring("xf"));
        assert!(is_posix_keystring("xvf"));
    }

    #[test]
    fn test_keystring_rejects_dash_prefix() {
        assert!(!is_posix_keystring("-c"));
        assert!(!is_posix_keystring("-cf"));
        assert!(!is_posix_keystring("-xvf"));
    }

    #[test]
    fn test_keystring_rejects_no_function_letter() {
        // modifier-only strings are not valid keystrings
        assert!(!is_posix_keystring("f"));
        assert!(!is_posix_keystring("vf"));
        assert!(!is_posix_keystring("v"));
    }

    #[test]
    fn test_keystring_rejects_invalid_chars() {
        assert!(!is_posix_keystring("cz")); // 'z' is not a key char
        assert!(!is_posix_keystring("c1")); // digits not allowed
        assert!(!is_posix_keystring("archive.tar")); // typical filename
    }

    #[test]
    fn test_keystring_rejects_empty() {
        assert!(!is_posix_keystring(""));
    }

    // --- expand_posix_keystring ---

    fn osvec(v: &[&str]) -> Vec<std::ffi::OsString> {
        v.iter().map(std::ffi::OsString::from).collect()
    }

    #[test]
    fn test_expand_cf() {
        let input = osvec(&["tar", "cf", "archive.tar", "file.txt"]);
        let expected = osvec(&["tar", "-c", "-f", "archive.tar", "file.txt"]);
        assert_eq!(expand_posix_keystring(input), expected);
    }

    #[test]
    fn test_expand_cvf() {
        let input = osvec(&["tar", "cvf", "archive.tar", "file.txt"]);
        let expected = osvec(&["tar", "-c", "-v", "-f", "archive.tar", "file.txt"]);
        assert_eq!(expand_posix_keystring(input), expected);
    }

    #[test]
    fn test_expand_xf() {
        let input = osvec(&["tar", "xf", "archive.tar"]);
        let expected = osvec(&["tar", "-x", "-f", "archive.tar"]);
        assert_eq!(expand_posix_keystring(input), expected);
    }

    #[test]
    fn test_expand_xvf() {
        let input = osvec(&["tar", "xvf", "archive.tar"]);
        let expected = osvec(&["tar", "-x", "-v", "-f", "archive.tar"]);
        assert_eq!(expand_posix_keystring(input), expected);
    }

    #[test]
    fn test_expand_preserves_dash_prefix_args() {
        // When args already use '-' prefixes, no expansion should occur
        let input = osvec(&["tar", "-cvf", "archive.tar", "file.txt"]);
        assert_eq!(expand_posix_keystring(input.clone()), input);
    }

    #[test]
    fn test_expand_f_before_files() {
        // 'f' consumes only the archive name; remaining args are files
        let input = osvec(&["tar", "cf", "archive.tar", "a.txt", "b.txt"]);
        let expected = osvec(&["tar", "-c", "-f", "archive.tar", "a.txt", "b.txt"]);
        assert_eq!(expand_posix_keystring(input), expected);
    }

    #[test]
    fn test_expand_function_letter_only() {
        // No 'f' modifier: no archive consumed from file operands
        let input = osvec(&["tar", "c", "file.txt"]);
        let expected = osvec(&["tar", "-c", "file.txt"]);
        assert_eq!(expand_posix_keystring(input), expected);
    }

    #[test]
    fn test_expand_cbf() {
        let input = osvec(&["tar", "cbf", "20", "archive.tar", "file.txt"]);
        let expected = osvec(&["tar", "-c", "-b", "20", "-f", "archive.tar", "file.txt"]);
        assert_eq!(expand_posix_keystring(input), expected);
    }
}
