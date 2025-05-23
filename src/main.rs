use clap::ArgMatches;
use std::fmt::{self, Debug, Display, Formatter};
use std::io::{BufRead, BufReader, Write};
use std::os::unix::net::UnixStream;
use std::str::FromStr;
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use std::{env, u64};

use clap::{App, Arg};
use discord_rpc_client::models::Activity;
use discord_rpc_client::Client;

use env_logger;
use log::{debug, info, warn};
use regex::Regex;

/*
  Here can the configuration be edited,
  if changes are made you need to recompile and
  then restart cmus-discord-rpc with: `cargo install --path .`
  and rerun cmus-discord-rpc
*/

const DEFAULT_MAIN_THREAD_WAIT: u64 = 5000; /* NEEDED: Dont touch, use -m flag with arguments instead or default */
const DEFAULT_UNIX_THREAD_WAIT: u64 = 15000; /* NEEDED:  Dont touch, use -u flag with arguments instead or default */
const ARTIST_SONG_SEPERATOR: &str = "|"; /* NEEDED: Seperator used for Artist and Song */
const APPLICATION_ID: u64 = 1212098714341089433; /* NEEDED: Application ID, don't change if you don't need any custom images */
const IMAGE_NAME_LARGE: &str = "ignorance"; /* OPTIONAL: Image name uploaded to the application id used for displaying the large image */
const IMAGE_NAME_SMALL: &str = "none"; /* OPTIONAL: Image name uploaded to the application id used for displaying the small image */
const IMAGE_TEXT_LARGE: &str = "// to be ignorant is to be free //"; /* OPTIONAL: Tooltip for the big image if it exists */
const IMAGE_TEXT_SMALL: &str = "wishes"; /* OPTIONAL: Tooltip for the small image if it exists */
const EXTRA_1: &str = " "; /* OPTIONAL: Extra string 1, will be appended after the song name */
const EXTRA_2: &str = " "; /* OPTIONAL: Extra string 2, will be appended after EXTRA_1 */
const EXTRA_3: &str = " "; /* OPTIONAL: Extra string 3, will be appended after EXTRA_2 */

/* End of configs */

#[derive(PartialEq, Debug)]
enum Status {
    Playing,
    Paused,
    Stopped,
}

impl Display for Status {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        Debug::fmt(self, f)
    }
}

#[derive(Debug)]
struct ParseStatusError;

impl FromStr for Status {
    type Err = ParseStatusError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "playing" => Ok(Status::Playing),
            "paused" => Ok(Status::Paused),
            "stopped" => Ok(Status::Stopped),
            _ => Err(ParseStatusError),
        }
    }
}

fn cli() -> ArgMatches {
    App::new("cmus-discord-rpc")
        .arg(
            Arg::with_name("main_thread_wait")
                .short('m')
                .long("main-thread-wait")
                .value_name("SECONDS")
                .help("Sets the refresh rate for the main thread in milliseconds")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("unix_thread_wait")
                .short('u')
                .long("unix-thread-wait")
                .value_name("SECONDS")
                .help("Sets the wait time for getting the Unix stream in milliseconds")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("verbose")
                .short('v')
                .long("verbose")
                .help("Enable verbose output for debugging"),
        )
        .get_matches()
}

