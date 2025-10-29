//! ```sh
//! cargo run --package=cloud-storage --bin=quickstart -- <project-id>
//! ```
use google_cloud_storage as gcs;
use google_cloud_storage::client::{Storage, StorageControl};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let project_id = std::env::args().nth(1).unwrap();
    let bucket_id = format!("my-bucket-{}", uuid::Uuid::new_v4());

    let control = StorageControl::builder().build().await?;
    let bucket = control
        .create_bucket()
        .set_parent("projects/_")
        .set_bucket_id(bucket_id)
        .set_bucket(
            gcs::model::Bucket::new()
                .set_project(format!("projects/{project_id}"))
                .set_iam_config(
                    gcs::model::bucket::IamConfig::new().set_uniform_bucket_level_access(
                        gcs::model::bucket::iam_config::UniformBucketLevelAccess::new()
                            .set_enabled(true),
                    ),
                ),
        )
        .send()
        .await?;
    println!("bucket successfully created {bucket:?}");

    let client = Storage::builder().build().await?;
    let object = client
        .write_object(&bucket.name, "hello.txt", "Hello World!")
        .send_buffered()
        .await?;
    println!("object successfully uploaded {object:?}");

    let mut reader = client.read_object(&bucket.name, "hello.txt").send().await?;
    let mut contents = Vec::new();
    while let Some(chunk) = reader.next().await.transpose()? {
        contents.extend_from_slice(&chunk);
    }
    println!(
        "object contents successfully downloaded: {:?}",
        bytes::Bytes::from_owner(contents)
    );

    control
        .delete_object()
        .set_bucket(&bucket.name)
        .set_object(&object.name)
        .set_generation(object.generation)
        .send()
        .await?;
    control
        .delete_bucket()
        .set_name(&bucket.name)
        .send()
        .await?;

    Ok(())
}
