// This file is part of the uutils tar package.
//
// For the full copyright and license information, please view the LICENSE
// file that was distributed with this source code.

pub mod errors;
mod operations;
mod options;

use clap::{arg, crate_version, error::ErrorKind, value_parser, Arg, ArgAction, ArgGroup, Command};
use operations::operation::TarOperation;
use options::TarParams;
use std::path::PathBuf;
use uucore::error::{UResult, USimpleError};
use uucore::format_usage;

const ABOUT: &str = "an archiving utility";
const USAGE: &str = "tar {A|c|d|r|t|u|x}[GnSkUWOmpsMBiajJzZhPlRvwo] -f ARCHIVE [FILES...]";
const BLOCK_SIZE: usize = 512;

#[uucore::main]
pub fn uumain(args: impl uucore::Args) -> UResult<()> {
    // // Collect args - the test framework may add util_name as args[1], so skip it if present
    // let args_vec: Vec<_> = args.collect();
    // let util_name = uucore::util_name();
    //
    // // Skip duplicate util name if present (can be "tar" or "tarapp")
    // let args_to_parse = if args_vec.len() > 1
    //     && (args_vec[1] == util_name || args_vec[1] == "tar" || args_vec[1] == "tarapp")
    // {
    //     let mut result = vec![args_vec[0].clone()];
    //     result.extend_from_slice(&args_vec[2..]);
    //     result
    // } else {
    //     args_vec
    // };

    let matches =
        match uu_app().try_get_matches_from(args) {
            Ok(m) => m,
            Err(e) => match e.kind() {
                ErrorKind::MissingSubcommand => return Err(USimpleError::new(
                    1,
                    "You must specify one of the '-Acdtrux', '--delete' or '--test-label' options",
                )),
                ErrorKind::MissingRequiredArgument => {
                    return Err(USimpleError::new(
                        1,
                        format!(
                            "option requires an argument {}",
                            e.get(clap::error::ContextKind::InvalidArg)
                                .unwrap()
                                .to_string()
                        ),
                    ));
                }
                _ => return Err(e.into()),
            },
        };

    let (op, options) = TarParams::with_operation(&matches)?;

    op.exec(&options)
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
            Command::new("catenate")
                .short_flag('A')
                .long_flag("catenate")
                .long_flag("concatenate")
                .arg(
                    Arg::new("archives")
                        .help("Files to archive or extract")
                        .value_parser(clap::value_parser!(PathBuf))
                        .num_args(0..)
                        .required(true),
                    .requires("archive"),
                ),
            Command::new("create")
                .short_flag('c')
                .long_flag("create")
                .arg(
                    Arg::new("files")
                        .help("Files to archive or extract")
                        .value_parser(clap::value_parser!(PathBuf))
                        .num_args(0..)
                        .required(true),
                    .requires("archive"),
                ),
            Command::new("diff")
                .short_flag('d')
                .long_flag("diff")
                .long_flag("compare")
                .arg(
                    Arg::new("files")
                        .help("Files to archive or extract")
                        .value_parser(clap::value_parser!(PathBuf))
                        .num_args(0..)
                        .required(true),
                    .requires("archive"),
                ),
            Command::new("list").short_flag('t').long_flag("list").arg(
                Arg::new("members")
                    .help("Archive members to list")
                    .value_parser(clap::value_parser!(PathBuf))
                    .num_args(0..)
                    .last(true).requires("archive")
            ),
            Command::new("append")
                .short_flag('r')
                .long_flag("append")
                .arg(
                    Arg::new("files")
                        .help("Files to archive or extract")
                        .value_parser(clap::value_parser!(PathBuf))
                        .num_args(0..)
                        .required(true),
                    .requires("archive"),
                ),
            Command::new("update")
                .short_flag('u')
                .long_flag("update")
                .arg(
                    Arg::new("files")
                        .help("Files to archive or extract")
                        .value_parser(clap::value_parser!(PathBuf))
                        .num_args(0..)
                        .required(true),
                    .requires("archive"),
                ),
            Command::new("extract")
                .short_flag('x')
                .long_flag("extract")
                .long_flag("get")
                .arg(
                    Arg::new("members")
                        .help("Files to archive or extract")
                        .value_parser(clap::value_parser!(PathBuf))
                        .num_args(0..)
                        .required(true),
                    .requires("archive"),
                ),
            Command::new("delete").long_flag("delete").arg(
                Arg::new("members")
                    .help("Files to archive or extract")
                    .value_parser(clap::value_parser!(PathBuf))
                    .num_args(0..)
                    .required(true),
                    .requires("archive"),
            ),
            Command::new("test-label").long_flag("test-label").arg(
                Arg::new("label")
                    .help("Files to archive or extract")
                    .value_parser(value_parser!(PathBuf))
                    .num_args(0..)
                    .required(true)
                    .requires("archive"),
            ),
        ])
        .subcommand_help_heading("Operation Modes")
        .args([
            // Archive file
            Arg::new("archive")
                .value_name("FILE")
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
                .action(clap::ArgAction::SetTrue),
        ])
}
