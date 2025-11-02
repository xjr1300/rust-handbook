use google_cloud_gax::error::rpc::StatusDetails;
use google_cloud_language_v2 as lang;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let client = lang::client::LanguageService::builder().build().await?;

    let result = client
        .analyze_sentiment()
        .set_document(
            lang::model::Document::new()
                // ドキュメントのコンテンツを欠落させる
                // .set_content("Hello World!")
                .set_type(lang::model::document::Type::PlainText),
        )
        .send()
        .await;

    match result {
        Ok(response) => {
            println!("{response:#?}")
        }
        Err(err) => {
            handle_error(err);
        }
    }

    Ok(())
}

fn handle_error(err: lang::Error) {
    println!("\nrequest failed with error {err:#?}");

    if let Some(status) = err.status() {
        println!(
            "  status.code={}, status.message={}",
            status.code, status.message,
        );
        for detail in status.details.iter() {
            match detail {
                StatusDetails::BadRequest(bad) => {
                    for f in bad.field_violations.iter() {
                        println!(
                            "  the request field {} has a problem: \"{}\"",
                            f.field, f.description
                        );
                    }
                }
                _ => {
                    println!("  additional error details: {detail:?}");
                }
            }
        }
    }
}
