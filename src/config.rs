use {serde::Deserialize, std::path::PathBuf, structopt::StructOpt, url::Url};

#[derive(Debug, StructOpt)]
pub struct Opt {
    #[structopt(short, long)]
    pub debug: bool,
    #[structopt(parse(from_os_str))]
    pub config: PathBuf,
}

#[derive(Debug, Deserialize)]
pub struct AppConfig {
    pub debug: bool,
    pub redis: RedisConfig,
}

#[derive(Debug, Deserialize)]
pub struct RedisConfig {
    pub url: Url,
}
