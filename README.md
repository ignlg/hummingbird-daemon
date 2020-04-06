# hummingbird-daemon

[![Build Status](https://travis-ci.org/ignlg/hummingbird-daemon.svg?branch=master)](https://travis-ci.org/ignlg/hummingbird-daemon)

Launcher to daemonize AirVPN's Hummingbird OpenVPN client.

## Installation

### Download binary

Download release binary from [releases page](https://github.com/ignlg/hummingbird-daemon/releases).

### Build your own

Build with

```
cargo build --release
```

You will find your executable at `./target/release/hummingbird-daemon`.

## Usage

```
USAGE:
    hummingbird-daemon [FLAGS] [OPTIONS] <FILES>...

FLAGS:
    -h, --help
            Prints help information

    -V, --version
            Prints version information

    -v
            Verbose

OPTIONS:
        --wait-check <wait-check>
            Seconds to check again after network is reachable [default: 5]

        --wait-init <wait-init>
            Seconds to check network after executing a Hummingbird instance [default: 5]

ARGS:
    <FILES>...
            .ovpn files to pass to Hummingbird. Random on each execution if more than one
```

## Changelog

### v0.1.0

- [x] launch Hummingbird
- [x] detect if network is reachable
- [x] restart on network error or exit
- [x] accept multiple .ovpn files
- [x] choose a random .ovpn file each time
- [x] opt `--wait-check`
- [x] opt `--wait-init`

## Backlog

- [ ] check Hummingbird is available
- [ ] check Hummingbird version
- [ ] opt `--ping-hosts`
- [ ] opt `--hummingbird-args`
- [ ] check if interface is up
- [ ] detect unrecoverable Hummingbird error
- [ ] detect network unreachable due to Hummingbird dirty exit
- [ ] write logs to `/var/log`
- [ ] opt `--quiet-bird`

## License

hummingbird-daemon
Copyright (C) 2020 Ignacio Lago

This program is free software: you can redistribute it and/or modify
it under the terms of the GNU General Public License as published by
the Free Software Foundation, either version 3 of the License, or
(at your option) any later version.

This program is distributed in the hope that it will be useful,
but WITHOUT ANY WARRANTY; without even the implied warranty of
MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
GNU General Public License for more details.

You should have received a copy of the GNU General Public License
along with this program. If not, see <https://www.gnu.org/licenses/>.
