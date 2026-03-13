# timeout

A Rust implementation of the `timeout` command, based on GNU coreutils 9.4.

## Overview

The `timeout` command runs a specified command with a time limit. If the command runs longer than the specified time, it will be automatically terminated.

## Features

- Run commands with a specified time limit
- Customize the signal sent on timeout (default: TERM)
- Option to send a KILL signal after a grace period
- Preserve command exit status even when timed out
- Foreground mode for TTY interaction
- Verbose output for debugging

## Installation

### Prerequisites
- Rust 1.70+ (for 2024 edition support)
- Cargo package manager

### Build from source

```bash
git clone <repository-url>
cd timeout
cargo build --release

# Install to system (optional)
cargo install --path .
```

## Usage

### Basic Usage

```bash
# Run command with 10 second timeout
timeout 10s command [args...]

# Run command with 1 minute timeout
timeout 1m command [args...]
```

### Options

| Option | Long Option | Description |
|--------|-------------|-------------|
| `-k` | `--kill-after=DURATION` | If command doesn't exit after initial signal, send KILL signal |
| `-s` | `--signal=SIGNAL` | Specify signal to send on timeout (name or number) |
| `-v` | `--verbose` | Print signal information to stderr |
| | `--preserve-status` | Exit with command's status even when timed out |
| | `--foreground` | Allow command to read from TTY and get TTY signals |

### Time Format

Durations can be specified in the following formats:
- `10` - 10 seconds
- `10s` - 10 seconds
- `10m` - 10 minutes
- `10h` - 10 hours
- `10d` - 10 days

### Examples

```bash
# Run command with 5 second timeout, then send KILL after 2 more seconds
timeout -k 2s 5s command

# Run command with custom signal (HUP)
timeout -s HUP 10s command

# Run command with timeout and preserve exit status
timeout --preserve-status 10s command

# Run command in foreground mode
timeout --foreground 10s command

# Run with verbose output
timeout -v 10s command
```

## Exit Status

| Status | Meaning |
|--------|---------|
| 124 | Command timed out and `--preserve-status` not set |
| 125 | timeout command itself failed |
| 126 | Command found but cannot be invoked |
| 127 | Command not found |
| 137 | Command killed by KILL signal |
| Other | Command's original exit status |

## Building

```bash
# Build in debug mode
cargo build

# Build in release mode
cargo build --release
```

## Testing

```bash
cargo test
```

## Implementation Details

- Written in Rust 2024 edition
- Uses `tokio` for async execution
- Uses `clap` for command line parsing
- Uses `libc` for signal handling
- Supports cross-platform operation (Linux, macOS, Unix)

## Specification

This implementation is based on the specifications in `spec.md`, which details the functionality, command line options, and expected behavior of the timeout command.

## License

[MIT](LICENSE)
