use google_cloud_storage::client::StorageControl;
use google_cloud_storage::model::Bucket;
use google_cloud_storage::model::bucket::IamConfig;
use google_cloud_storage::model::bucket::iam_config::UniformBucketLevelAccess;

pub const PROJECT_ID: &str = "gcp-for-rust";
pub const BUCKET_NAME: &str = "my-bucket-6624150a-c3ca-491a-8071-0561f63e7a0b";

pub fn bucket_id(bucket_name: &str) -> String {
    format!("projects/_/buckets/{bucket_name}")
}

pub async fn create_bucket(
    control: &StorageControl,
    project_id: &str,
    bucket_name: &str,
) -> anyhow::Result<Bucket> {
    let bucket = control
        .create_bucket()
        .set_parent("projects/_")
        .set_bucket_id(bucket_name)
        .set_bucket(
            Bucket::new()
                .set_project(format!("projects/{project_id}"))
                .set_iam_config(IamConfig::new().set_uniform_bucket_level_access(
                    UniformBucketLevelAccess::new().set_enabled(true),
                )),
        )
        .send()
        .await?;
    Ok(bucket)
}