fn main() {
    // get cli arguments
    let matches = cli();

    // If verbose is enabled set log level
    let log_level = if matches.is_present("verbose") {
        log::LevelFilter::Debug
    } else {
        log::LevelFilter::Info
    };

    env_logger::Builder::from_env(
        env_logger::Env::default().default_filter_or(log_level.to_string()),
    )
    .init();

    // Parse arguments or use default values
    let main_thread_wait = matches
        .value_of("main_thread_wait")
        .and_then(|s| s.parse().ok())
        .unwrap_or(DEFAULT_MAIN_THREAD_WAIT);

    let unix_thread_wait = matches
        .value_of("unix_thread_wait")
        .and_then(|s| s.parse().ok())
        .unwrap_or(DEFAULT_UNIX_THREAD_WAIT);

    match main_thread_wait {
        DEFAULT_MAIN_THREAD_WAIT => warn!(
            "Using default refresh rate: {} milliseconds",
            main_thread_wait
        ),
        _ => {
            info!(
                "Using custom refresh rate: {} milliseconds",
                main_thread_wait
            );
            if main_thread_wait < 3000 {
                warn!("Refresh rates under 3000 milliseconds may desync the time left randomly!");
            }
        }
    }

    match unix_thread_wait {
        DEFAULT_UNIX_THREAD_WAIT => warn!(
            "Using default Unix stream wait: {} milliseconds",
            unix_thread_wait,
        ),
        _ => {
            info!(
                "Using custom Unix stream wait: {} milliseconds",
                unix_thread_wait
            );
        }
    }

    debug!("Starting cmus-discord-rpc...");

    let socket_path = get_socket_path();
    debug!("Using cmus socket {}", socket_path);
    let mut stream = get_unix_stream(&socket_path, unix_thread_wait);
    let mut drpc = Client::new(APPLICATION_ID);
    drpc.start();

    let mut output = String::new();
    let mut counter: u64 = 0;

    loop {
        counter = counter + 1;
        if stream.write_all(b"status\n").is_err() {
            drpc.clear_activity().expect("Failed to clear presence");
            stream = get_unix_stream(&socket_path, unix_thread_wait);
            continue;
        }

        let mut reader = BufReader::new(&stream);
        output.clear();

        // Read until an empty line
        while reader.read_line(&mut output).unwrap() != 1 {}
        debug!("Received\n{}", output);

        let status = get_value(&output, "status")
            .unwrap()
            .parse::<Status>()
            .unwrap();

        let mut ac = Activity::new().details(status.to_string());
        if status != Status::Stopped {
            let artist = get_value(&output, "tag artist");
            let title = get_value(&output, "tag title");

            if artist.is_none() || title.is_none() {
                // Capture filename
                let file_r = Regex::new(r"(?m)^file .+/(.+)\..+\n").unwrap();
                match file_r.captures(&output) {
                    Some(v) => {
                        ac = ac.state(
                            v.get(1).unwrap().as_str().to_owned() + EXTRA_1 + EXTRA_2 + EXTRA_3,
                        )
                    }
                    None => ac = ac.state(""),
                }
            } else {
                ac = ac.state(
                    artist.unwrap().to_owned()
                        + " "
                        + ARTIST_SONG_SEPERATOR
                        + " "
                        + title.unwrap()
                        + EXTRA_1
                        + EXTRA_2
                        + EXTRA_3,
                )
            }

            // Add configs to all types of outcomes
            ac = ac.assets(|assets| {
                { assets }
                    .large_image(IMAGE_NAME_LARGE)
                    .small_image(IMAGE_NAME_SMALL)
                    .small_text(IMAGE_TEXT_SMALL)
                    .large_text(IMAGE_TEXT_LARGE)
            });

            if status == Status::Playing {
                let duration = get_value(&output, "duration")
                    .unwrap()
                    .parse::<u64>()
                    .unwrap();
                let position = get_value(&output, "position")
                    .unwrap()
                    .parse::<u64>()
                    .unwrap();
                let sce = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_secs();
                ac = ac.timestamps(|t| t.end(sce + duration - position));
            }
        }

        drpc.set_activity(|_| ac)
            .unwrap_or_else(|_| panic!("Failed to set Presence"));

        info!("Successfully set discord-rpc x{}", counter);

        // PATCHED: use arguments from clap
        debug!(
            "Sleeping for length of main_thread_wait: {}",
            main_thread_wait
        );
        thread::sleep(Duration::from_millis(main_thread_wait));
    }
}

fn get_unix_stream(socket_path: &str, unix_thread_wait: u64) -> UnixStream {
    loop {
        if let Ok(s) = UnixStream::connect(socket_path) {
            return s;
        }

        // PATCHED: use arguments from clap
        debug!(
            "Sleeping for length of unix_thread_wait: {}",
            unix_thread_wait
        );
        thread::sleep(Duration::from_millis(unix_thread_wait));
    }
}

/// Get the path to the cmus socket the same way as cmus itself
fn get_socket_path() -> String {
    if let Ok(v) = env::var("CMUS_SOCKET") {
        return v;
    }

    if let Ok(v) = env::var("XDG_RUNTIME_DIR") {
        return v + "/cmus-socket";
    }

    let cmus_config_dir = match env::var("XDG_CONFIG_HOME") {
        Ok(v) => v,
        Err(_) => env::var("HOME").unwrap() + "/.config",
    } + "/cmus";

    cmus_config_dir + "/socket"
}
fn get_value<'t>(input: &'t str, key: &str) -> Option<&'t str> {
    let re = Regex::new(&format!("(?m)^{} (.+)$", key)).unwrap();

    Some(re.captures(input)?.get(1)?.as_str())
}
