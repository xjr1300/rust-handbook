# 6.1 オブジェクトの書き込み時にデータをプッシュする

[Cloud Storage](https://cloud.google.com/storage)オブジェクトを書き込むクライアントAPIは、アプリケーションによって提供される型（例：ファイルハンドル、バイト列、ストリームなど）からペイロード（データを）をプッシュします。
あるアプリケーションはスレッド内でペイロードを生成し、サービスにオブジェクトのペイロードを「プッシュ」します。

> Cloud Storageにデータを書き込むAPIは、アプリケーションが用意したデータソース（ファイルやストリームなど）から内容をプルして（読み取って）、オブジェクトとしてアップロードする。

このガイドは、データソースのプッシュを使用して、[Cloud Storage](https://cloud.google.com/storage)にオブジェクトを書き込む方法を紹介します。

## 事前条件

このガイドは、[支払いが有効](https://cloud.google.com/billing/docs/how-to/verify-billing-enabled#confirm_billing_is_enabled_on_a_project)な[Google Cloudプロジェクト](https://cloud.google.com/resource-manager/docs/creating-managing-projects)と、そのプロジェクトにCloud Storageバケットが存在していることを想定しています。

## 依存関係としてクライアントライブラリを追加する

```sh
cargo add google-cloud-storage
```

## キューを`StreamingSource`に変換する

鍵となるアイデアは、キューを使用して、新しいデータをプッシュするタスクとペイロードをプルするタスクを分離することです。
このチュートリアルはTokioの[mpscキュー](https://docs.rs/tokio/latest/tokio/sync/mpsc/index.html)を使用しますが、Tokio非同期ランタイムと統合する任意のキューを使用できます。

まず、独自の型に受信者をラップします。

```rust
use tokio::sync::mpsc::{self, Receiver};

#[derive(Debug)]
struct QueueSource(Receiver<bytes::Bytes>)
```

そして、Google Cloud Client Librariesに要求されるトレイトを実装します。

```rust
use google_cloud_storage::streaming_source::StreamingSource;

impl StreamingSource for QueueSource {
    type Error = std::convert::Infallible;

    async fn next(&mut self) -> Option<Result<bytes::Bytes, Self::Error>> {
        self.0.recv().await.map(Ok)
    }
}
```

このチュートリアルでは、バケットとオブジェクトの名前を引数として受け取る関数内に、残りのコードを記述します。

```rust
pub async fn queue(bucket_name: &str, object_name: &str) -> anyhow::Result<()> {
    // ... ここにコードが入ります ...
    Ok(())
}
```

クライアントを初期化します。

```rust
use google_cloud_storage::client::Storage;

let client = Storage::builder().build().await?;
```

キューを作成し、受信者と送信者を得ます。

```rust
let (sender, receiver) = mpsc::channel::<bytes::Bytes>(32);
```

このキューから受け取ったデータでオブジェクトを書き込むクライアントを使用します。
`write_object()`メソッド内で作成したフューチャーを`await`していないことに注意してください。

```rust
let upload = client
    .write_object(bucket_name, object_name, QueueSource(receiver))
    .send_buffered();
```

バックグランドでキューを処理して、データを書き込むタスクを作成します。

```rust
let task = tokio::spawn(upload);
```

メインタスクにおいて、書き込むデータを送信します。

```rust
for _ in 0..1000 {
    let line = "I will not write funny examples in class\n";
    sender
        .send(bytes::Bytes::from_static(line.as_bytes()))
        .await?;
}
```

データの送信を終了したら、キューの送信側を閉じるために送信者をドロップします。

```rust
drop(sender);
```

これで、タスクが終了するのを待ち、結果を抽出できます。

```rust
let object = task.await??;
println!("object successfully uploaded {object:?}");
```

## 完全なプログラム

```rust
use google_cloud_storage::{client::Storage, streaming_source::StreamingSource};
use tokio::sync::mpsc::{self, Receiver};

#[derive(Debug)]
struct QueueSource(Receiver<bytes::Bytes>);

impl StreamingSource for QueueSource {
    type Error = std::convert::Infallible;

    async fn next(&mut self) -> Option<Result<bytes::Bytes, Self::Error>> {
        self.0.recv().await.map(Ok)
    }
}

pub async fn queue(bucket_name: &str, object_name: &str) -> anyhow::Result<()> {
    let client = Storage::builder().build().await?;

    let (sender, receiver) = mpsc::channel::<bytes::Bytes>(32);
    let upload = client
        .write_object(bucket_name, object_name, QueueSource(receiver))
        .send_buffered();
    let task = tokio::spawn(upload);

    for _ in 0..1000 {
        let line = "I will not write funny examples in class\n";
        sender
            .send(bytes::Bytes::from_static(line.as_bytes()))
            .await?;
    }
    drop(sender);
    let object = task.await??;
    println!("object successfully uploaded {object:?}");

    Ok(())
}
```
