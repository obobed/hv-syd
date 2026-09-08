#![recursion_limit = "256"]
use std::{io, process::ExitCode};

use satellite::{Config, Error};

async fn _main() -> Result<(), Error> {
    pretty_env_logger::init();

    let config = load_config().await?;
    let addr = config.general.address;

    Ok(warp::serve(satellite::router(config)?).run(addr).await)
}

const CONFIG_PATH: &str = "config.toml";

async fn load_config() -> Result<Config, Error> {
    match Config::load(CONFIG_PATH).await {
        Ok(c) => Ok(c),
        Err(e) if e.kind() == io::ErrorKind::NotFound => {
            Config::write_default(CONFIG_PATH).await?;
            Err(Error::Custom(
                413.try_into().unwrap(),
                "generated `config.toml`, please edit.".into(),
            ))
        }
        Err(e) => Err(Error::Custom(
            413.try_into().unwrap(),
            format!("loading config failed: {}", e),
        )),
    }
}

#[tokio::main]
async fn main() -> ExitCode {
    match _main().await {
        Ok(_) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("{}", e);
            ExitCode::FAILURE
        }
    }
}
