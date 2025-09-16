use conventional_release_action::{create_release_application, output::output_results};
use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let app = create_release_application().await?;
    let result = app.run().await?;
    output_results(result)?;
    Ok(())
}
