use env_logger::fmt::{Color, Formatter};
use log::{Level, LevelFilter, Record};
use std::io;
use std::io::Write;
use std::str::FromStr;

#[derive(Debug)]
pub struct Verbosity(u8);

impl Verbosity {
    pub const OFF: Verbosity = Verbosity(0);
    pub const ERROR: Verbosity = Verbosity(1);
    pub const WARN: Verbosity = Verbosity(2);
    pub const INFO: Verbosity = Verbosity(3);
    pub const DEBUG: Verbosity = Verbosity(4);
    pub const TRACE: Verbosity = Verbosity(5);

    pub fn level(&self) -> u8 {
        self.0
    }
}

impl FromStr for Verbosity {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.parse::<u8>() {
            Ok(n) => match n {
                0..=5 => Ok(Verbosity(n)),
                _ => Err(format!("Unsupported verbosity level '{}'", n)),
            },
            Err(e) => Err(e.to_string()),
        }
    }
}

impl TryFrom<u8> for Verbosity {
    type Error = String;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0..=5 => Ok(Verbosity(value)),
            _ => Err(format!("Unsupported verbosity level '{}'", value)),
        }
    }
}

pub fn setup_logging(verb: &Verbosity) {
    match std::env::var("RUST_LOG") {
        Ok(_) => log_by_env_var(),
        Err(_) => log_by_cmd_arg(verb),
    }
}

fn log_by_env_var() {
    env_logger::Builder::from_default_env()
        .format(formatter)
        .init()
}

fn log_by_cmd_arg(verb: &Verbosity) {
    let filter: LevelFilter = match verb.level() {
        0 => LevelFilter::Off,
        1 => LevelFilter::Error,
        2 => LevelFilter::Warn,
        3 => LevelFilter::Info,
        4 => LevelFilter::Debug,
        5 => LevelFilter::Trace,
        _ => panic!("Invalid verbosity level: {}", verb.level()),
    };

    env_logger::builder()
        .format(formatter)
        .filter_level(filter)
        .init()
}

fn formatter(buf: &mut Formatter, record: &Record) -> io::Result<()> {
    match record.level() {
        Level::Info => writeln!(buf, "{}", record.args()),
        Level::Warn => {
            let mut style = buf.style();
            style.set_color(Color::Yellow);
            writeln!(buf, "{}: {}", style.value(record.level()), record.args())
        }
        Level::Error => {
            let mut style = buf.style();
            style.set_color(Color::Red);
            writeln!(buf, "{}: {}", style.value(record.level()), record.args())
        }
        _ => writeln!(buf, "{}: {}", record.level(), record.args()),
    }
}
