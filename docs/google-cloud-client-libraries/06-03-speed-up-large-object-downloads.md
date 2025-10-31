# 6.3 大きなオブジェクトのダウンロード速度の向上

<https://googleapis.github.io/google-cloud-rust/storage/striped_downloads.html>

このチュートリアルでは、大きな[Cloud Storage](https://cloud.google.com/storage)のダウンロードを高速化するために、ストライプダウンロードを使用する方法を学びます。

## 前提条件

このガイドは、[支払いが有効](https://cloud.google.com/billing/docs/how-to/verify-billing-enabled#confirm_billing_is_enabled_on_a_project)な[Google Cloudプロジェクト](https://cloud.google.com/resource-manager/docs/creating-managing-projects)と、そのプロジェクトにCloud Storageバケットがあることを想定しています。

このチュートリアルで大きなオブジェクトを作成するため、過剰な支払いを避けるためにリソースをクリーンアップすることを忘れないでください。

このチュートリアルは、クライアントライブラリを使用する基礎を持っていることを想定しています。
もし、そうでない場合、[クイックスタートガイド](https://googleapis.github.io/storage.html#quickstart)を読んでください。

## 依存関係にクライアントライブラリを追加

```sh
cargo add google-cloud-storage
```

## ソースデータの作成

このチュートリアルを実行するために、Cloud Storageに大きなオブジェクトが必要です。
希望するサイズのオブジェクトを作成するために、小さなオブジェクトを作成し、繰り返しそれを合成することで、そのようなオブジェクトを作成できます。

独自の関数内にデータを作成するすべてのコードを入れることができます。
この関数は、引数でストレージとストレージコントロールクライアントを受け取ります。
これらのクライアントを作成する方法に関する情報は、[クイックスタートガイド](https://googleapis.github.io/storage.html#quickstart)を参照してください。

```rust
use google_cloud_storage::client::{Storage, StorageControl};
use google_cloud_storage::model::Object;

async fn seed(client: Storage, control: StorageControl, bucket_name: &str) -> anyhow::Result<()> {
    // ... 詳細は省略 ...
    Ok(())
}
```

通常、関数は、コードを簡素化するために、ある`use`宣言で始まります。

```rust
use google_cloud_storage::model::compose_object_request::SourceObject;
```

ストレージクライアントを使用して、1MiBオブジェクトを作成します。

```rust
let buffer = String::from_iter(('a'..='z').cycle().take(1024 * 1024));
let seed = client
    .write_object(bucket_name, "1MiB.txt", bytes::Bytes::from_owner(buffer))
    .send_unbuffered()
    .await?;
println!("Uploaded object {}, size={}KiB", seed.name, seed.size / 1024);
```

次に、ストレージコントロールクライアントを使用して、このオブジェクトの32コピーを連結し、大きなオブジェクトにします。
この操作は、オブジェクトデータをクライアントに転送する必要はなく、サービスによって実行されます。

```rust
let seed_32 = control
    .compose_object()
    .set_destination(Object::new().set_bucket(bucket_name).set_name("32MiB.txt"))
    .set_source_objects((0..32).map(|_| {
        SourceObject::new()
            .set_name(&seed.name)
            .set_generation(seed.generation)
    }))
    .send()
    .await?;
println!("Created object {}, size={}MiB", seed.name, seed.size / (1024 * 1024));
```

より大きなオブジェクトを作成するために、操作を繰り返すことができます。

```rust
let seed_1024 = control
    .compose_object()
    .set_destination(Object::new().set_bucket(bucket_name).set_name("1GiB.txt"))
    .set_source_objects((0..32).map(|_| {
        SourceObject::new()
            .set_name(&seed_32.name)
            .set_generation(&seed_32.generation)
    }))
    .send()
    .await?;
println!("Created object {}, size={}MiB", seed_1024.name, seed_1024.size / (1024 * 1024));
for s in [2, 4, 8, 16, 32] {
    let name = format!("{s}GiB.txt");
    let target = control
        .compose_object()
        .set_destination(Object::new().set_bucket(bucket_name).set_name(&name))
        .set_source_objects((0..s).map(|_| {
            SourceObject::new()
                .set_name(&seed_1024.name)
                .set_generation(seed_1024.generation)
        }))
        .send()
        .await?;
    println!("Created object {} size={}MiB", target.name, target.size / (1024 * 1024));
}
```

## ストライプダウンロード

再度、ストライプダウンロードを実行する関数を記述します。

```rust
async fn download(
    client: Storage,
    control: StorageControl,
    bucket_name: &str,
    object_name: &str,
    stripe_size: usize,
    destination: &str,
) -> anyhow::Result<()> {
    // ... 詳細は後述 ...
    Ok(())
}
```

ストレージコントロールクライアントを使用し、オブジェクトのメタデータを問い合わせます。
このメタデータは、オブジェクトのサイズと現在のジェネレーションを含んでいます。

```rust
let metadata = control
    .get_object()
    .set_bucket(bucket_name)
    .set_object(object_name)
    .send()
    .await?;
```

それぞれのストライプのダウンロードを別の関数に分割します。
すぐにこの関数の詳細を確認しますが、現時点では、それが非同期であるため、`Future`を返すことだけに注意してください。

```rust
async fn write_stripe(
    client: Storage,
    file: &tokio::fs::File,
    offset: u64,
    limit: u64,
    metadata: &Object,
) -> anyhow::Result<()> {
    use google_cloud_storage::model_ext::ReadRange;
    use tokio::io::AsyncSeekExt;
    // ... 詳細は後述 ...
    Ok(())
}
```

それぞれのストライプのサイズを計算し、次に`write_stripe()`を呼び出して、これらのストライプをそれぞれダウンロードできます。
そして、結果をベクターに集めます。

```rust
let size = metadata.size as u64;
let limit = stripe_size as u64;
let count = size / limit;
let mut stripes = (0..count)
    .map(|i| write_stripe(client.clone(), &file, i * limit, limit, &metadata))
    .collect::<Vec<_>>();
if size % limit != 0 {
    stripes.push(write_stripe(
        client.clone(),
        &file,
        count * limit,
        limit,
        &metadata,
    ))
}
```

標準のRustの機能を使用して、これらすべてのフューチャーを同時に待つことができます。

```rust
futures::future::join_all(stripes)
    .await
    .into_iter()
    .collect::<anyhow::Result<Vec<_>>>()?;
```

それらが完了すると、ファイルがダウンロードされます。

次に、`write_stripe()`関数の記述に進みます。
まず、書き込み先を複製し、目的のオフセットから書き込みを開始する準備をします。

```rust
let mut writer = file.try_clone().await?;
writer.seek(std::io::SeekFrom::Start(offset)).await?;
```

Cloud Storageからのダウンロードを開始します。

```rust
let mut reader = client
    .read_object(&metadata.bucket, &metadata.name)
    .send()
    .await?;
```

目的のストライプにダウンロードを制限するために、`.with_read_offset()`と`.with_read_limit()`を使用します。

```rust
    .set_read_range(ReadRange::segment(offset, limit))
```

また、正しいオブジェクトのジェネレーションをダウンロードするように制限したいかもしれません。
これは、他のプロセスがそのオブジェクトを上書きし、一貫性のない読み込みになる競合状態を避けます。

```rust
    .set_generation(metadata.generation)
```

その後、データを読み込み、ローカルファイルにそれを書き込みます。

```rust
while let Some(b) = reader.next().await.transpose()? {
    use tokio::io::AsyncWriteExt;
    writer.write_all(&b).await?;
}
```

## 次のステップ

最後のストライプが少しのバイトしかない場合に最適化することを考えてください。

## 予想される性能

これらのダウンロードの性能は、次に依存します。

- I/Oサブシステム: ローカルストレージが十分に速くない場合、ダウンロードはディスクへの書き込みによって抑制されます。
- VMの構成: 十分なCPUがない場合、Cloud Storageとクライアントライブラリは、転送中に常にデータを暗号化するため、データの復号化を試行する際に、ダウンロードが抑制されます。
- バケットと特定のオブジェクトの場所: バケットがVMの場所と異なるリージョンにすべてのオブジェクト（またはいくつかのオブジェクト）を蓄積しているかもしれません。
  この場合、広域ネットワークの容量によって抑制されるかもしれません。

> 3つ目は、Cloud Storageのバケット/オブジェクトの保存場所（リージョン）と、ダウンロードを実行しているVMの場所（リージョン）が異なる場合に、ダウンロード速度が低下する可能性を指摘している。

十分に大きなVM、SSD、そして同じリージョン内のVMを使用すると、1,000MiB/sに近い効率的なスループットを得られるはずです。

## 完全なプログラム

```rust

use google_cloud_storage::client::Storage;
use google_cloud_storage::client::StorageControl;
use google_cloud_storage::model::Object;

async fn seed(client: Storage, control: StorageControl, bucket_name: &str) -> anyhow::Result<()> {
    use google_cloud_storage::model::compose_object_request::SourceObject;

    let buffer = String::from_iter(('a'..='z').cycle().take(1024 * 1024));
    let seed = client
        .write_object(bucket_name, "1MiB.txt", bytes::Bytes::from_owner(buffer))
        .send_unbuffered()
        .await?;
    println!(
        "Uploaded object {}, size={}KiB",
        seed.name,
        seed.size / 1024
    );

    let seed_32 = control
        .compose_object()
        .set_destination(Object::new().set_bucket(bucket_name).set_name("32MiB.txt"))
        .set_source_objects((0..32).map(|_| {
            SourceObject::new()
                .set_name(&seed.name)
                .set_generation(seed.generation)
        }))
        .send()
        .await?;
    println!(
        "Created object {}, size={}MiB",
        seed.name,
        seed.size / (1024 * 1024)
    );

    let seed_1024 = control
        .compose_object()
        .set_destination(Object::new().set_bucket(bucket_name).set_name("1GiB.txt"))
        .set_source_objects((0..32).map(|_| {
            SourceObject::new()
                .set_name(&seed_32.name)
                .set_generation(seed_32.generation)
        }))
        .send()
        .await?;
    println!(
        "Created object {}, size={}MiB",
        seed.name,
        seed.size / (1024 * 1024)
    );

    for s in [2, 4, 8, 16, 32] {
        let name = format!("{s}GiB.txt");
        let target = control
            .compose_object()
            .set_destination(Object::new().set_bucket(bucket_name).set_name(&name))
            .set_source_objects((0..s).map(|_| {
                SourceObject::new()
                    .set_name(&seed_1024.name)
                    .set_generation(seed_1024.generation)
            }))
            .send()
            .await?;
        println!(
            "Created object {} size={} MiB",
            target.name,
            target.size / (1024 * 1024)
        );
    }

    Ok(())
}

async fn download(
    client: Storage,
    control: StorageControl,
    bucket_name: &str,
    object_name: &str,
    stripe_size: usize,
    destination: &str,
) -> anyhow::Result<()> {
    let metadata = control
        .get_object()
        .set_bucket(bucket_name)
        .set_object(object_name)
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
    if size % limit != 0 {
        stripes.push(write_stripe(
            client.clone(),
            &file,
            count * limit,
            limit,
            &metadata,
        ))
    }

    futures::future::join_all(stripes)
        .await
        .into_iter()
        .collect::<anyhow::Result<Vec<_>>>()?;

    let elapsed = std::time::Instant::now() - start;
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
    use google_cloud_storage::model_ext::ReadRange;
    use tokio::io::AsyncSeekExt;
    let mut writer = file.try_clone().await?;
    writer.seek(std::io::SeekFrom::Start(offset)).await?;
    let mut reader = client
        .read_object(&metadata.bucket, &metadata.name)
        .set_generation(metadata.generation)
        .set_read_range(ReadRange::segment(offset, limit))
        .send()
        .await?;
    while let Some(b) = reader.next().await.transpose()? {
        use tokio::io::AsyncWriteExt;
        writer.write_all(&b).await?;
    }
    Ok(())
}
```
