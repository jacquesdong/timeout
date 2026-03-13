use clap::Parser;
use std::process::ExitStatus;
use tokio::process::Command;
use tokio::time::{Duration, sleep};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    /// Duration after which to kill the command
    #[arg(short = 'k', long = "kill-after")]
    pub kill_after: Option<String>,

    /// Signal to send on timeout
    #[arg(short = 's', long = "signal", default_value = "TERM")]
    pub signal: String,

    /// Diagnose to stderr any signal sent upon timeout
    #[arg(short = 'v', long = "verbose")]
    pub verbose: bool,

    /// Exit with the same status as COMMAND, even when the command times out
    #[arg(long = "preserve-status")]
    pub preserve_status: bool,

    /// When not running timeout directly from a shell prompt, allow COMMAND to read from the TTY and get TTY signals
    #[arg(long = "foreground")]
    pub foreground: bool,

    /// Duration before sending signal
    pub duration: Option<String>,

    /// Command to run
    pub command: Option<String>,

    /// Arguments to pass to the command
    pub args: Vec<String>,
}

/// Parse duration string to seconds
/// Supported formats: "10", "10s", "10m", "10h", "10d"
fn parse_duration(duration: &str) -> Result<f64, String> {
    let duration = duration.trim();

    if duration == "0" {
        return Ok(0.0);
    }

    let (num_str, unit) = if duration.ends_with('s') {
        (&duration[..duration.len() - 1], 1.0)
    } else if duration.ends_with('m') {
        (&duration[..duration.len() - 1], 60.0)
    } else if duration.ends_with('h') {
        (&duration[..duration.len() - 1], 3600.0)
    } else if duration.ends_with('d') {
        (&duration[..duration.len() - 1], 86400.0)
    } else {
        (duration, 1.0)
    };

    match num_str.parse::<f64>() {
        Ok(num) => Ok(num * unit),
        Err(_) => Err(format!("Invalid duration format: {}", duration)),
    }
}

/// Convert signal name or number to libc signal constant
fn parse_signal(signal: &str) -> Result<i32, String> {
    // Try to parse as number first
    if let Ok(num) = signal.parse::<i32>() {
        return Ok(num);
    }

    // Try to parse as signal name
    match signal.to_uppercase().as_str() {
        "TERM" => Ok(libc::SIGTERM),
        "HUP" => Ok(libc::SIGHUP),
        "INT" => Ok(libc::SIGINT),
        "KILL" => Ok(libc::SIGKILL),
        "QUIT" => Ok(libc::SIGQUIT),
        "ALRM" => Ok(libc::SIGALRM),
        "USR1" => Ok(libc::SIGUSR1),
        "USR2" => Ok(libc::SIGUSR2),
        _ => Err(format!("Invalid signal: {}", signal)),
    }
}

#[cfg(test)]
mod tests {
    include!("tests/mod.rs");
}

/// Run command with timeout
async fn run_with_timeout(args: &Args) -> Result<(ExitStatus, bool), String> {
    let Some(ref command) = args.command else {
        return Err("Command is required".to_string());
    };

    let Some(ref duration_str) = args.duration else {
        return Err("Duration is required".to_string());
    };

    let duration_secs = parse_duration(duration_str)?;
    if duration_secs <= 0.0 {
        // Duration 0 means no timeout
        let mut cmd = Command::new(command);
        for arg in &args.args {
            cmd.arg(arg);
        }
        match cmd.status().await {
            Ok(status) => Ok((status, false)),
            Err(e) => Err(format!("Failed to execute command: {}", e)),
        }
    } else {
        let mut cmd = Command::new(command);
        for arg in &args.args {
            cmd.arg(arg);
        }

        let mut child = match cmd.spawn() {
            Ok(child) => child,
            Err(e) => return Err(format!("Failed to spawn command: {}", e)),
        };

        let duration = Duration::from_secs_f64(duration_secs);
        tokio::select! {
            result = child.wait() => {
                match result {
                    Ok(status) => Ok((status, false)),
                    Err(e) => Err(format!("Failed to wait for command: {}", e)),
                }
            },
            _ = sleep(duration) => {
                // Timeout occurred
                if args.verbose {
                    eprintln!("Sending {} signal to command", args.signal);
                }

                // Send initial signal
                let signal = parse_signal(&args.signal)?;
                let Some(pid) = child.id() else {
                    return Err("Failed to get command PID".to_string());
                };
                unsafe {
                    if libc::kill(pid as i32, signal) != 0 {
                        return Err(format!("Failed to send signal {} to command", args.signal));
                    }
                }

                // Check if kill-after is specified
                if let Some(ref kill_after_str) = args.kill_after {
                    let kill_after_secs = parse_duration(kill_after_str)?;
                    if kill_after_secs > 0.0 {
                        if args.verbose {
                            eprintln!("Waiting {} seconds before sending KILL signal", kill_after_secs);
                        }

                        let kill_after_duration = Duration::from_secs_f64(kill_after_secs);
                        tokio::select! {
                            result = child.wait() => {
                                match result {
                                    Ok(status) => Ok((status, true)),
                                    Err(e) => Err(format!("Failed to wait for command: {}", e)),
                                }
                            },
                            _ = sleep(kill_after_duration) => {
                                // Send KILL signal
                                if args.verbose {
                                    eprintln!("Sending KILL signal to command");
                                }

                                if let Err(e) = child.kill().await {
                                    return Err(format!("Failed to kill command: {}", e));
                                }

                                // Wait for command to exit
                                match child.wait().await {
                                    Ok(status) => Ok((status, true)),
                                    Err(e) => Err(format!("Failed to wait for command after kill: {}", e)),
                                }
                            }
                        }
                    } else {
                        // Wait for command to exit after initial signal
                        match child.wait().await {
                            Ok(status) => Ok((status, true)),
                            Err(e) => Err(format!("Failed to wait for command after signal: {}", e)),
                        }
                    }
                } else {
                    // No kill-after, just wait for command to exit
                    match child.wait().await {
                        Ok(status) => Ok((status, true)),
                        Err(e) => Err(format!("Failed to wait for command after signal: {}", e)),
                    }
                }
            }
        }
    }
}

#[tokio::main]
async fn main() {
    let args = Args::parse();

    // Check if command and duration are provided
    if args.command.is_none() || args.duration.is_none() {
        eprintln!("Usage: timeout [OPTIONS] DURATION COMMAND [ARG]...");
        std::process::exit(125);
    }

    // Run command with timeout
    match run_with_timeout(&args).await {
        Ok((status, timed_out)) => {
            // Determine exit status based on whether command timed out and preserve-status option
            if timed_out && !args.preserve_status {
                // Command timed out and preserve-status is not set, return 124
                std::process::exit(124);
            } else {
                // Return command's exit status
                if status.success() {
                    std::process::exit(0);
                } else {
                    // Convert ExitStatus to exit code
                    let exit_code = match status.code() {
                        Some(code) => code,
                        None => 137, // Killed by signal
                    };
                    std::process::exit(exit_code);
                }
            }
        }
        Err(e) => {
            eprintln!("Error: {}", e);
            // Determine error exit code
            if e.contains("Failed to spawn command") {
                if e.contains("No such file or directory") {
                    std::process::exit(127);
                } else {
                    std::process::exit(126);
                }
            } else {
                std::process::exit(125);
            }
        }
    }
}
