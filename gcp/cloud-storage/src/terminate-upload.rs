use std::error::Error;
use std::fmt::Display;

use google_cloud_storage::client::{Storage, StorageControl};
use google_cloud_storage::streaming_source::StreamingSource;

use cloud_storage::{PROJECT_ID, create_bucket};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // バケットを作成
    let control = StorageControl::builder().build().await?;
    let bucket_name = format!("my-bucket-{}", uuid::Uuid::new_v4());
    let bucket = create_bucket(&control, PROJECT_ID, &bucket_name).await?;
    println!("bucket successfully created {bucket:?}");

    let client = Storage::builder().build().await?;
    let upload = client
        .write_object(&bucket.name, "expect-error", MySource::default())
        .send_buffered()
        .await;
    println!("Upload result {upload:?}");
    let err = upload.expect_err("the source is supposed to terminate the upload");
    assert!(err.is_serialization(), "{err:?}");
    assert!(err.source().is_some_and(|e| e.is::<MyError>()), "{err:?}");

    Ok(())
}

#[derive(Debug)]
pub enum MyError {
    ExpectedProblem,
    OhNoes,
}

impl Error for MyError {}

impl Display for MyError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ExpectedProblem => write!(f, "this kind of thing happens"),
            Self::OhNoes => write!(f, "oh noes! something terrible happened"),
        }
    }
}

#[derive(Debug, Default)]
struct MySource(u32);

impl StreamingSource for MySource {
    type Error = MyError;

    async fn next(&mut self) -> Option<Result<bytes::Bytes, Self::Error>> {
        self.0 += 1;
        match self.0 {
            42 => Some(Err(MyError::ExpectedProblem)),
            n if n > 42 => None,
            n => Some(Ok(bytes::Bytes::from_owner(format!(
                "test data for the example {n}\n"
            )))),
        }
    }
}
