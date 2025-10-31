use gcs::client::Storage;
use gcs::model::Object;
use google_cloud_storage as gcs;

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
        count += buffer.into_iter().filter(|c| *c == b'\n').count();
    }
    Ok(count)
}

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

fn main() {}

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

            async fn write_object_unbuffered<P: StreamingSource + Seek + Send + Sync+'static>(
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
            contents.push_str(&format!("{i}\n"));
        }
        ReadObjectResponse::from_source(ObjectHighlights::default(), bytes::Bytes::from(contents))
    }

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

    #[tokio::test]
    async fn test_upload() -> anyhow::Result<()> {
        let mut mock = MockStorage::new();
        mock.expect_write_object_unbuffered().return_once(
            |_payload: Payload<BytesSource>, r, _| {
                // リクエストの内容を検証
                assert_eq!(r.spec.if_generation_match, Some(42));
                let o = r.spec.resource.unwrap_or_default();
                assert_eq!(o.bucket, "projects/_/buckets/my-bucket");
                assert_eq!(o.name, "my-object");
                // オブジェクトを返す
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
