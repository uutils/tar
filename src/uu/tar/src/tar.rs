// This file is part of the uutils tar package.
//
// For the full copyright and license information, please view the LICENSE
// file that was distributed with this source code.

pub mod errors;
mod operations;
mod options;

use clap::{arg, crate_version, Arg, ArgAction, ArgGroup, Command, error::ErrorKind};
use operations::operation::TarOperation;
use options::TarOptions;
use std::path::PathBuf;
use uucore::error::{UResult, USimpleError};
use uucore::format_usage;

const ABOUT: &str = "an archiving utility";
const USAGE: &str = "tar {A|c|d|r|t|u|x}[GnSkUWOmpsMBiajJzZhPlRvwo] -f ARCHIVE [FILES...]";

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

    let matches = uu_app().try_get_matches_from(args_to_parse)?;

    let (op, options) = TarOptions::with_operation(&matches)?;

    op.exec(&options)

    // // Handle extract operation
    // if matches.get_flag("extract") {
    //     let archive_path = matches.get_one::<PathBuf>("file").ok_or_else(|| {
    //         uucore::error::USimpleError::new(1, "tar: option requires an argument -- 'f'")
    //     })?;
    //
    //     return operations::extract::extract_archive(archive_path, verbose);
    // }
    //
    // // Handle create operation
    // if matches.get_flag("create") {
    //     let archive_path = matches.get_one::<PathBuf>("file").ok_or_else(|| {
    //         uucore::error::USimpleError::new(1, "tar: option requires an argument -- 'f'")
    //     })?;
    //     // NOTE: changing to path buf
    //     let files: Vec<PathBuf> = matches
    //         .get_many::<PathBuf>("files")
    //         .expect("No files listed to create archive from")
    //         .map(|x| x.to_owned())
    //         .collect();
    //
    //     if files.is_empty() {
    //         return Err(uucore::error::USimpleError::new(
    //             1,
    //             "tar: Cowardly refusing to create an empty archive",
    //         ));
    //     }
    //
    //     return operations::create::create_archive(archive_path, &files, verbose);
    // }

    // If no operation specified, show error
    // Err(uucore::error::USimpleError::new(
    //     1,
    //     "tar: You must specify one of the '-c' or '-x' options",
    // ))
}
#[allow(clippy::cognitive_complexity)]
pub fn uu_app() -> Command {
    Command::new(uucore::util_name())
        .version(crate_version!())
        .about(ABOUT)
        // Since -h flag is used for --dereference for some reason in GNU tar?
        .disable_help_flag(true)
        .override_usage(format_usage(USAGE))
        .infer_long_args(true)
        .subcommand_required(true)
        .subcommands([
            Command::new("catenate").short_flag('A').long_flag("catenate"),
            Command::new("create").short_flag('c').long_flag("create"),
            Command::new("diff").short_flag('d').long_flag("diff"),
            Command::new("list").short_flag('t').long_flag("list"),
            Command::new("append").short_flag('r').long_flag("append"),
            Command::new("update").short_flag('u').long_flag("update"),
            Command::new("extract").short_flag('x').long_flag("extract"),
            Command::new("delete").long_flag("delete"),
        ])
        .subcommand_help_heading("Operation Modes")
        .args([

            // // Main operation modes
            // arg!(-A --catenate "Append tar files to archive").group("operations").requires("files"),
            // arg!(-c --create "Create a new archive").group("operations").requires("files"),
            // arg!(-d --diff "Find differences between archive and file system")
            //     .alias("compare")
            //     .group("operations"),
            // arg!(-r --append "Append files to end of archive").group("operations").requires("files"),
            // arg!(-t --list "List contents of archive").group("operations"),
            // arg!(-u --update "Only append files newer than copy in archive").group("operations").requires("files"),
            // arg!(-x --extract "Extract files from archive")
            //     .alias("get")
            //     .group("operations"),

            // Archive file
            Arg::new("archive")
                .long("file")
                .short('f')
                .help("Use archive file")
                .value_parser(clap::value_parser!(PathBuf))
                .global(true),
            
            // Compression options
            arg!(-z --gzip "Filter through gzip"),
            arg!(-j --bzip2 "Filter through bzip2"),
            arg!(-J --xz "Filter through xz"),

            // Common options
            arg!(-v --verbose "Verbosely list files processed").global(true),
            arg!(-h --dereference "Follow symlinks"),

            // custom long help
            Arg::new("help").long("help").action(ArgAction::Help),

            // arg macro has an issue with the '-' in the middle of the long args
            Arg::new("preserve-permissions")
                .short('p')
                .long("preserve-permissions")
                .action(clap::ArgAction::SetTrue)
                .group("options"),
            Arg::new("absolute-names")
                .short('P')
                .long("absolute-names")
                .action(clap::ArgAction::SetTrue)
                .group("options"),

            // Files to process
            Arg::new("files")
                .help("Files to archive or extract")
                .value_parser(clap::value_parser!(PathBuf))
                .num_args(0..)
                .group("options") 
                .global(true)
        ])
}

// #[allow(clippy::cognitive_complexity)]
// pub fn uu_app() -> Command {
//     Command::new(uucore::util_name())
//         .version(crate_version!())
//         .about(ABOUT)
//         .override_usage(format_usage(USAGE))
//         .infer_long_args(true)
//         .disable_help_flag(true)
//         .args([
//             // Main operation modes
//             arg!(-c --create "Create a new archive"),
//             arg!(-x --extract "Extract files from archive").alias("get"),
//             // Archive file
//             arg!(-f --file <ARCHIVE> "Use archive file or device ARCHIVE")
//                 .value_parser(clap::value_parser!(PathBuf))
//                 .required(false),
//             // Common options
//             arg!(-v --verbose "Verbosely list files processed"),
//             // Help
//             Arg::new("help")
//                 .long("help")
//                 .help("Print help information")
//                 .action(ArgAction::Help),
//             // Files to process
//             Arg::new("files")
//                 .help("Files to archive or extract")
//                 .action(ArgAction::Append)
//                 .value_parser(clap::value_parser!(PathBuf))
//                 .num_args(0..),
//         ])
// }
