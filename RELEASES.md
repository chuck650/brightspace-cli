# Releases

This directory contains pre-built binaries for the Brightspace CLI.

## Current Release: v0.1.0

- **Linux (x86_64)**: `brightspace-cli-linux-x86_64`

## Installation

Download the binary for your platform, rename it to `brightspace-cli` (or `brightspace-cli.exe` on Windows), and place it in a directory in your system's `PATH`.

### Linux / macOS
```bash
chmod +x brightspace-cli
sudo mv brightspace-cli /usr/local/bin/
```

## Build from Source

If a binary is not available for your platform, you can build from source using Rust and Cargo:

```bash
cargo install --path .
```

## Future Roadmap

We plan to provide pre-built binaries for the following platforms in future releases:
- Windows (x86_64)
- macOS (Apple Silicon & Intel)
- Linux (ARM64)

## Release Process (For Maintainers)

To create a new release:

1.  Update the version in `Cargo.toml`.
2.  Run tests: `cargo test`.
3.  Build release binary: `cargo build --release`.
4.  Tag the release in git: `git tag -a v0.1.0 -m "Release v0.1.0"`.
5.  Copy the binary to the `releases/` folder with the platform suffix.
