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

#[derive(Debug, PartialEq, Eq)]
pub enum LogLevel {
  ERROR = 0,
  WARN = 1,
  INFO = 2,
  DEBUG = 3,
}

/// Filter and write output to stdout/stderr
pub fn logger(opt_verbose: u32) -> impl Fn(LogLevel, String) -> () {
  move |level: LogLevel, message: String| match (level, opt_verbose) {
    (LogLevel::ERROR, _) => eprintln!("Daemon >>> ERROR: {}", message),
    (LogLevel::WARN, v) if v > 0 => println!("Daemon >>> WARN:  {}", message),
    (LogLevel::INFO, v) if v > 1 => println!("Daemon >>> INFO:  {}", message),
    (LogLevel::DEBUG, v) if v > 2 => println!("Daemon >>> DEBUG: {}", message),
    (_, _) => (),
  }
}
