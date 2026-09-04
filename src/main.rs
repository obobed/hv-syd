use satellite::Config;
use tokio::io;

#[tokio::main]
async fn main() {
    pretty_env_logger::init();

    let config = load_config().await;
    let addr = config.general.address;

    warp::serve(satellite::router(config)).run(addr).await;
}

const CONFIG_PATH: &str = "config.toml";

async fn load_config() -> Config {
    match Config::load(CONFIG_PATH).await {
        Ok(c) => c,
        Err(e) if e.kind() == io::ErrorKind::NotFound => {
            Config::write_default(CONFIG_PATH)
                .await
                .expect("Config::write_default");
            Config::load(CONFIG_PATH).await.expect("Config::load")
        }
        Err(e) => {
            panic!("Config::load: {e:?}");
        }
    }
}
