mod api;
mod app;
mod config;
mod error;
mod redis;
mod sse;

use {
    crate::{
        app::App,
        config::{AppConfig, Opt},
    },
    structopt::StructOpt,
};

#[tokio::main(core_threads = 8)]
async fn main() -> anyhow::Result<()> {
    simple_logger::init_with_level(log::Level::Info)?;

    let mut base_config = configer::Config::default();
    let opt = Opt::from_args();
    base_config
        .set_default("debug", opt.debug)?
        .merge(configer::File::from(opt.config))?;
    let config: AppConfig = base_config.try_into()?;
    let app = App::from_config(config).await?;

    warp::serve(app.into_filter())
        .run(([0, 0, 0, 0], 8000))
        .await;

    Ok(())
}
