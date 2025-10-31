//! # 分割ダウンロードのストライプサイズ
//!
//! | 推奨されるストライプサイズ| 理由 |
//! | --- | --- |
//! | 8MiB ($8,388,608$ Byte) | 多くのクラウドストレージ環境でバランスの取れた良いパフォーマンスを示す標準的なサイズで、APIのオーバーヘッドを十分に吸収 |
//! | 16MiB ($16,777,216$ Byte) | 比較的安定した高速ネットワーク接続がある場合に推奨 |
//! | 32MiB ($33,554,432$ Byte) | 非常に高速で低遅延なネットワーク環境（例：GCP内のVM間）での最大スループットを目指す場合 |
use google_cloud_storage::client::{Storage, StorageControl};
use google_cloud_storage::model::compose_object_request::SourceObject;
use google_cloud_storage::model::{Bucket, Object};
use google_cloud_storage::model_ext::ReadRange;
use tokio::io::{AsyncSeekExt as _, AsyncWriteExt as _};

use cloud_storage::{PROJECT_ID, create_bucket};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let client = Storage::builder().build().await?;
    let control = StorageControl::builder().build().await?;

    let bucket_name = format!("my-bucket-{}", uuid::Uuid::new_v4());
    let file_sizes = [2, 4 /*, 8, 16, 32*/];

    // バケットを作成
    let bucket = create_bucket(&control, PROJECT_ID, &bucket_name).await?;
    println!("bucket successfully created {bucket:?}");

    // 巨大なファイルを作成
    let filenames = seed(&client, &control, &bucket.name, &file_sizes).await?;

    // 巨大なファイルをダウンロード
    let destination = format!("outputs/{}", filenames.last().unwrap());
    download(
        &client,
        &control,
        &bucket,
        filenames.last().unwrap(),
        8 * 1024 * 1024,
        &destination,
    )
    .await?;

    //// バケットを削除
    //control
    //    .delete_bucket()
    //    .set_name(&bucket.name)
    //    .send()
    //    .await?;

    Ok(())
}

/// Cloud Storageに巨大なファイルを作成
async fn seed(
    client: &Storage,
    control: &StorageControl,
    bucket_id: &str,
    file_sizes: &[i32],
) -> anyhow::Result<Vec<String>> {
    // 1MiB(1024 * 1024 =1,048,576 Byte)のバッファーを作成
    let buffer = String::from_iter(('a'..='z').cycle().take(1024 * 1024));
    // 上記バッファーをGCSにアップロードして、1MiB.txtファイルとして保存
    let seed = client
        .write_object(bucket_id, "1MiB.txt", bytes::Bytes::from_owner(buffer))
        .send_unbuffered()
        .await?;
    println!(
        "Uploaded object {}, size={}KiB",
        seed.name,
        seed.size / 1024
    );

    // 32MiBのサイズを持つファイルを作成
    // ソースオブジェクトとして、1MiB.txtを指定して、32回合成
    // 1MiB * 32 = 32MiB
    let seed_32 = control
        .compose_object()
        .set_destination(Object::new().set_bucket(bucket_id).set_name("32MiB.txt"))
        .set_source_objects((0..32).map(|_| {
            SourceObject::new()
                .set_name(&seed.name)
                .set_generation(seed.generation)
        }))
        .send()
        .await?;
    println!(
        "Create object {}, size-{}MiB",
        seed_32.name,
        seed_32.size / (1024 * 1024)
    );

    // 1GiBのサイズを持つファイルを作成
    // 32MiB * 32 = 1024MiB = 1GiB
    let seed_1024 = control
        .compose_object()
        .set_destination(Object::new().set_bucket(bucket_id).set_name("1GiB.txt"))
        .set_source_objects((0..32).map(|_| {
            SourceObject::new()
                .set_name(&seed_32.name)
                .set_generation(seed_32.generation)
        }))
        .send()
        .await?;
    println!(
        "Created object{}, size={}MiB",
        seed_1024.name,
        seed_1024.size / (1024 * 1024)
    );

    // 1GiB.txtファイルを合成して巨大なファイルを
    let mut filenames = vec![];
    for s in file_sizes {
        let name = format!("{s}GiB.txt");
        let target = control
            .compose_object()
            .set_destination(Object::new().set_bucket(bucket_id).set_name(&name))
            .set_source_objects((0..*s).map(|_| {
                SourceObject::new()
                    .set_name(&seed_1024.name)
                    .set_generation(seed_1024.generation)
            }))
            .send()
            .await?;
        println!(
            "Created object {}, size={}MiB",
            target.name,
            target.size / (1024 * 1024)
        );
        filenames.push(name);
    }
    Ok(filenames)
}

// 巨大なファイルをダウンロード
async fn download(
    client: &Storage,
    control: &StorageControl,
    bucket: &Bucket,
    filename: &str,
    stripe_size: usize,
    destination: &str,
) -> anyhow::Result<()> {
    let metadata = control
        .get_object()
        .set_bucket(&bucket.name)
        .set_object(filename)
        .send()
        .await?;

    let file = tokio::fs::File::create(destination).await?;
    let start = std::time::Instant::now();

    let size = metadata.size as u64;
    let limit = stripe_size as u64;
    let count = size / limit;
    let mut stripes = (0..count)
        .map(|i| write_stripe(client.clone(), &file, i * limit, limit, &metadata))
        .collect::<Vec<_>>();
    // ダウンロードするファイルについて、ファイル末尾にあるストライプサイズ未満の残りのデータをダウンロードして、ファイルに書き込み
    if !size.is_multiple_of(limit) {
        stripes.push(write_stripe(
            client.clone(),
            &file,
            count * limit,
            limit,
            &metadata,
        ));
    }

    futures::future::join_all(stripes)
        .await
        .into_iter()
        .collect::<anyhow::Result<Vec<_>>>()?;

    let elapsed = start.elapsed();
    let mib = metadata.size as f64 / (1024.0 * 1024.0);
    let bw = mib / elapsed.as_secs_f64();
    println!(
        "Completed {mib:.2} MiB download in {elapsed:?}, using {count} stripes, effective bandwidth = {bw:.2} MiB/s"
    );

    Ok(())
}

async fn write_stripe(
    client: Storage,
    file: &tokio::fs::File,
    offset: u64,
    limit: u64,
    metadata: &Object,
) -> anyhow::Result<()> {
    let mut writer = file.try_clone().await?;
    writer.seek(std::io::SeekFrom::Start(offset)).await?;
    let mut reader = client
        .read_object(&metadata.bucket, &metadata.name)
        .set_generation(metadata.generation)
        .set_read_range(ReadRange::segment(offset, limit))
        .send()
        .await?;
    while let Some(b) = reader.next().await.transpose()? {
        writer.write_all(&b).await?;
    }

    Ok(())
}
