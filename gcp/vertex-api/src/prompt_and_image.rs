//! ```sh
//! cargo run --package=vertex-api --bin=prompt-and-image -- <project-id>
//! ```
use google_cloud_aiplatform_v1 as vertex_ai;

const MODEL: &str = "gemini-2.0-flash-001";
const PROMPT: &str = "Describe this picture.";
const FILE_URI: &str = "gs://generativeai-downloads/images/scones.jpg";

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let project_id = std::env::args().nth(1).unwrap();
    let client = vertex_ai::client::PredictionService::builder()
        .build()
        .await?;

    let model = format!("projects/{project_id}/locations/global/publishers/google/models/{MODEL}");

    let response = client
        .generate_content()
        .set_model(&model)
        .set_contents([vertex_ai::model::Content::new()
            .set_role("user")
            .set_parts([
                vertex_ai::model::Part::new().set_file_data(
                    vertex_ai::model::FileData::new()
                        .set_mime_type("image/jpeg")
                        .set_file_uri(FILE_URI),
                ),
                vertex_ai::model::Part::new().set_text(PROMPT),
            ])])
        .send()
        .await;
    println!("RESPONSE = {response:#?}");

    Ok(())
}
