# cmus-discord-rpc

Fork of [Bond-009/cmus-discord-rpc](https://github.com/Bond-009/cmus-discord-rpc) including patches that are nice

Discord Rich Presence integration for the C* Music Player (`cmus`).

## Usage

- Help message
```
cmus-discord-rpc 

USAGE:
    cmus-discord-rpc [OPTIONS]

OPTIONS:
    -h, --help                          Print help information
    -m, --main-thread-wait <SECONDS>    Sets the refresh rate for the main thread in milliseconds
    -u, --unix-thread-wait <SECONDS>    Sets the wait time for getting the Unix stream in
                                        milliseconds
```

- Start rich presence refreshing songs every 2 seconds and getting the unix stream every 5 seconds
``` 
cmus-discord-rpc -m 2000 -u 5000
```

## Installing

- If it isn't already on your system, install `rust`, and `cargo`. You should do this through `rustup` by installing it with your package manager or from [rustup.rs](https://rustup.rs).

- Obtain the sources. You can either do this by cloning the repository using `git` or downloading an archive of the repository.

  Cloning using HTTPS:

      git clone https://github.com/POP303U/cmus-discord-rpc

  Cloning using `ssh`:

      git clone git@github.com:POP303U/cmus-discord-rpc.git

  Downloading an archive using `wget`:

       wget https://github.com/POP303U/cmus-discord-rpc/archive/master.zip

       unzip master.zip

- Change your directory into where the sources were cloned/extracted to.

      cd cmus-discord-rpc

- Next, build and install it to your home directory.

      cargo install --path .

- Once `cargo`'s installation directory is in your `PATH` (`cargo` should tell you where the end of the previous step) simply run `cmus-discord-rpc` and it should start!

## Building

- Obtain the sources. You can either do this by cloning the repository or downloading an archive of the repository.

- Change your directory into where the sources were cloned/extracted to.

- Finally to build, use the following commands:

  For debugging:

      cargo build

  For production use:

      cargo build --release

- You should see a new directory called `target`. There you can find subfolders for each of your build targets.

## License

This program is free software: you can redistribute it and/or modify
it under the terms of the GNU General Public License as published by
the Free Software Foundation, either version 3 of the License, or
(at your option) any later version.

This program is distributed in the hope that it will be useful,
but WITHOUT ANY WARRANTY; without even the implied warranty of
MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
GNU General Public License for more details.

You should have received a copy of the GNU General Public License
along with this program.  If not, see https://www.gnu.org/licenses/.
