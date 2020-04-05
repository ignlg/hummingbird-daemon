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

use std::path;
use structopt::StructOpt;

mod daemon;
use daemon::Daemon;

mod logger;
use logger::{logger, LogLevel};

use rand::{thread_rng, Rng};

/// Time between loops
const CHECK_MS: usize = 100;

// hummingbird-daemon
/// Copyright (C) 2020  Ignacio Lago
///
/// This program comes with ABSOLUTELY NO WARRANTY.
/// This is free software, and you are welcome to redistribute it under certain conditions.

#[derive(StructOpt, Debug)]
#[structopt(name = "Hummingbird Daemon")]
struct Opt {
    /// Verbose
    #[structopt(short, parse(from_occurrences))]
    verbose: u32,
    /// Seconds to check network after executing a Hummingbird instance
    #[structopt(long, default_value = "5")]
    wait_init: usize,
    /// Seconds to check again after network is reachable
    #[structopt(long, default_value = "5")]
    wait_check: usize,
    // Options to pass to Hummingbird
    // #[structopt(long)]
    // hummingbird_options: String,
    /// .ovpn files to pass to Hummingbird. Random on each execution if more than one.
    #[structopt(required = true, name = "FILES", parse(from_os_str))]
    files: Vec<path::PathBuf>,
}

fn main() {
    // opts
    let opt = Opt::from_args();
    // logger
    let logger = logger(opt.verbose);
    // daemon
    let mut daemon = Daemon::new();
    // rng
    let mut rng = thread_rng();
    // wait options to millis
    let wait_init_ms = opt.wait_init * 1000;
    let wait_check_ms = opt.wait_check * 1000;
    // flags and counters
    let mut is_init = false;
    let mut time_since_last_check: usize = 0;
    loop {
        let should_spawn;
        match daemon.is_alive() {
            Ok(is_alive) => {
                if is_alive {
                    // Do nothing if too early after init or after check
                    if (is_init && time_since_last_check < wait_init_ms)
                        || (!is_init && time_since_last_check < wait_check_ms)
                    {
                        should_spawn = false;
                    } else {
                        is_init = false;
                        if let Ok(host) = daemon.is_network_reachable() {
                            // Network is working
                            logger(LogLevel::DEBUG, format!("Network reachable -> {}", host));
                            should_spawn = false;
                        } else {
                            // Network unreachable
                            logger(LogLevel::WARN, String::from("Network unreachable"));
                            should_spawn = true;
                        }
                        time_since_last_check = 0;
                    }
                } else {
                    logger(LogLevel::WARN, String::from("Hummingbird is dead"));
                    // TODO: Check if unrecoverable error
                    should_spawn = true;
                }
            }
            Err(err) => {
                logger(LogLevel::ERROR, format!("is_alive -> {}", err));
                should_spawn = true;
            }
        }

        // Check process launch
        if should_spawn {
            // Reset time to check
            time_since_last_check = 0;
            logger(LogLevel::DEBUG, String::from("Should spawn a Hummingbird"));
            // If previous child, SIGINT
            if let Some(pid) = daemon.get_pid() {
                daemon.interrupt();
                logger(LogLevel::INFO, format!("Sent SIGINT to pid {}", pid));
            } else {
                let file = if opt.files.len() == 1 {
                    opt.files[0].to_str()
                } else {
                    let idx = rng.gen_range(0, opt.files.len() - 1);
                    opt.files[idx].to_str()
                };
                if let Some(file) = file {
                    daemon.execute(file);
                    if let Some(pid) = daemon.get_pid() {
                        logger(
                            LogLevel::INFO,
                            format!("Hummingbird starting with pid {}", pid),
                        );
                        is_init = true;
                    } else {
                        logger(LogLevel::ERROR, String::from("Hummingbird is dead on boot"));
                    }
                } else {
                    logger(LogLevel::ERROR, String::from("VPN configuration failed"));
                    return;
                }
            }
        } else if !is_init && time_since_last_check >= wait_check_ms {
            // Time to check network again
            time_since_last_check = 0;
        }
        std::thread::sleep(std::time::Duration::from_millis(CHECK_MS as u64));
        // Add time to timer
        time_since_last_check += CHECK_MS;
    }
}
