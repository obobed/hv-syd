use satellite::AdminApiDoc;
use utoipa::OpenApi;

fn main() -> Result<(), Box<dyn core::error::Error>> {
    let doc = AdminApiDoc::openapi().to_pretty_json()?;

    println!("{}", doc);

    Ok(())
}
