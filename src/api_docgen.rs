use satellite::OpenApiDoc;
use utoipa::OpenApi;

const OUTPUT_PATH: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/openapi.json");

fn main() -> Result<(), Box<dyn core::error::Error>> {
    let doc = OpenApiDoc::openapi().to_pretty_json()?;

    std::fs::write(OUTPUT_PATH, &doc)?;

    Ok(())
}
