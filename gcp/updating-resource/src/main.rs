use std::collections::HashMap;

use google_cloud_secretmanager_v1::{
    client::SecretManagerService,
    model::{Replication, Secret, replication::Automatic},
};
use google_cloud_wkt::FieldMask;

const PROJECT_ID: &str = "gcp-for-rust";

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let client = SecretManagerService::builder().build().await?;

    let secret = client
        .create_secret()
        .set_parent(format!("projects/{PROJECT_ID}"))
        .set_secret_id("your-secret")
        .set_secret(
            Secret::new().set_replication(Replication::new().set_automatic(Automatic::new())),
        )
        .send()
        .await?;
    println!("CREATE = {secret:?}");

    let tag = |mut labels: HashMap<_, _>, msg: &str| {
        labels.insert("uploaded".to_string(), msg.to_string());
        labels
    };

    let update = client
        .update_secret()
        .set_secret(
            Secret::new()
                .set_name(&secret.name)
                .set_etag(secret.etag)
                .set_labels(tag(secret.labels, "your-label"))
                .set_annotations(tag(secret.annotations, "your-annotations")),
        )
        .set_update_mask(FieldMask::default().set_paths(["annotations", "labels"]))
        .send()
        .await?;
    println!("UPDATE = {update:?}");

    Ok(())
}
