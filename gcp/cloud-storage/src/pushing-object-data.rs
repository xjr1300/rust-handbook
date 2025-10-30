use google_cloud_storage as gcs;
use google_cloud_storage::client::Storage;
use google_cloud_storage::client::StorageControl;
use google_cloud_storage::streaming_source::StreamingSource;
use tokio::sync::mpsc::{self, Receiver};

#[derive(Debug)]
struct QueueSource(Receiver<bytes::Bytes>);

impl StreamingSource for QueueSource {
    type Error = std::convert::Infallible;

    async fn next(&mut self) -> Option<Result<bytes::Bytes, Self::Error>> {
        self.0.recv().await.map(Ok)
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let project_id = std::env::args().nth(1).unwrap();
    let object_name = std::env::args().nth(2).unwrap();

    let bucket_id = format!("my-bucket-{}", uuid::Uuid::new_v4());
    println!("bucket: {bucket_id}");

    // バケットを作成
    let control = StorageControl::builder().build().await?;
    let bucket = control
        .create_bucket()
        .set_parent("projects/_")
        .set_bucket_id(bucket_id)
        .set_bucket(gcs::model::Bucket::new().set_project(format!("projects/{project_id}")))
        .send()
        .await?;
    println!("bucket successfully created {bucket:?}");

    let client = Storage::builder().build().await?;

    // データをプッシュしてオブジェクトを作成
    let (sender, receiver) = mpsc::channel::<bytes::Bytes>(1024);
    let upload = client
        .write_object(&bucket.name, object_name, QueueSource(receiver))
        .send_buffered();
    let task = tokio::spawn(upload);

    for _ in 0..1000 {
        let line = "I will not write funny examples in class\n";
        sender
            .send(bytes::Bytes::from_static(line.as_bytes()))
            .await?;
    }

    // 送信側をドロップして、チャネルを閉じる（受信側も閉じられる）
    drop(sender);

    let object = task.await??;
    println!("object successfully uploaded {object:?}");

    Ok(())
}
