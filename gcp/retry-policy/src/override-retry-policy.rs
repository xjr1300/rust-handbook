use std::time::Duration;

use google_cloud_gax::options::RequestOptionsBuilder;
use google_cloud_gax::paginator::ItemPaginator as _;
use google_cloud_gax::retry_policy::{Aip194Strict, AlwaysRetry, RetryPolicyExt};
use google_cloud_secretmanager_v1 as secret_manager;

const PROJECT_ID: &str = "gcp-for-rust";
const SECRET_ID: &str = "your-secret";

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let client = secret_manager::client::SecretManagerService::builder()
        .with_retry_policy(Aip194Strict)
        .build()
        .await?;

    client
        .delete_secret()
        .set_name(format!("projects/{PROJECT_ID}/secrets/{SECRET_ID}"))
        .with_retry_policy(
            AlwaysRetry
                .with_attempt_limit(5)
                .with_time_limit(Duration::from_secs(15)),
        )
        .send()
        .await?;

    let mut list = client
        .list_secrets()
        .set_parent(format!("projects/{PROJECT_ID}"))
        .by_item();
    while let Some(secret) = list.next().await {
        let secret = secret?;
        println!("secret={}", secret.name);
    }

    Ok(())
}
