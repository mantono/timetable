use clap::Parser;

#[derive(Parser, Debug)]
#[clap(author, version, about)]
pub struct Config {
    #[clap(short, long, env = "DB_URL")]
    database: String,
}

impl Config {
    pub fn db_url(&self) -> &str {
        &self.database
    }
}
