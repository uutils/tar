# Testing the tar Application

This directory contains tests for the `tar` command-line utility.

## Philosophy

The `tar` utility is built on top of the `tar-rs` library. Because of this, we split our testing into two distinct areas:

1.  **The Library (`tar-rs`)**: This is where we test the nitty-gritty details of the tar format. If you want to verify that permissions are preserved correctly, that long paths are handled according to the UStar spec, or that unicode filenames are encoded properly, those tests belong in `tar-rs/tests/`.

2.  **The Application (`tar`)**: This is where we test the user interface. These tests ensure that the command-line arguments are parsed correctly, that the program exits with the right status codes, and that basic operations like creating and extracting archives actually work from a user's perspective.

## Writing Tests for the Application

When writing tests here, focus on the **user experience**.

*   **Do** check that flags like `-c`, `-x`, `-v`, and `-f` do what they say.
*   **Do** check that invalid combinations of flags produce a helpful error message and a usage exit code (64).
*   **Do** check that serious errors (like file not found) return exit code 2.
*   **Do** perform "smoke tests" — create an archive and make sure the file appears; extract an archive and make sure the files come out.

*   **Don't** inspect the internal bytes of the archive to verify header fields. Trust that `tar-rs` handles that.
*   **Don't** write complex tests for edge cases in file system permissions or encoding, unless they are specifically related to a CLI flag.

### Example

If you are testing that `tar -cf archive.tar file.txt` works:

*   **Good**: Run the command, assert it succeeds (exit code 0), and assert that `archive.tar` exists on disk.
*   **Bad**: Run the command, open `archive.tar` with a library, parse the headers, and assert that the checksum is correct.

## Running Tests

You can run these tests just like any other Rust project:

```bash
cargo test --all
```

To run a specific test:

```bash
cargo test test_name
```

## Exit Codes

We follow GNU tar conventions for exit codes:

*   **0**: Success.
*   **1**: Some files differ (used in compare mode).
*   **2**: Fatal error (file not found, permission denied, etc.).
*   **64**: Usage error (invalid flags, bad syntax).
