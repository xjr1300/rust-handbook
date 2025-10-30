# 6.2 オブジェクトの書き換え

<https://googleapis.github.io/google-cloud-rust/storage/rewrite_object.html>

[Cloud Storage](https://cloud.google.com/storage)オブジェクトの書き換えは、操作の[詳細](https://cloud.google.com/storage/docs/json_api/v1/objects/rewrite)によって、複数のクライアントリクエストが必要になる場合があります。
そのような場合、サービスは、クライアントが操作を継続するために使用しなければならない`rewrite_token`を伴って、その時点の進捗を表現するレスポンスを返します。

このガイドは、Cloud Storageオブジェクトに対して書き換えループを完全に実行する方法を紹介します。

## 前提条件

このガイドは、[支払いが有効](https://cloud.google.com/billing/docs/how-to/verify-billing-enabled#confirm_billing_is_enabled_on_a_project)な[Google Cloudプロジェクト](https://cloud.google.com/resource-manager/docs/creating-managing-projects)と、そのプロジェクトにCloud Storageバケットが存在していることを想定しています。

## 依存関係としてクライアントライブラリを追加する

```sh
cargo add google-cloud-storage
```

## オブジェクトの書き換え

### クライアントの準備

まず、クライアントを作成します。

サービスは、少なくとも30秒以上の[全体的なタイムアウト](https://docs.rs/google-cloud-gax/latest/google_cloud_gax/retry_policy/trait.RetryPolicyExt.html#method.with_time_limit)を設定することを推奨しています。
この例では、操作にタイムアウトを設定しない`RetryPolicy`を使用します。

```rust
let control = StorageControl::builder()
    .with_retry_policy(RetryableErrors.with_attempt_limit(5))
    .build()
    .await?;
```

### ビルダーの準備

次にリクエストビルダーを準備します。

```rust
let mut builder = control
    .rewrite_object()
    .set_source_bucket(bucket_name)
    .set_source_object(&source_object.name)
    .set_destination_bucket(bucket_name)
    .set_destination_name("rewrite-object-clone");
```

オプションとして、サービスが進捗レポートで応答する前に、呼び出しごとに書き込まれる最大バイト数を制限できます。
このオプションを設定することは、試行タイムアウトを増加させることに対する代替手段です。

この例で使用されている値を意図的に小さくすることで、複数の反復が必要な再書き込みループを強制していることに注意してください。
実際には、より大きな値を使用することになります。

```rust
builder = builder.set_max_bytes_rewritten_per_call(1024 * 1024);
```

オブジェクトの書き換えは、そのデータを異なるバケットにコピー、同じバケットの異なるオブジェクトにコピー、暗号鍵の変更、そして／またはその[ストレージクラス](https://cloud.google.com/storage/docs/storage-classes)を変更できるようにします。
書き換えループは、これらすべての変換で同一です。
コードを説明するために、ストレージクラスを変更します。

```rust
builder = builder.set_destination(Object::new().set_storage_class("NEARLINE"));
```

新しいストレージクラスに関連する[最小のストレージ期間](https://cloud.google.com/storage/pricing#early-delete)があることに注意してください。
この例（3MiB）で使用されるオブジェクトは、`$0.001`未満のコストが発生しますが、より大きなオブジェクトの場合、請求額が高額になるかもしれません。

### `rewrite_until_done`ヘルパーの使用

ライブラリは自動で書き換えループを実行するヘルパー関数を提供しています。

```rust
use google_cloud_storage::builder_ext::RewriteObjectExt;
let dest_object = builder.rewrite_until_done().await?;
println!("dest_object={dest_object:?}");
```

このヘルパー関数は書き換えトークンを扱い、書き換えが終了するまで操作を継続します。

### 手動による書き換えループの実行

もし、書き換えループをより制御する必要がある場合、それを自動で実行できます。
まず、書き換えループの反復を実行するヘルパー関数を導入します。
リクエストを送信して、レスポンスを処理します。
進捗状況をログに記録します。
もし、操作が終了した場合、オブジェクトのメタデータを返し、そうでない場合は書き換えトークンを返します。

```rust
enum RewriteProgress {
    // これは書き換えトークンを保持します。
    Incomplete(String),
    Done(Box<Object>),
}

async fn make_one_request(builder: RewriteObject) -> Result<RewriteProgress> {
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
        resp.total_bytes_rewritten, resp.object_size,
    );
    Ok(RewriteProgress::Incomplete(resp.rewrite_token))
}
```

### 書き換えループの実行

これで、操作が終了するまで書き換えループを実行する準備ができました。

```rust
let dest_object = loop {
    let progress = make_one_request(builder.clone()).await?;
    match progress {
        RewriteProgress::Incomplete(rewrite_token) => {
            builder = builder.set_rewrite_token(rewrite_token);
        }
        RewriteProgress:Done(object) => break object,
    };
};
println!("dest_object={dest_object:?}");
```

操作が完了していない場合、サーバーから返された書き換えトークンを次のリクエストに提供していることに注意してください。

```rust
RewriteProgress::Incomplete(rewrite_token) => {
    builder = builder.set_rewrite_token(rewrite_token);
}
```

また、書き換えトークンは、他の処理から操作を継続するためにも使用できることに注意してください。
書き換えトークンは、最大1週間有効です。

## 完全なプログラム

これらの手順をすべてまとめて提供します。

```rust
use gcs::Result;
use gcs::builder::storage_control::RewriteObject;
use gcs::client::StorageControl;
use gcs::model::Object;
use gcs::retry_policy::RetryableErrors;
use google_cloud_gax::retry_policy::RetryPolicyExt as _;
use google_cloud_storage as gcs;

pub async fn rewrite_object(bucket_name: &str) -> anyhow::Result<()> {
    let source_object = upload(bucket_name).await?;

    let control = StorageControl::builder()
        .with_retry_policy(RetryableErrors.with_attempt_limit(5))
        .build()
        .await?;

    let mut builder = control
        .rewrite_object()
        .set_source_bucket(bucket_name)
        .set_source_object(&source_object.name)
        .set_destination_bucket(bucket_name)
        .set_destination_name("rewrite-object-clone");

    // オプションでリクエストあたりの最大バイト数を制限
    builder = builder.set_max_bytes_rewritten_per_call(1024 * 1024);

    // オプションでストレージクラスを変更して、GCSにバイトコピーを強制
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

    cleanup(control, bucket_name, &source_object.name, &dest_object.name).await;
    Ok(())
}

enum RewriteProgress {
    // これは書き換えトークンを保持
    Incomplete(String),
    Done(Box<Object>),
}

async fn make_one_request(builder: RewriteObject) -> Result<RewriteProgress> {
    let resp = builder.send().await?;
    if resp.done {
        println!(
            "DONE:     total_bytes_rewritten={}; object_size={}",
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

// 書き換えるオブジェクトをアップロード
async fn upload(bucket_name: &str) -> anyhow::Result<Object> {
    let storage = gcs::client::Storage::builder().build().await?;
    // 書き換えトークンのロジックを実行するために、1MiBを超えるサイズが必要
    let payload = bytes::Bytes::from(vec![65_u8; 3 * 1024 * 1024]);
    let object = storage
        .write_object(bucket_name, "rewrite-object-source", payload)
        .send_unbuffered()
        .await?;
    Ok(object)
}

// このサンプルで作成したリソースをクリーンアップ
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
```

次のようなアプトプットを確認できるはずです。

```text
PROGRESS: total_bytes_rewritten=1048576; object_size=3145728
PROGRESS: total_bytes_rewritten=2097152; object_size=3145728
DONE:     total_bytes_rewritten=3145728; object_size=3145728
dest_object=Object { name: "rewrite-object-clone", ... }
```
