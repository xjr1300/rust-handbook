use std::time::Duration;

use google_cloud_gax::{
    self as gax,
    options::RequestOptionsBuilder as _,
    retry_policy::{AlwaysRetry, RetryPolicyExt as _},
};
use google_cloud_secretmanager_v1::{self as sm, client::SecretManagerService, model::Secret};

const PROJECT_ID: &str = "gcp-for-rust";
const SECRET_ID: &str = "my-secret";

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let client = SecretManagerService::builder().build().await?;
    let data = b"Hello, World!".to_vec();
    let _ = update_secret(&client, PROJECT_ID, SECRET_ID, data).await?;

    Ok(())
}

async fn update_secret(
    client: &SecretManagerService,
    project_id: &str,
    secret_id: &str,
    data: Vec<u8>,
) -> gax::Result<sm::model::SecretVersion> {
    // シークレットの更新を試行
    match update_attempt(client, project_id, secret_id, data.clone()).await {
        Ok(version) => {
            println!("new version is {}", version.name);
            Ok(version)
        }
        Err(e) => {
            if let Some(status) = e.status() {
                use gax::error::rpc::Code;
                if status.code == Code::NotFound {
                    // シークレットが存在しない場合は、シークレットを作成して、再度更新を試行
                    let _ = create_secret(client, project_id, secret_id).await?;
                    let version = update_attempt(client, project_id, secret_id, data).await?;
                    println!("new version is {}", version.name);
                    return Ok(version);
                }
            }
            Err(e)
        }
    }
}

async fn create_secret(
    client: &SecretManagerService,
    project_id: &str,
    secret_id: &str,
) -> gax::Result<Secret> {
    client
        .create_secret()
        .set_parent(format!("projects/{project_id}"))
        .with_retry_policy(
            AlwaysRetry
                .with_attempt_limit(5)
                .with_time_limit(Duration::from_secs(15)),
        )
        .set_secret_id(secret_id)
        .set_secret(
            Secret::new()
                .set_replication(sm::model::Replication::new().set_replication(
                    sm::model::replication::Replication::Automatic(
                        sm::model::replication::Automatic::new().into(),
                    ),
                ))
                .set_labels([("integration-test", "true")]),
        )
        .send()
        .await
}

async fn update_attempt(
    client: &SecretManagerService,
    project_id: &str,
    secret_id: &str,
    data: Vec<u8>,
) -> gax::Result<sm::model::SecretVersion> {
    let checksum = crc32c::crc32c(&data) as i64;
    client
        .add_secret_version()
        .set_parent(format!("projects/{project_id}/secrets/{secret_id}"))
        .set_payload(
            sm::model::SecretPayload::new()
                .set_data(data)
                .set_data_crc32c(checksum),
        )
        .send()
        .await
}
