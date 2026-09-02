use satellite::{Config, GUIDES_FOLDER, PRIVATE_ASSET_FOLDER, PUBLIC_ASSET_FOLDER};
use tokio::{fs, io};

#[tokio::main]
async fn main() {
    pretty_env_logger::init();

    let config = load_config().await;
    let addr = config.general.address;

    initialise_folders().await;

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
            Config::load(CONFIG_PATH).await.unwrap()
        }
        Err(e) => {
            panic!("Config::load: {e:?}");
        }
    }
}

async fn initialise_folders() {
    fs::create_dir_all(PUBLIC_ASSET_FOLDER)
        .await
        .expect("fs::create_dir_all");
    fs::create_dir_all(PRIVATE_ASSET_FOLDER)
        .await
        .expect("fs::create_dir_all");
    fs::create_dir_all(GUIDES_FOLDER)
        .await
        .expect("fs::create_dir_all");
}
