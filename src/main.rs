#[tokio::main]
async fn main() {
    pretty_env_logger::init();

    warp::serve(satellite::router())
        .run(([127, 0, 0, 1], 8008))
        .await
}
