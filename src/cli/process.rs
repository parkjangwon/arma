use std::fmt;
use std::fs;
use std::path::{Path, PathBuf};

use daemonize::Daemonize;
use nix::sys::signal::{kill, Signal};
use nix::unistd::Pid;

/// Process lifecycle command errors.
#[derive(Debug)]
pub enum ProcessError {
    Io(std::io::Error),
    Nix(nix::Error),
    ParsePid(std::num::ParseIntError),
    Daemonize(daemonize::Error),
}

impl fmt::Display for ProcessError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(err) => write!(f, "I/O error: {err}"),
            Self::Nix(err) => write!(f, "signal error: {err}"),
            Self::ParsePid(err) => write!(f, "pid parse error: {err}"),
            Self::Daemonize(err) => write!(f, "daemonization error: {err}"),
        }
    }
}

impl std::error::Error for ProcessError {}

impl From<std::io::Error> for ProcessError {
    fn from(value: std::io::Error) -> Self {
        Self::Io(value)
    }
}

impl From<nix::Error> for ProcessError {
    fn from(value: nix::Error) -> Self {
        Self::Nix(value)
    }
}

impl From<std::num::ParseIntError> for ProcessError {
    fn from(value: std::num::ParseIntError) -> Self {
        Self::ParsePid(value)
    }
}

impl From<daemonize::Error> for ProcessError {
    fn from(value: daemonize::Error) -> Self {
        Self::Daemonize(value)
    }
}

/// Returns canonical PID file path.
pub fn pid_file_path() -> Result<PathBuf, ProcessError> {
    Ok(std::env::current_dir()?.join(".arma.pid"))
}

/// Daemonizes process when requested and writes current PID.
pub fn prepare_start(daemon: bool) -> Result<(), ProcessError> {
    if daemon {
        let working_dir = std::env::current_dir()?;
        Daemonize::new().working_directory(working_dir).start()?;
    }

    write_current_pid_file()
}

/// Stops running ARMA process using SIGTERM.
pub fn stop_process() -> Result<(), ProcessError> {
    let pid_path = pid_file_path()?;
    let pid = read_pid_file(&pid_path)?;
    match kill(Pid::from_raw(pid), Signal::SIGTERM) {
        Ok(()) => {}
        Err(error) => {
            if error != nix::errno::Errno::ESRCH {
                return Err(ProcessError::Nix(error));
            }
        }
    }
    remove_pid_file_if_exists(&pid_path)?;
    Ok(())
}

/// Sends SIGHUP to running ARMA process.
pub fn reload_process() -> Result<(), ProcessError> {
    let pid = read_pid_file(&pid_file_path()?)?;
    kill(Pid::from_raw(pid), Signal::SIGHUP)?;
    Ok(())
}

/// Removes PID file if present.
pub fn clear_pid_file() -> Result<(), ProcessError> {
    let pid_path = pid_file_path()?;
    remove_pid_file_if_exists(&pid_path)
}

/// Returns true when PID file exists and target process is alive.
pub fn is_active() -> bool {
    let pid_path = match pid_file_path() {
        Ok(value) => value,
        Err(_) => return false,
    };

    let pid = match read_pid_file(&pid_path) {
        Ok(value) => value,
        Err(_) => return false,
    };

    matches!(kill(Pid::from_raw(pid), None), Ok(()))
}

fn write_current_pid_file() -> Result<(), ProcessError> {
    let pid = std::process::id();
    let content = pid.to_string();
    fs::write(pid_file_path()?, content)?;
    Ok(())
}

fn read_pid_file(path: &Path) -> Result<i32, ProcessError> {
    let raw = fs::read_to_string(path)?;
    let pid = raw.trim().parse::<i32>()?;
    Ok(pid)
}

fn remove_pid_file_if_exists(path: &Path) -> Result<(), ProcessError> {
    if path.exists() {
        fs::remove_file(path)?;
    }
    Ok(())
}
