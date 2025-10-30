use gcs::builder::storage_control::RewriteObject;
use gcs::client::StorageControl;
use gcs::model::Object;
use gcs::retry_policy::RetryableErrors;
use google_cloud_gax::retry_policy::RetryPolicyExt as _;
use google_cloud_storage as gcs;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let project_id = std::env::args().nth(1).unwrap();

    // バケットを作成
    let bucket_id = format!("my-bucket-{}", uuid::Uuid::new_v4());
    let control = StorageControl::builder().build().await?;
    let bucket = control
        .create_bucket()
        .set_parent("projects/_")
        .set_bucket_id(&bucket_id)
        .set_bucket(gcs::model::Bucket::new().set_project(format!("projects/{project_id}")))
        .send()
        .await?;
    println!("bucket successfully created {bucket:?}");

    let source_object = upload(&bucket.name).await?;

    let control = StorageControl::builder()
        .with_retry_policy(RetryableErrors.with_attempt_limit(5))
        .build()
        .await?;

    let mut builder = control
        .rewrite_object()
        .set_source_bucket(&bucket.name)
        .set_source_object(&source_object.name)
        .set_destination_bucket(&bucket.name)
        .set_destination_name("rewrite-object-clone");

    // オプションで、リクエストごとの最大書き込みバイト数を制限
    builder = builder.set_max_bytes_rewritten_per_call(1024 * 1024);

    // オプションで、バイトコピーをGCSに強制するためにストレージクラスを変更
    builder = builder.set_destination(Object::new().set_storage_class("NEARLINE"));

    let dest_object = loop {
        let progress = make_one_request(builder.clone()).await?;
        match progress {
            RewriteProgress::Incomplete(rewrite_token) => {
                builder = builder.set_rewrite_token(rewrite_token);
            }
            RewriteProgress::Done(object) => break object,
        };
    };
    println!("dest_object={dest_object:?}");

    cleanup(
        control,
        &bucket.name,
        &source_object.name,
        &dest_object.name,
    )
    .await;

    Ok(())
}

enum RewriteProgress {
    // これは書き換えトークンを保持
    Incomplete(String),
    Done(Box<Object>),
}

async fn make_one_request(builder: RewriteObject) -> gcs::Result<RewriteProgress> {
    let resp = builder.send().await?;
    if resp.done {
        println!(
            "DONE: total_bytes_rewritten={}; object_size={}",
            resp.total_bytes_rewritten, resp.object_size
        );
        return Ok(RewriteProgress::Done(Box::new(
            resp.resource
                .expect("A `done` response must have an object."),
        )));
    }
    println!(
        "PROGRESS: total_bytes_rewritten={}; object_size={}",
        resp.total_bytes_rewritten, resp.object_size
    );
    Ok(RewriteProgress::Incomplete(resp.rewrite_token))
}

// 書き換えするためにオブジェクトをアップロード
async fn upload(bucket_name: &str) -> anyhow::Result<Object> {
    let storage = gcs::client::Storage::builder().build().await?;
    // 書き換えトークンロジックを実行するために1MiBを超えるサイズが必要
    let payload = bytes::Bytes::from(vec![65_u8; 3 * 1024 * 1024]);
    let object = storage
        .write_object(bucket_name, "rewrite-object-source", payload)
        .send_unbuffered()
        .await?;
    Ok(object)
}

// この例で作成したリソースをクリーンアップ
async fn cleanup(control: StorageControl, bucket_name: &str, o1: &str, o2: &str) {
    let _ = control
        .delete_object()
        .set_bucket(bucket_name)
        .set_object(o1)
        .send()
        .await;
    let _ = control
        .delete_object()
        .set_bucket(bucket_name)
        .set_object(o2)
        .send()
        .await;
    let _ = control.delete_bucket().set_name(bucket_name).send().await;
}
