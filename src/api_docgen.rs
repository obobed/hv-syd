use satellite::AdminApiDoc;
use utoipa::OpenApi;

const ADMIN_DOC_OUTPUT_PATH: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/admin-openapi.json");

fn main() -> Result<(), Box<dyn core::error::Error>> {
    let doc = AdminApiDoc::openapi().to_pretty_json()?;

    std::fs::write(ADMIN_DOC_OUTPUT_PATH, &doc)?;

    Ok(())
}
