# 6.5 ストレージクライアントを使用してテストを記述する方法

<https://googleapis.github.io/google-cloud-rust/storage/mocking.html>

Google Cloud Client Libraries for Rustは、本物のクライアント実装のスタブを作成する方法を提供しているため、テストにモックを注入できます。

アプリケーションは、モックを使用して、ネットワーク呼び出しを伴わず、課金も発生しない、制御された信頼性の高い単体テストを作成できます。

このガイドでは、次を学びます。

- `Storage`クライアントを作成したテスト可能なインターフェースを記述する方法
- モックが読み込みする方法
- モックが書き込みする方法
- `Storage`クライアントの設計が、他のGoogle Cloudクライアントの設計と異なっている理由

このガイドは、`Storage`クライアントのモックに特化しています。
一般的なモックガイド（`StorageControl`クライアントに適用する）は、[クライアントを使用してテストを記述する方法](https://googleapis.github.io/google-cloud-rust/mock_a_client.html)を参照してください。

## テスト可能なインターフェイス

コードをテストする必要がないアプリケーションは、すべてのインターフェイスを`Storage`の観点で単純に記述するだけです。
デフォルトの`T`はクライアントの実際の実装となります。

```rust
pub async fn my_function(_client: Storage) {}
```

コードをテストする必要があるアプリケーションは、適切な制約を持つジェネリック`T`の観点でインターフェイスを記述するべきです。

```rust
pub async fn my_testable_function<T>(_client: Storage<T>)
where
    T: gcs::stub::Storage + 'static,
{
}
```

## 読み込みのモック

このガイドのこのセクションは、`read_object`リクエストをモックする方法を紹介します。

オブジェクトをダウンロードし、オブジェクトが含む改行の数をカウントするアプリケーションの関数があるとします。

```rust
// GCSからオブジェクトをダウンロードして、合計行数をカウント
pub async fn count_newlines<T>(
    client: &Storage<T>,
    bucket_id: &str,
    object_id: &str,
) -> gcs::Result<usize>
where
    T: gcs::stub::Storage + 'static,
{
    let mut count = 0;
    let mut reader = client
        .read_object(format!("projects/_/buckets/{bucket_id}"), object_id)
        .set_generation(42)
        .send()
        .await?;
    while let Some(buffer) = reader.next().await.transpose()? {
        count += buffer.into_iter().filter(|c| *c ==b'\n').count();
    }
    Ok(count)
}
```

サーバーからの既知なレスポンスに対してコードをテストしたいと考えています。
`ReadObjectResponse`を偽造することで、これができます。

`ReadObjectResponse`は、基本的にバイトストリームです。
テストでは、`ReadObjectResponse::from_source`にペイロードを提供することにより、擬似的な`ReadObjectResponse`を作成することができます。
ライブラリは、`Storage::write_object`と同じペイロードの型を受け入れます。

```rust
fn fake_response(size: usize) -> ReadObjectResponse {
    let mut contents = String::new();
    for i in 0..size {
        contents.push_str(&format!("{i}\n"));
    }
    ReadObjectResponse::from_source(ObjectHighlights::default(), bytes::Bytes::from(contents))
}
```

擬似的なレスポンスを返すために、クライアントをモックする必要があります。

このガイドは、`mockall`クレートを作成してモックを作成します。
テストでは異なるモックフレームワークを使用できます。

```rust
mockall::mock! {
    #[derive(Debug)]
    Storage {}

    impl gcs::stub::Storage for Storage {
        async fn read_object(&self, _req: ReadObjectRequest, _options: RequestOptions) -> Result<ReadObjectResponse>;

        async fn write_object_buffered<P: StreamingSource + Send + Sync + 'static>(
            &self,
            _payload: P,
            _req: WriteObjectRequest,
            _options: RequestOptions,
        ) -> Result<Object>;

        async fn write_object_unbuffered<P: StreamingSource + Seek + Send + Sync + 'static>(
            &self,
            _payload: P,
            _req: WriteObjectRequest,
            _options: RequestOptions,
        ) -> Result<Object>;
    }
}
```

これで、`count_newlines`関数を内部で呼び出すユニットテストを記述する準備ができました。

```rust
#[tokio::test]
async fn test_count_lines() -> anyhow::Result<()> {
    let mut mock = MockStorage::new();
    mock.expect_read_object().return_once({
        move |r, _| {
            // リクエストの内容を検証
            assert_eq!(r.generation, 42);
            assert_eq!(r.bucket, "projects/_/buckets/my-bucket");
            assert_eq!(r.object, "my-object");
            // `ReadObjectResponse`を返す
            Ok(fake_response(100))
        }
    });
    let client = gcs::client::Storage::from_stub(mock);

    let count = count_newlines(&client, "my-bucket", "my-object").await?;
    assert_eq!(count, 100);

    Ok(())
}
```

## 書き込みのモック

このガイドのこのセクションは、`write_object`リクエストをモックする方法を紹介します。

メモリからオブジェクトをアップロードするアプリケーションの関数があるとします。

```rust
// GCSにオブジェクトをアップロード
pub async fn upload<T>(client: &Storage<T>, bucket_id: &str, object_id: &str) -> gcs::Result<Object>
where
    T: gcs::stub::Storage + 'static,
{
    client
        .write_object(
            format!("projects/_/buckets/{bucket_id}"),
            object_id,
            "payload",
        )
        .set_if_generation_match(42)
        .send_unbuffered()
        .await
}
```

この関数をテストするために、クライアントをモックする必要があります。

このガイドは、`mockall`クレートを使用して、モックを作成します。
テストでは異なるモックフレームワークを使用できます。

```rust
mockall::mock! {
    #[derive(Debug)]
    Storage {}

    impl gcs::stub::Storage for Storage {
        async fn read_object(&self, _req: ReadObjectRequest, _options: RequestOptions) -> Result<ReadObjectResponse>;

        async fn write_object_buffered<P: StreamingSource + Send + Sync + 'static>(
            &self,
            _payload: P,
            _req: WriteObjectRequest,
            _options: RequestOptions,
        ) -> Result<Object>;

        async fn write_object_unbuffered<P: StreamingSource + Seek + Send + Sync + 'static>(
            &self,
            _payload: P,
            _req: WriteObjectRequest,
            _options: RequestOptions,
        ) -> Result<Object>;
    }
}
```

これで、`upload`関数を内部で呼び出すユニットテストを記述する準備ができました。

```rust
#[tokio::test]
async fn test_upload() -> anyhow::Result<()> {
    let mut mock = MockStorage::new();
    mock.expect_write_object_unbuffered()
        .return_once(
            |_payload: Payload<BytesSource>, r, _| {
                // リクエストの内容を検証
                assert_eq!(r.spec.if_generation_match, Some(42));
                let o = r.spec.resource.unwrap_or_default();
                assert_eq!(o.bucket, "projects/_/buckets/my-bucket");
                assert_eq!(o.name, "my-object");

                // オブジェクトを返す
                Ok(Object::default()
                    .set_bucket("projects/_/buckets/my-buckets")
                    .set_name("my-object")
                    .set_generation(42))
            },
        );
    let client = gcs::client::Storage::from_stub(mock);

    let object = upload(&client, "my-bucket", "my-object").await?;
    assert_eq!(object.generation, 42);

    Ok(())
}
```

### 詳細

関数が`send_unbuffered()`を呼び出すため、対応する`write_object_unbuffered()`を使用しなければなりません。

```rust
mock.expect_write_object_unbuffered()
```

`mockall::mock!`内のジェネリックは、異なる関数として扱われます。
正確なペイロードの型を提供しなければならないため、コンパイラーは使用する関数を知ることができます。

```rust
|_payload: Payload<BytesSource>, r, _| {
```

## 設計根拠

### 他のクライアント

`StorageControl`のようなほとんどのクライアントは、内部的にスタブトレイトをボックス化した`dyn`互換実装を保持します。
それらは動的ディスパッチを使用して、クライアントからスタブ（本物の実装またはモックになり得る）にリクエストを送信します。

これらのクライアントは動的ディスパッチを使用するため、スタブの正確な型をコンパイラーが知っている必要はありません。
クライアントは、スタブ型に対してジェネリックである必要もありません。

### ストレージクライアント

`dyn`互換トレイトを持つために、すべての型のサイズは知られていなければなりません。

`Storage`クライアントはそのインターフェイス内に複雑な型を持ちます。

- `write_object`はジェネリックなペイロードを受け入れます。
- `read_object`はストリームのようなものを返します。

したがって、もし`Storage`クライアントに対して同じ動的ディスパッチ手法を使用したい場合、すべてのジェネリック／トレイト`impl`を最終的にボックス化しなければなりません。
それぞれのボックスは追加のヒープ割り当てであり、さらに動的ディスパッチを伴います。

可能な限り`Storage`クライアントの性能を高めたいため、`dyn`互換のスタブトレイトの具体的な実装でクライアントをテンプレート化することが望ましいと判断しました。

---

## 完全なアプリケーションコードとテストスイート

```rust
use gcs::client::Storage;
use gcs::model::Object;
use google_cloud_storage as gcs;

// Downloads an object from GCS and counts the total lines.
pub async fn count_newlines<T>(
    client: &Storage<T>,
    bucket_id: &str,
    object_id: &str,
) -> gcs::Result<usize>
where
    T: gcs::stub::Storage + 'static,
{
    let mut count = 0;
    let mut reader = client
        .read_object(format!("projects/_/buckets/{bucket_id}"), object_id)
        .set_generation(42)
        .send()
        .await?;
    while let Some(buffer) = reader.next().await.transpose()? {
        count += buffer.into_iter().filter(|c| *c == b'\n').count();
    }
    Ok(count)
}

// Uploads an object to GCS.
pub async fn upload<T>(client: &Storage<T>, bucket_id: &str, object_id: &str) -> gcs::Result<Object>
where
    T: gcs::stub::Storage + 'static,
{
    client
        .write_object(
            format!("projects/_/buckets/{bucket_id}"),
            object_id,
            "payload",
        )
        .set_if_generation_match(42)
        .send_unbuffered()
        .await
}

#[cfg(test)]
mod tests {
    use super::{count_newlines, upload};
    use gcs::Result;
    use gcs::model::{Object, ReadObjectRequest};
    use gcs::model_ext::{ObjectHighlights, WriteObjectRequest};
    use gcs::read_object::ReadObjectResponse;
    use gcs::request_options::RequestOptions;
    use gcs::streaming_source::{BytesSource, Payload, Seek, StreamingSource};
    use google_cloud_storage as gcs;

    mockall::mock! {
        #[derive(Debug)]
        Storage {}
        impl gcs::stub::Storage for Storage {
            async fn read_object(&self, _req: ReadObjectRequest, _options: RequestOptions) -> Result<ReadObjectResponse>;
            async fn write_object_buffered<P: StreamingSource + Send + Sync + 'static>(
                &self,
                _payload: P,
                _req: WriteObjectRequest,
                _options: RequestOptions,
            ) -> Result<Object>;
            async fn write_object_unbuffered<P: StreamingSource + Seek + Send + Sync + 'static>(
                &self,
                _payload: P,
                _req: WriteObjectRequest,
                _options: RequestOptions,
            ) -> Result<Object>;
        }
    }

    fn fake_response(size: usize) -> ReadObjectResponse {
        let mut contents = String::new();
        for i in 0..size {
            contents.push_str(&format!("{i}\n"))
        }
        ReadObjectResponse::from_source(ObjectHighlights::default(), bytes::Bytes::from(contents))
    }

    #[tokio::test]
    async fn test_count_lines() -> anyhow::Result<()> {
        let mut mock = MockStorage::new();
        mock.expect_read_object().return_once({
            move |r, _| {
                // Verify contents of the request
                assert_eq!(r.generation, 42);
                assert_eq!(r.bucket, "projects/_/buckets/my-bucket");
                assert_eq!(r.object, "my-object");

                // Return a `ReadObjectResponse`
                Ok(fake_response(100))
            }
        });
        let client = gcs::client::Storage::from_stub(mock);

        let count = count_newlines(&client, "my-bucket", "my-object").await?;
        assert_eq!(count, 100);

        Ok(())
    }

    #[tokio::test]
    async fn test_upload() -> anyhow::Result<()> {
        let mut mock = MockStorage::new();
        mock.expect_write_object_unbuffered()
            .return_once(
                |_payload: Payload<BytesSource>, r, _| {
                    // Verify contents of the request
                    assert_eq!(r.spec.if_generation_match, Some(42));
                    let o = r.spec.resource.unwrap_or_default();
                    assert_eq!(o.bucket, "projects/_/buckets/my-bucket");
                    assert_eq!(o.name, "my-object");

                    // Return the object
                    Ok(Object::default()
                        .set_bucket("projects/_/buckets/my-bucket")
                        .set_name("my-object")
                        .set_generation(42))
                },
            );
        let client = gcs::client::Storage::from_stub(mock);

        let object = upload(&client, "my-bucket", "my-object").await?;
        assert_eq!(object.generation, 42);

        Ok(())
    }
}
```
