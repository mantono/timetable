use clap::Parser;

use crate::logger::Verbosity;

#[derive(Parser, Debug)]
#[clap(author, version, about)]
pub struct Config {
    #[clap(short, long, env = "DB_URL")]
    database: String,

    /// Set verbosity level, 0 - 5
    ///
    /// Set the verbosity level, from 0 (least amount of output) to 5 (most verbose). Note that
    /// logging level configured via RUST_LOG overrides this setting.
    #[structopt(short, long = "verbosity", default_value = "1")]
    verbosity_level: u8,
}

impl Config {
    pub fn db_url(&self) -> &str {
        &self.database
    }

    pub fn verbosity(&self) -> Verbosity {
        self.verbosity_level
            .try_into()
            .expect("Invalid vervosity level")
    }
}
