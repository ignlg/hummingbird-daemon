/*
    Launcher to daemonize AirVPN's Hummingbird OpenVPN client
    Copyright (C) 2020  Ignacio Lago

    This program is free software: you can redistribute it and/or modify
    it under the terms of the GNU General Public License as published by
    the Free Software Foundation, either version 3 of the License, or
    (at your option) any later version.

    This program is distributed in the hope that it will be useful,
    but WITHOUT ANY WARRANTY; without even the implied warranty of
    MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
    GNU General Public License for more details.

    You should have received a copy of the GNU General Public License
    along with this program.  If not, see <https://www.gnu.org/licenses/>.
*/

use pipeliner::Pipeline;
use rand::{seq::SliceRandom, thread_rng};
use std::fmt;
use std::io::{self, Read};
use std::process;

const DEFAULT_HOSTS: [&str; 4] = ["8.8.8.8", "8.8.6.6", "1.1.1.1", "1.0.0.1"];

// > /var/log/hummingbird.log 2> /var/log/hummingbird_error.log

/// Ping a host and return if it is reachable
fn ping(host: &str) -> bool {
  process::Command::new("/bin/sh")
    .arg("-c")
    .arg(&format!("ping -c1 {}", host))
    .output()
    .expect("No shell?")
    .status
    .success()
}

/// Daemon errors
#[derive(Debug, PartialEq, Eq)]
pub enum DaemonError {
  NoStatus,
  ChildFound,
  CannotRecoverNetwork,
  NetworkUnreachable,
  NetworkSendError,
  NetworkConnectError,
}
impl fmt::Display for DaemonError {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "{}", self)
  }
}

/// Daemon
#[derive(Debug)]
pub struct Daemon {
  child: Option<process::Child>,
  error: Option<io::Error>,
  hosts_alive: Vec<&'static str>,
}

// Public impl
impl Daemon {
  /// Create a new Daemon instance
  pub fn new() -> Self {
    Self {
      child: None,
      error: None,
      hosts_alive: DEFAULT_HOSTS.to_vec(),
    }
  }
  /// Spawn a hummingbird child process
  pub fn execute(&mut self, config: &str) {
    match process::Command::new("hummingbird").arg(config).spawn() {
      Ok(child) => {
        self.child = Some(child);
        self.error = None;
      }
      Err(error) => {
        self.error = Some(error);
        self.child = None;
      }
    };
  }
  /// Recover network
  pub fn recover_network(&self) -> Result<bool, DaemonError> {
    if self.child.is_none() {
      let output = process::Command::new("hummingbird")
        .arg("--recover-network")
        .output();
      return match output {
        Ok(output) => Ok(output.status.success()),
        Err(_) => Err(DaemonError::CannotRecoverNetwork),
      };
    }
    Err(DaemonError::ChildFound)
  }
  /// Kill child process, if any
  pub fn interrupt(&mut self) {
    if let Some(child) = &mut self.child {
      nix::sys::signal::kill(
        nix::unistd::Pid::from_raw(child.id() as i32),
        nix::sys::signal::Signal::SIGINT,
      )
      .expect("cannot send SIGINT");
      child.wait().ok();
      self.child = None;
    }
  }
  /// Kill child process, if any
  pub fn kill(&mut self) {
    if let Some(child) = &mut self.child {
      if child.kill().is_ok() {
        child.wait().ok();
        self.child = None;
      }
    }
  }
  /// Is child process alive?
  pub fn is_alive(&mut self) -> Result<bool, DaemonError> {
    if let Some(child) = &mut self.child {
      let status = child.try_wait();
      return match status {
        Ok(None) => Ok(true),
        Err(_) => Err(DaemonError::NoStatus),
        Ok(_) => Ok(false),
      };
    }
    Ok(false)
  }
  /// Get child process PID, if any
  pub fn get_pid(&mut self) -> Option<u32> {
    if let Ok(is_alive) = self.is_alive() {
      if is_alive {
        if let Some(child) = &mut self.child {
          return Some(child.id());
        }
      }
    }
    None
  }
  /// Is Hummingbird running with enough permissions?
  pub fn is_root_user(&mut self) -> bool {
    !self.read_stderr().contains("need to be root")
  }
  /// Check if network is reachable
  pub fn is_network_reachable(&self) -> Result<&str, DaemonError> {
    let mut hosts = self.hosts_alive.to_vec();
    let mut rng = thread_rng();
    hosts.shuffle(&mut rng);
    if ping(hosts[0]) {
      return Ok(hosts[0]);
    }
    let n = self.hosts_alive.len();
    for result in hosts.with_threads(n).map(|s| (s, ping(s))) {
      match result {
        (host, true) => return Ok(host),
        (_, _) => {}
      }
    }
    Err(DaemonError::NetworkUnreachable)
  }
  /// Check if Hummingbird complains about network error
  pub fn has_network_error(&mut self) -> Result<(), DaemonError> {
    let stdout = self.read_stdout();
    if stdout.contains("NETWORK_SEND_ERROR") {
      Err(DaemonError::NetworkSendError)
    } else if stdout.contains("CONNECT ERROR") {
      Err(DaemonError::NetworkConnectError)
    } else {
      Ok(())
    }
  }
  /// Needs to recover
  pub fn has_recover_error(&mut self) -> bool {
    let errors = ["not exit gracefully"];
    let stdout = self.read_stdout();
    for error in errors.iter() {
      if stdout.contains(error) {
        return true;
      }
    }
    false
  }
  /// Set alternative hosts
  pub fn set_hosts_alive(&mut self, hosts: Vec<&'static str>) {
    self.hosts_alive = hosts
  }
}

// Private impl
impl Daemon {
  /// Read stdout, if any
  fn read_stdout(&mut self) -> String {
    if let Some(child) = &mut self.child {
      if let Some(stdout) = &mut child.stdout {
        let mut output = String::new();
        if let Ok(_n) = stdout.read_to_string(&mut output) {
          return output;
        }
      }
    }
    String::new()
  }
  /// Read stderr, if any
  fn read_stderr(&mut self) -> String {
    if let Some(child) = &mut self.child {
      if let Some(stderr) = &mut child.stderr {
        let mut output = String::new();
        if let Ok(_n) = stderr.read_to_string(&mut output) {
          return output;
        }
      }
    }
    String::new()
  }
}
