# 6.4 エラーを使用してオブジェクトの書き込みを中止する

<https://googleapis.github.io/google-cloud-rust/storage/terminate_uploads.html>

このガイドでは、エラーとカスタムのデータソースを使用して、完了前に、オブジェクトの書き込みを中止する方法を学びます。
アプリケーションがエラー状態になった際、クライアントライブラリにオブジェクトの作成を完了しないようにしたい場合に役に立ちます。

## 前提条件

このガイドは、[支払いが有効](https://cloud.google.com/billing/docs/how-to/verify-billing-enabled#confirm_billing_is_enabled_on_a_project)な[Google Cloudプロジェクト](https://cloud.google.com/resource-manager/docs/creating-managing-projects)と、そのプロジェクトにCloud Storageバケットがあることを想定しています。

このチュートリアルは、クライアントライブラリを使用するために必要な基礎知識があることを想定しています。
もし、そうでない場合、[クイックスタートガイド](https://googleapis.github.io/storage.html#quickstart)を読んでください。

## クライアントライブラリを依存関係に追加

```sh
cargo add google-cloud-storage
```

## 概要

クライアントライブラリは、`StreamingSource`トレイトを実装した任意の型を使用してオブジェクトを作成します。
クライアントライブラリは、トレイトの実装からデータを取得します（プルします）。
ライブラリは、データ取得中にエラーが発生すると、オブジェクトの書き込みを中止します。

このガイドでは、あるデータを提供した後、エラーで停止する`StreamingSource`のカスタム実装を作成します。
エラーを返すカスタムデータソースを使用して、オブジェクトの書き込みが中止されることを検証します。

## カスタムエラー型の作成

完了させずにオブジェクトの書き込みを中止するために、[StreamingSource](https://docs.rs/google-cloud-storage/latest/google_cloud_storage/streaming_source/trait.StreamingSource.html)はエラーを返す必要があります。
この例において、単純なエラー型を作成しますが、アプリケーションコードでは既存のエラー型を使用できます。

```rust
#[derive(Debug)]
pub enum MyError {
    ExpectedProblem,
    OhNoes,
}
```

クライアントライブラリは、カスタムのエラー型が標準ライブラリの[Error](https://doc.rust-lang.org/std/error/trait.Error.html)トレイトを実装していることを要求します。

```rust
impl std::error::Error for MyError {}
```

御存知の通り、これは[Display](https://doc.rust-lang.org/std/fmt/trait.Display.html)の実装も要求します。

```rust
impl std::fmt::Display for MyError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ExpectedProblem => write!(f, "this kind of thing happens"),
            Self::OhNoes => write!(f, "oh noes! something terrible happened"),
        }
    }
}
```

## カスタムの`StreamingSource`の作成

オブジェクトのデータを生成する型を作成します。
この例では、カウンターを使用して合成データを生成します。

```rust
#[derive(Debug, Default)]
struct MySource(u32);
```

その型に`StreamingSource`を実装します。

```rust
impl google_cloud_storage::streaming_source::StreamingSource for MySource {
    // ... より詳細は後述 ...
}
```

エラー型を定義します。

```rust
type Error = MyError;
```

そして、このトレイトの主要なメソッドを実装します。
この関数が（最終的に）上で定義したエラー型を返す点に注目してください。

```rust
async fn next(&mut self) -> Option<Result<bytes::Bytes, Self::Error>> {
    self.0 += 1;
    match self.0 {
        42 => Some(Err(MyError::ExpectedProblem)),
        n if n > 42 => None,
        n => Some(Ok(bytes::Bytes::from_owner(
          format!("test data for the example {n}\n")
        ))),
    }
}
```

## オブジェクトの作成

[Cloud Storage](https://cloud.google.com/storage)とやり取りするクライアントが必要です。

```rust
pub async fn attempt_upload(bucket_name: &str) -> anyhow::Result<()> {
    use google_cloud_storage::client::Storage;
    let client = Storage::builder().build().await?;
}
```

カスタム型を使用して、オブジェクトを作成します。

```rust
let upload = client
    .write_object(bucket_name, "expect-error", MySource::default())
    .send_buffered()
    .await;
```

予想通り、このオブジェクトの書き込みは失敗します。
エラーの詳細を検証できます。

## 次のステップ

- [オブジェクトを書き込む際にデータをプッシュする](https://googleapis.github.io/google-cloud-rust/storage/queue.html)

## 完全なプログラム

```rust
#[derive(Debug)]
pub enum MyError {
    ExpectedProblem,
    OhNoes,
}

impl std::error::Error for MyError {}

impl std::fmt::Display for MyError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ExpectedProblem => write!(f, "this kind of thing happens"),
            Self::OhNoes => write!(f, "oh noes! something terrible happened"),
        }
    }
}

#[derive(Debug, Default)]
struct MySource(u32);

impl google_cloud_storage::streaming_source::StreamingSource for MySource {
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

pub async fn attempt_upload(bucket_name: &str) -> anyhow::Result<()> {
    use google_cloud_storage::client::Storage;
    let client = Storage::builder().build().await?;
    let upload = client
        .write_object(bucket_name, "expect-error", MySource::default())
        .send_buffered()
        .await;
    println!("Upload result {upload:?}");
    let err = upload.expect_err("the source is supposed to terminate the upload");
    assert!(err.is_serialization(), "{err:?}");
    use std::error::Error as _;
    assert!(err.source().is_some_and(|e| e.is::<MyError>()), "{err:?}");
    Ok(())
}
```
