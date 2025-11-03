//! このサンプルを実行するために、サンプルプログラム内で階層的名前空間を有効にしたバケットを作成している。
use anyhow::anyhow;
use google_cloud_longrunning as long_running;
use google_cloud_lro::{Poller, PollingResult};
use google_cloud_storage::client::StorageControl;
use google_cloud_storage::model::bucket::iam_config::UniformBucketLevelAccess;
use google_cloud_storage::model::bucket::{HierarchicalNamespace, IamConfig};
use google_cloud_storage::model::{Bucket, Folder, RenameFolderMetadata};

const PROJECT_ID: &str = "gcp-for-rust";

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let control = StorageControl::builder().build().await?;
    let bucket_name = format!("my-bucket-{}", uuid::Uuid::new_v4());
    let bucket = control
        .create_bucket()
        .set_parent("projects/_")
        .set_bucket_id(&bucket_name)
        .set_bucket(
            Bucket::new()
                .set_project(format!("projects/{PROJECT_ID}"))
                .set_hierarchical_namespace(HierarchicalNamespace::new().set_enabled(true))
                .set_iam_config(IamConfig::new().set_uniform_bucket_level_access(
                    UniformBucketLevelAccess::new().set_enabled(true),
                )),
        )
        .send()
        .await?;
    println!("created bucket: {}", bucket.name);

    test(&control, &bucket.name).await?;

    Ok(())
}

async fn test(control: &StorageControl, bucket_id: &str) -> anyhow::Result<()> {
    for folder_id in ["manual/", "automatic/", "polling/"] {
        let folder = control
            .create_folder()
            .set_parent(bucket_id)
            .set_folder_id(folder_id)
            .send()
            .await?;
        println!("created folder {folder_id}: {folder:?}");
    }
    let bucket_id = bucket_id
        .strip_prefix("projects/_/buckets/")
        .ok_or(anyhow!(
            "bad bucket name format {bucket_id}, should start with `projects/_/buckets/`"
        ))?;
    println!("bucket_id: {bucket_id}");

    println!("running manual LRO example");
    manual(bucket_id, "manual", "manual-renamed").await?;

    println!("running automatic LRO example");
    automatic(bucket_id, "automatic", "automatic-renamed").await?;

    println!("running automatic LRO with polling example");
    polling(bucket_id, "polling", "polling-renamed").await?;

    Ok(())
}

async fn manual(bucket: &str, folder: &str, dest: &str) -> anyhow::Result<()> {
    let client = StorageControl::builder().build().await?;

    let operation = client
        .rename_folder()
        .set_name(format!("projects/_/buckets/{bucket}/folders/{folder}"))
        .set_destination_folder_id(dest)
        .send()
        .await?;
    println!("LRO started, response={operation:?}");

    let mut operation = operation;
    let response: anyhow::Result<Folder> = loop {
        if operation.done {
            match &operation.result {
                None => {
                    break Err(anyhow!("missing result for finished operation"));
                }
                Some(r) => {
                    break match r {
                        long_running::model::operation::Result::Error(s) => {
                            Err(anyhow!("operation completed with error {s:?}"))
                        }
                        long_running::model::operation::Result::Response(any) => {
                            let response = any.to_msg::<Folder>()?;
                            Ok(response)
                        }
                        _ => Err(anyhow!("unexpected result branch {r:?}")),
                    };
                }
            }
        }
        if let Some(any) = &operation.metadata {
            let metadata = any.to_msg::<RenameFolderMetadata>()?;
            println!("LRO in progress, metadata={metadata:?}");
        }
        tokio::time::sleep(std::time::Duration::from_millis(500)).await;
        if let Ok(attempt) = client
            .get_operation()
            .set_name(&operation.name)
            .send()
            .await
        {
            operation = attempt;
        }
    };
    println!("LRO completed, response={response:?}");

    Ok(())
}

async fn automatic(bucket: &str, folder: &str, dest: &str) -> anyhow::Result<()> {
    let client = StorageControl::builder().build().await?;

    let response = client
        .rename_folder()
        .set_name(format!("projects/_/buckets/{bucket}/folders/{folder}"))
        .set_destination_folder_id(dest)
        .poller()
        .until_done()
        .await?;

    println!("LRO completed, response={response:?}");

    Ok(())
}

async fn polling(bucket: &str, folder: &str, dest: &str) -> anyhow::Result<()> {
    let client = StorageControl::builder().build().await?;

    let mut poller = client
        .rename_folder()
        .set_name(format!("projects/_/buckets/{bucket}/folders/{folder}"))
        .set_destination_folder_id(dest)
        .poller();

    while let Some(p) = poller.poll().await {
        match p {
            PollingResult::Completed(r) => {
                println!("LRO completed, response={r:?}");
            }
            PollingResult::InProgress(m) => {
                println!("LRO in progress, metadata={m:?}")
            }
            PollingResult::PollingError(e) => {
                println!("Transient error polling the LRO: {e}");
            }
        }
        tokio::time::sleep(std::time::Duration::from_millis(500)).await;
    }

    Ok(())
}
