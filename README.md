[![Crates.io](https://img.shields.io/crates/v/uu_tar.svg)](https://crates.io/crates/uu_tar)
[![Discord](https://img.shields.io/badge/discord-join-7289DA.svg?logo=discord&longCache=true&style=flat)](https://discord.gg/wQVJbvJ)
[![License](http://img.shields.io/badge/license-MIT-blue.svg)](https://github.com/uutils/tar/blob/main/LICENSE)
[![dependency status](https://deps.rs/repo/github/uutils/tar/status.svg)](https://deps.rs/repo/github/uutils/tar)

[![CodeCov](https://codecov.io/gh/uutils/tar/branch/master/graph/badge.svg)](https://codecov.io/gh/uutils/tar)

# tar

Rust reimplementation of the tar utility.

## Installation

Ensure you have Rust installed on your system. You can install Rust through [rustup](https://rustup.rs/).

Clone the repository and build the project using Cargo:

```bash
git clone https://github.com/uutils/tar.git
cd tar
cargo build --release
cargo run --release
```

## Testing

The tar application has a focused testing philosophy that separates concerns between the application (CLI interface, error handling, user experience) and the underlying `tar-rs` library (archive format correctness, encoding, permissions).

See [tests/README.md](tests/README.md) for comprehensive documentation.

```bash
# Run all tests
cargo test --all

# Run specific test
cargo test test_create_single_file
```

## License

tar is licensed under the MIT License - see the `LICENSE` file for details
