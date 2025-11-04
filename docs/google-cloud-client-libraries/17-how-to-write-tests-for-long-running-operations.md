# 17. 長時間かかる操作をテストする方法

<https://googleapis.github.io/google-cloud-rust/mocking_lros.html>

Google Cloud Client Libraries for Rustは、長時間かかる操作（以後、LROs）と簡単にやり取りするためのヘルパーがあります。

テストでLROsの振る舞いを模倣することは、これらヘルパーの内部の仕組みを理解する必要があります。
このガイドは、それを実施する方法を紹介します。

## 事前条件

このガイドは、前のチャプターを理解していることを想定しています。

- [長時間かかる操作を実行する](https://googleapis.github.io/google-cloud-rust/working_with_long_running_operations.html)
- [クライアントを使用してテストを記述する方法](https://googleapis.github.io/google-cloud-rust/mock_a_client.html)

## 自動ポーリングのテスト

アプリケーションコードが`lro::Poller::until_done()`で待機しているとします。
前のセクションでは、これを「自動ポーリング」と読んでいました。

```rust
    // 自動ポーリングするアプリケーションの関数の例
    //
    // これはLROを開始して、結果を待ち、それを処理します。
    pub async fn my_automatic_poller(
        client: &speech::client::Speech,
        project_id: &str,
    ) -> Result<Option<wkt::Duration>> {
        use google_cloud_lro::Poller;
        client
            .batch_recognize()
            .set_recognizer(format!(
                "projects/{project_id}/locations/global/recognizers/_"
            ))
            .poller()
            .until_done()
            .await
            .map(|r| r.total_billed_duration)
    }
```

アプリケーションがLROの最終的な結果についてのみ関心があることに注意してください。
LROのポーリングからの中間結果を処理する必要はありません。
テストは、モックからのLROの最終的な結果を返すだけで済みます。

### `longrunning::model::Operation`の作成

次のレスポンスが結果となる呼び出しを期待しているとします。

```rust
    fn expected_duration() -> Option<wkt::Duration> {
        Some(wkt::Duration::clamp(100 0))
    }

    fn expected_response() -> BatchRecognizeResponse {
        BatchRecognizeResponse::new().set_or_clear_total_billed_duration(expected_duration())
    }
```

スタブが`BatchRecognizeResponse`ではなく、`longrunning::model::Operation`を返すことに気付いたかもしれません。
期待しているレスポンスを`Operation::result`にまとめる必要があります。

```rust
    fn make_finished_operation(response: &BatchRecognizeResponse) -> Result<Response<Operation>> {
        let any = wkt::Any::from_msg(response).expect("test message should succeed");
        let operation = Operation::new()
            .set_done(true)
            .set_result(OperationResult::Response(any.into()));
        Ok(Response::from(operation))
    }
```

また、`done`フィールドを`true`に設定していることにも注意してください。
これは、操作が完了したため、`Poller`がポーリングループを終了していることを示しています。

```rust
            .set_done(true)
```

### テストコード

これで、テストを記述する準備ができました。

まず、[speech::stub::Speech](https://docs.rs/google-cloud-speech-v2/latest/google_cloud_speech_v2/stub/trait.Speech.html)トレイトを実装したモッククラスを定義します。

```rust
    mockall::mock! {
        #[derive(Debug)]
        Speech {}

        impl speech::stub::Speech for Speech {
            async fn batch_recognize(&self, req: BatchRecognizeRequest, _options: gax::options::RequestOptions) -> Result<Response<Operation>>;
        }
    }
```

ここで、テスト内にモックを作成し、その予期された振る舞いを設定します。

```rust
        let mut mock = MockSpeech::new();
        mock.expect_batch_recognize()
            .return_once(|_, _| make_finished_operation(&expected_response()));
```

最後に、モックからクライアントを作成し、関数を呼び出し、レスポンスを検査します。

```rust
        // モックによって実装されたクライアントを作成
        let client = speech::client::Speech::from_stub(mock);

        // 自動的にポーリングする関数を呼び出し
        let billed_duration = my_automatic_poller(&client, "my-project").await?;

        // LROの最終的な結果を検査
        assert_eq!(billed_duration, expected_duration());
```

## 中間メタデータと持つ手動ポーリングのテスト

アプリケーションが手動でポーリングし、部分的な更新でいくらかの進捗があったとします。

```rust
    pub struct BatchRecognizeResult {
        pub progress_updates: Vec<i32>,
        pub billed_duration: Result<Option<wkt::Duration>>,
    }

    // 手動でポーリングするアプリケーションの関数の例
    //
    // これはLROを開始します。
    // これは完全または部分的なポーリング結果を統合します。
    //
    // この場合、これは`BatchRecognize`RPC呼び出しです。
    // もし、部分的な更新が得られた場合、`progress_percent`フィールドを抽出します。
    // もじ、最終的な結果が得られた場合、`total_billed_duration`フィールドを抽出します。
    pub async fn my_manual_poller(
        client: &speech::client::Speech,
        project_id: &str,
    ) -> BatchRecognizeResult {
        use google_cloud_lro::{Poller, PollingResult};
        let mut progress_updates = Vec::new();
        let mut poller = client
            .batch_recognize()
            .set_recognizer(format!(
                "projects/{project_id}/locations/global/recognizers/_"
            ))
            .poller();
        while let Some(p) = poller.poll().await {
            match p {
                PollingResult::Completed(r) => {
                    let billed_duration = r.map(|r| r.total_billed_duration);
                    return BatchRecognizeResult {
                        progress_updates,
                        billed_duration,
                    }
                }
                PollingResult::InProgress(m) => {
                    if let Some(metadata) = m {
                      // これは単純なアプリケーションです。
                      // アプリケーションは、操作が完了した後でそれを蓄積する代わりに、
                      // すぐに部分的な更新で何らかのタスクを実行します。
                      progress_updates.push(metadata.progress_percent)
                    }
                }
                PollingResult::PollingError(e) => {
                  return BatchRecognizeResult {
                      progress_updates,
                      billed_duration: Err(e),
                  }
                }
            }
            tokio::time::sleep(std::time::Duration::from_millis(500)).await;
        }

        // `poll`が`None`を返した場合にのみここにたどり着きますが、それは`PollingResult::Completed`
        // を返した後でしか`None`になりません。
        // したがって、次には決して到達しません。
        unreachable!("loop should exit via the `Completed` branch.");
    }
```

アプリケーションが中間のメタデータを受け取ったときにアプリケーションが動作する方法を模倣したいと考えています。
モックから操作が進行中であることを示す値を返すことにより、これを達成できます。

### `longrunning::model::Operation`の作成

`BatchRecognize`RPC呼び出しは、`speech::model::OperationMetadata`の形式の部分的な結果を返します。
前と同様に、これを返される`longrunning::model::Operation`にまとめる必要がありますが、今回は`Operation::metadata`フィールドにまとめます。

```rust
    fn make_partial_operation(progress: i32) -> Result<Response<Operation>> {
        let metadata = OperationMetadata::new().set_progress_percent(progress);
        let any = wkt::Any::from_msg(&metadata).expect("test messages should succeed");
        let operation = Operation::new().set_metadata(any);
        Ok(Response::from(operation))
    }
```

### テストコード

まず、[speech::stub::Speech](https://docs.rs/google-cloud-speech-v2/latest/google_cloud_speech_v2/stub/trait.Speech.html)トレイトを実装するモッククラスを定義します。
`get_operation()`をオーバーライドしていることに注意してください。
その理由を短く説明します。

```rust
    mockall::mock! {
        #[derive(Debug)]
        Speech {}

        impl speech::stub::Speech for Speech {
            async fn batch_recognize(&self, req: BatchRecognizeRequest, _options: gax::options::RequestOptions) -> Result<Response<Operation>>;
            async fn get_operation(&self, req: GetOperationRequest, _options: gax::options::RequestOptions) -> Result<Response<Operation>>;
        }
    }
```

では、テスト内にモックを作成し、それに予期する振る舞いを設定します。

```rust
        let mut seq = mockall::Sequence::new();
        let mut mock = MockSpeech::new();
        mock.expect_batch_recognize()
            .once()
            .in_sequence(&mut seq)
            .returning(|_, _| make_partial_operation(25));
        mock.expect_get_operation()
            .once()
            .in_sequence(&mut seq)
            .returning(|_, _| make_partial_operation(50));
        mock.expect_get_operation()
            .once()
            .in_sequence(&mut seq)
            .returning(|_, _| make_partial_operation(75));
        mock.expect_get_operation()
            .once()
            .in_sequence(&mut seq)
            .returning(|_, _| make_finished_operation(&expected_response()));
```

これらの予期された振る舞いは、部分的な結果（25%、50%、75%）を返し、次に望ましい最終結果を返します。

ここで、いくつか注意すべきことがあります。

**1.最初の予期された振る舞いには`batch_recognize()`が設定され、後続の予期された振る舞いには`get_operation()`が設定されます。**

最初の`BatchRecognize`RPC呼び出しは、サーバーサイドでLROを開始します。
サーバーはLROのいくつかの識別子を返します。
これは`name`フィールドで、簡潔さのためにテストコードから省略されています。

それ以後、クライアントライブラリはLROの状態を単純にポーリングします。
それは`GetOperation`RPC呼び出しを使用して行われます。

これが、最初のレスポンスと後続のすべてのレスポンスに対して、異なるRPC呼び出しの予期された振る舞いを設定した理由です。

**2.予期された振る舞いは[シーケンス](https://docs.rs/mockall/latest/mockall/struct.Sequence.html)に設定されます。**

これは`mockall`が呼び出しの順番を検査できるようにします。
それは、どの`expected_get_operation`が一致するか決定する必要もあります。

最後に、モックからクライアントを作成し、関数を呼び出し、レスポンスを検査します。

```rust
        // モックによって実装されたクライアントを作成
        let client = speech::client::Speech::from_stub(mock);

        // 手動でポーリングする関数を呼び出し
        let result = my_manual_poller(&client, "my-project").await;

        // 部分的なメタデータの更新と最終的な結果を検査
        assert_eq!(result.progress_updates, [25, 50, 75]);
        assert_eq!(result.billed_duration?, expected_duration());
```

## エラーの模倣

エラーはLROのいくつかの場所から発生します。

アプリケーションが自動ポーリングを使用している場合、次の場面はすべて同等です。
`until_done()`は、エラーの発生箇所にかかわらず、`Result`でエラーを返します。
[LROの開始でエラーを模倣する](https://googleapis.github.io/google-cloud-rust/mocking_lros.html#simulating-an-error-starting-an-lro)は最も単純なテストを生み出します。

スタブ化されたクライアントは再試行またはポーリングポリシーを持ちません。
このすべての場面で、ポーリングループは、エラーが通常一時的であると考えられても、最初のエラーで中止されます。

### LROの開始でエラーを模倣する

エラーを模倣する最も単純な方法は、エラーで最初のリクエストを失敗させることです。

```rust
        mock.expect_batch_recognize().return_once(|_, _| {
            use gax::error::Error;
            use gax::error::rpc::{Code, Status};
            let status = Status::default()
                .set_code(Code::Aborted)
                .set_message("Resource exhausted");
            Err(Error::service(status))
        });
```

手動ポーリング用に、LROの開始エラーは、完了ブランチを介して返されます。
これはポーリングループを終了させます。

```rust
                PollingResult::Completed(r) => {
```

### LROがエラーとなる結果を模倣する

LROがエラーとなる結果を模倣する必要がある場合、中間メタデータが返された後で、最終的な`longrunning::model::Operation`内でエラーを返す必要があります。

```rust
    fn make_failed_operation(status: rpc::model::Status) -> Result<Response<Operation>> {
        let operation = Operation::new()
            .set_done(true)
            .set_result(OperationResult::Error(status.into()));
        Ok(Response::from(operation))
    }
```

前に`get_operation`から`Operation`を返す予期された振る舞いを設定しました。

```rust
        mock.expect_get_operation()
            .once()
            .in_sequence(&mut seq)
            .returning(|_, _| {
                // これは`Create*`RPC呼び出しの一般的なエラーで、よくLROで発生します。
                // 実際には、`BatchRecognize`には適用されません。
                let status = rpc::model::Status::default()
                    .set_code(gax::error::rpc::Code::AlreadyExists as i32)
                    .set_message("resource already exists");
                make_failed_operation(status)
            });
```

LROがエラーで終了することは、完了ブランチを介して返されます。
これはポーリングループを終了させます。

```rust
            PollingResult::Completed(r) => {
```

### ポーリングエラーの模倣

また、ポーリングループは、ポーリングポリシーが使い果たされると終了します。
これが発生したとき、クライアントライブラリはLROが完了したかどうかを絶対的に判断できません。

アプリケーションがこのケースを扱う独自のロジックを持っている場合、`get_operation`の予期された振る舞いからエラーを返すことで試すことができます。

```rust
        mock.expect_get_operation()
            .once()
            .is_sequence(&mut seq)
            .returning(|_, _| {
                use gax::error::Error;
                use gax::error::rpc::{Code, Status};
                let status = Status::default()
                    .set_code(Code::Aborted)
                    .set_message("operation was aborted");
                Err(Error::service(status))
            });
```

ポーリングエラーでLROが終了することは、ポーリングエラーブランチを返して返されます。

```rust
            PollingResult::PollingError(e) => {
```

---

## 自動ポーリング: テストの全体

```rust
use gax::Result;
use gax::response::Response;
use google_cloud_gax as gax;
use google_cloud_longrunning as longrunning;
use google_cloud_speech_v2 as speech;
use google_cloud_wkt as wkt;
use longrunning::model::Operation;
use longrunning::model::operation::Result as OperationResult;
use speech::model::{BatchRecognizeRequest, BatchRecognizeResponse};

// テストに基づいたアプリケーションコードの例
mod my_application {
    use super::*;

    // 自動でポーリングするアプリケーションの関数の例
    //
    // これはLROを開始して、結果を待ち、それを処理します。
    pub async fn my_automatic_poller(
        client: &speech::client::Speech,
        project_id: &str,
    ) -> Result<Option<wkt::Duration>> {
        use google_cloud_lro::Poller;
        client
            .batch_recognize()
            .set_recognizer(format!(
                "projects/{project_id}/locations/global/recognizers/_"
            ))
            .poller()
            .until_done()
            .await
            .map(|r| r.total_billed_duration)
    }
}

#[cfg(test)]
mod tests {
    use super::my_application::*;
    use super::*;

    mockall::mock! {
        #[derive(Debug)]
        Speech {}
        impl speech::stub::Speech for Speech {
            async fn batch_recognize(&self, req: BatchRecognizeRequest, _options: gax::options::RequestOptions) -> Result<Response<Operation>>;
        }
    }

    fn expected_duration() -> Option<wkt::Duration> {
        Some(wkt::Duration::clamp(100, 0))
    }

    fn expected_response() -> BatchRecognizeResponse {
        BatchRecognizeResponse::new().set_or_clear_total_billed_duration(expected_duration())
    }

    fn make_finished_operation(response: &BatchRecognizeResponse) -> Result<Response<Operation>> {
        let any = wkt::Any::from_msg(response).expect("test message should succeed");
        let operation = Operation::new()
            .set_done(true)
            .set_result(OperationResult::Response(any.into()));
        Ok(Response::from(operation))
    }

    #[tokio::test]
    async fn automatic_polling() -> Result<()> {
        // モックを作成し、それに予期された振る舞いを設定
        let mut mock = MockSpeech::new();
        mock.expect_batch_recognize()
            .return_once(|_, _| make_finished_operation(&expected_response()));

        // モックによって実装されたクライアントを作成
        let client = speech::client::Speech::from_stub(mock);

        // 自動的にポーリングする関数を呼び出し
        let billed_duration = my_automatic_poller(&client, "my-project").await?;

        // LROの最終的な結果を検査
        assert_eq!(billed_duration, expected_duration());

        Ok(())
    }
}
```

## 中間のメタデータを持つ手動によるポーリング: テストの全体

```rust
use gax::Result;
use gax::response::Response;
use google_cloud_gax as gax;
use google_cloud_longrunning as longrunning;
use google_cloud_speech_v2 as speech;
use google_cloud_wkt as wkt;
use longrunning::model::operation::Result as OperationResult;
use longrunning::model::{GetOperationRequest, Operation};
use speech::model::{BatchRecognizeRequest, BatchRecognizeResponse, OperationMetadata};

// テストに基づいたアプリケーションコードの例
mod my_application {
    use super::*;

    pub struct BatchRecognizeResult {
        pub progress_updates: Vec<i32>,
        pub billed_duration: Result<Option<wkt::Duration>>,
    }

    // 手動でポーリングするアプリケーションの関数の例
    //
    // これはLROを開始します。
    // これは、完全または部分的なポーリング結果を統合します。
    //
    // この場合、これは`BatchRecognize`RPC呼び出しです。
    // 部分的な更新を得た場合、`progress_percent`フィールドを抽出します。
    // 最終的な結果を得た場合、`total_billed_duration`フィールドを抽出します。
    pub async fn my_manual_poller(
        client: &speech::client::Speech,
        project_id: &str,
    ) -> BatchRecognizeResult {
        use google_cloud_lro::{Poller, PollingResult};
        let mut progress_updates = Vec::new();
        let mut poller = client
            .batch_recognize()
            .set_recognizer(format!(
                "projects/{project_id}/locations/global/recognizers/_"
            ))
            .poller();
        while let Some(p) = poller.poll().await {
            match p {
                PollingResult::Completed(r) => {
                    let billed_duration = r.map(|r| r.total_billed_duration);
                    return BatchRecognizeResult {
                        progress_updates,
                        billed_duration,
                    };
                }
                PollingResult::InProgress(m) => {
                    if let Some(metadata) = m {
                        // これは不合理なアプリケーションです。
                        // アプリケーションは操作が完了した後でそれを蓄積する代わりに、すぐに部分的更新を持つ何らかのタスクを実行します。
                        progress_updates.push(metadata.progress_percent);
                    }
                }
                PollingResult::PollingError(e) => {
                    return BatchRecognizeResult {
                        progress_updates,
                        billed_duration: Err(e),
                    };
                }
            }
            tokio::time::sleep(std::time::Duration::from_millis(500)).await;
        }

        // We can only get here if `poll()` returns `None`, but it only returns
        // `None` after it returned `PollingResult::Completed`. Therefore this
        // is never reached.

        unreachable!("loop should exit via the `Completed` branch.");
    }
}

#[cfg(test)]
mod tests {
    use super::my_application::*;
    use super::*;

    mockall::mock! {
        #[derive(Debug)]
        Speech {}
        impl speech::stub::Speech for Speech {
            async fn batch_recognize(&self, req: BatchRecognizeRequest, _options: gax::options::RequestOptions) -> Result<Response<Operation>>;
            async fn get_operation(&self, req: GetOperationRequest, _options: gax::options::RequestOptions) -> Result<Response<Operation>>;
        }
    }

    fn expected_duration() -> Option<wkt::Duration> {
        Some(wkt::Duration::clamp(100, 0))
    }

    fn expected_response() -> BatchRecognizeResponse {
        BatchRecognizeResponse::new().set_or_clear_total_billed_duration(expected_duration())
    }

    fn make_finished_operation(
        response: &BatchRecognizeResponse,
    ) -> Result<gax::response::Response<Operation>> {
        let any = wkt::Any::from_msg(response).expect("test message should succeed");
        let operation = Operation::new()
            .set_done(true)
            .set_result(OperationResult::Response(any.into()));
        Ok(Response::from(operation))
    }

    fn make_partial_operation(progress: i32) -> Result<Response<Operation>> {
        let metadata = OperationMetadata::new().set_progress_percent(progress);
        let any = wkt::Any::from_msg(&metadata).expect("test message should succeed");
        let operation = Operation::new().set_metadata(any);
        Ok(Response::from(operation))
    }

    #[tokio::test]
    async fn manual_polling_with_metadata() -> Result<()> {
        let mut seq = mockall::Sequence::new();
        let mut mock = MockSpeech::new();
        mock.expect_batch_recognize()
            .once()
            .in_sequence(&mut seq)
            .returning(|_, _| make_partial_operation(25));
        mock.expect_get_operation()
            .once()
            .in_sequence(&mut seq)
            .returning(|_, _| make_partial_operation(50));
        mock.expect_get_operation()
            .once()
            .in_sequence(&mut seq)
            .returning(|_, _| make_partial_operation(75));
        mock.expect_get_operation()
            .once()
            .in_sequence(&mut seq)
            .returning(|_, _| make_finished_operation(&expected_response()));

        // Create a client, implemented by our mock.
        let client = speech::client::Speech::from_stub(mock);

        // Call our function which manually polls.
        let result = my_manual_poller(&client, "my-project").await;

        // Verify the partial metadata updates, and the final result.
        assert_eq!(result.progress_updates, [25, 50, 75]);
        assert_eq!(result.billed_duration?, expected_duration());

        Ok(())
    }
}
```

## エラーの模倣: テストの全体

```rust
use gax::Result;
use gax::response::Response;
use google_cloud_gax as gax;
use google_cloud_longrunning as longrunning;
use google_cloud_rpc as rpc;
use google_cloud_speech_v2 as speech;
use google_cloud_wkt as wkt;
use longrunning::model::operation::Result as OperationResult;
use longrunning::model::{GetOperationRequest, Operation};
use speech::model::{BatchRecognizeRequest, OperationMetadata};

// Example application code that is under test
mod my_application {
    use super::*;

    pub struct BatchRecognizeResult {
        pub progress_updates: Vec<i32>,
        pub billed_duration: Result<Option<wkt::Duration>>,
    }

    // An example application function that manually polls.
    //
    // It starts an LRO. It consolidates the polling results, whether full or
    // partial.
    //
    // In this case, it is the `BatchRecognize` RPC. If we get a partial update,
    // we extract the `progress_percent` field. If we get a final result, we
    // extract the `total_billed_duration` field.
    pub async fn my_manual_poller(
        client: &speech::client::Speech,
        project_id: &str,
    ) -> BatchRecognizeResult {
        use google_cloud_lro::{Poller, PollingResult};
        let mut progress_updates = Vec::new();
        let mut poller = client
            .batch_recognize()
            .set_recognizer(format!(
                "projects/{project_id}/locations/global/recognizers/_"
            ))
            .poller();
        while let Some(p) = poller.poll().await {
            match p {
                PollingResult::Completed(r) => {
                    let billed_duration = r.map(|r| r.total_billed_duration);
                    return BatchRecognizeResult {
                        progress_updates,
                        billed_duration,
                    };
                }
                PollingResult::InProgress(m) => {
                    if let Some(metadata) = m {
                        // This is a silly application. Your application likely
                        // performs some task immediately with the partial
                        // update, instead of storing it for after the operation
                        // has completed.
                        progress_updates.push(metadata.progress_percent);
                    }
                }
                PollingResult::PollingError(e) => {
                    return BatchRecognizeResult {
                        progress_updates,
                        billed_duration: Err(e),
                    };
                }
            }
            tokio::time::sleep(std::time::Duration::from_millis(500)).await;
        }

        // `poll()`が`None`を返した場合のみここに到達できますが、それは`PollingResult::Completed`
        // を返した後でしか`None`を返しません。
        // したがって、ここには決して到達しません。
        unreachable!("loop should exit via the `Completed` branch.");
    }
}

#[cfg(test)]
mod tests {
    use super::my_application::*;
    use super::*;

    mockall::mock! {
        #[derive(Debug)]
        Speech {}
        impl speech::stub::Speech for Speech {
            async fn batch_recognize(&self, req: BatchRecognizeRequest, _options: gax::options::RequestOptions) -> Result<Response<Operation>>;
            async fn get_operation(&self, req: GetOperationRequest, _options: gax::options::RequestOptions) -> Result<Response<Operation>>;
        }
    }

    fn make_partial_operation(progress: i32) -> Result<Response<Operation>> {
        let metadata = OperationMetadata::new().set_progress_percent(progress);
        let any = wkt::Any::from_msg(&metadata).expect("test message should succeed");
        let operation = Operation::new().set_metadata(any);
        Ok(Response::from(operation))
    }

    fn make_failed_operation(status: rpc::model::Status) -> Result<Response<Operation>> {
        let operation = Operation::new()
            .set_done(true)
            .set_result(OperationResult::Error(status.into()));
        Ok(Response::from(operation))
    }

    #[tokio::test]
    async fn error_starting_lro() -> Result<()> {
        let mut mock = MockSpeech::new();
        mock.expect_batch_recognize().return_once(|_, _| {
            use gax::error::Error;
            use gax::error::rpc::{Code, Status};
            let status = Status::default()
                .set_code(Code::Aborted)
                .set_message("Resource exhausted");
            Err(Error::service(status))
        });

        // モックによって実装されたクライアントを作成
        let client = speech::client::Speech::from_stub(mock);

        // 手動でポーリングする関数を呼び出し
        let result = my_manual_poller(&client, "my-project").await;

        // 最終的な結果を検証
        assert!(result.billed_duration.is_err());

        Ok(())
    }

    #[tokio::test]
    async fn lro_ending_in_error() -> Result<()> {
        let mut seq = mockall::Sequence::new();
        let mut mock = MockSpeech::new();
        mock.expect_batch_recognize()
            .once()
            .in_sequence(&mut seq)
            .returning(|_, _| make_partial_operation(25));
        mock.expect_get_operation()
            .once()
            .in_sequence(&mut seq)
            .returning(|_, _| make_partial_operation(50));
        mock.expect_get_operation()
            .once()
            .in_sequence(&mut seq)
            .returning(|_, _| make_partial_operation(75));
        mock.expect_get_operation()
            .once()
            .in_sequence(&mut seq)
            .returning(|_, _| {
                // これは`Create*`RPC呼び出しの一般的なエラーで、よくLROで発生します。
                // 実際に、それは`BatchRecognize`に適用されることは少ないです。
                let status = rpc::model::Status::default()
                    .set_code(gax::error::rpc::Code::AlreadyExists as i32)
                    .set_message("resource already exists");
                make_failed_operation(status)
            });

        // モックによって実装されたクライアントを作成
        let client = speech::client::Speech::from_stub(mock);

        // 手動でポーリングする関数を呼び出し
        let result = my_manual_poller(&client, "my-project").await;

        // 部分的なメタデータ更新と最終的な結果を検査
        assert_eq!(result.progress_updates, [25, 50, 75]);
        assert!(result.billed_duration.is_err());

        Ok(())
    }

    #[tokio::test]
    async fn polling_loop_error() -> Result<()> {
        let mut seq = mockall::Sequence::new();
        let mut mock = MockSpeech::new();
        mock.expect_batch_recognize()
            .once()
            .in_sequence(&mut seq)
            .returning(|_, _| make_partial_operation(25));
        mock.expect_get_operation()
            .once()
            .in_sequence(&mut seq)
            .returning(|_, _| {
                use gax::error::Error;
                use gax::error::rpc::{Code, Status};
                let status = Status::default()
                    .set_code(Code::Aborted)
                    .set_message("Operation was aborted");
                Err(Error::service(status))
            });

        // モックによって実装されたクライアントを作成
        let client = speech::client::Speech::from_stub(mock);

        // 手動でポーリングする関数を呼び出し
        let result = my_manual_poller(&client, "my-project").await;

        // 部分的なメタデータの更新と最終的な結果を検査
        assert_eq!(result.progress_updates, [25]);
        assert!(result.billed_duration.is_err());

        Ok(())
    }
}
```
