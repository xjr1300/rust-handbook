# 16. クライアントを使用してテストを記述する方法

<https://googleapis.github.io/google-cloud-rust/mock_a_client.html>

Google Cloud Client Libraries for Rustは、実際のクライアントの実装をスタブする方法を提供しており、テストでモックを注入できます。

アプリケーションは、ネットワーク呼び出しがなく、支払いが発生しない、制御され、信頼できるユニットテストを記述するためにモックを使用できます。

このガイドは、その方法を紹介します。

## 依存関係

Rustにはいくつかの[モックフレームワーク](https://blog.logrocket.com/mocking-rust-mockall-alternatives/)が存在します。
このガイドは、最も一般的であると考えられる[mockall](https://docs.rs/mockall/latest/mockall/)を使用します。

```sh
cargo add --dev mockall
```

このガイドは[Speech](https://docs.rs/google-cloud-speech-v2/latest/google_cloud_speech_v2/client/struct.Speech.html)クライアントを使用します。
このガイドの同様な考え方は、`Speech`クライアントだけでなく、他のすべてのクライアントに適用できます。

`Cargo.toml`に依存関係を宣言します。
あなたのプロジェクトでも同様ですが、カスタム`path`は必要ありません。

```sh
cargo add google-cloud-speech-v2 google-cloud-lro
```

## クライアントをモックする

まず、コードを簡潔にするために、いくつかの`use`宣言をします。

```rust
use google_cloud_gax as gax;
use google_cloud_speech_v2 as speech;
```

アプリケーションが、RPCを呼び出し、サーバーからのレスポンスを処理する`Speech`クライアントを使用する関数があると想定しましょう。

```rust
// アプリケーションの関数例
//
// これはRPCを呼び出し、いくつかのフィールドを設定します。
// この場合、それは`GetRecognizer`RPC呼び出しで、`name`フィールドを設定しています。
//
// そして、サーバーからのレスポンスを処理します。
// この場合、レコグナイザーの表示名を抽出します。
async fn my_application_function(client: &speech::client::Speech) -> gax::Result<String> {
    client
        .get_recognizer()
        .set_name("invalid-test-recognizer")
        .send()
        .await
        .map(|r| r.display_name)
}
```

サーバからのさまざまなレスポンスを処理するコードをテストしたいと考えています。

まず、モッククラスを定義します。
このクラスは、[speech::stub::Speech](https://docs.rs/google-cloud-speech-v2/latest/google_cloud_speech_v2/stub/trait.Speech.html)トレイトを実装しています。

```rust
    mockall:mock! {
        #[derive(Debug)]
        Speech {}

        impl speech::stub::Speech for Speech {
            async fn get_recognizer(
                &self,
                req: speech::model::GetRecognizerRequest,
                _options: gax::options::RequestOptions
            ) -> gax::Result<gax::Response<speech::model::Recognizer>>;
        }
    }
```

次に、モックのインスタンスを作成します。
[mockall::mock!](https://docs.rs/mockall/latest/mockall/macro.mock.html)マクロは、上記構造体の名前に`Mock`接頭辞を付加することに注意してください。

```rust
        let mut mock = MockSpeech::new();
```

次に、モックに予期する動作を設定します。
特定の名前で`GetRecognizer`が呼び出されることを想定します。

それが発生した場合、サーバーからの成功レスポンスを模倣します。

```rust
        mock.expect_get_recognizer()
            .withf(move |r, _|
                // オプションとして、リクエスト内のフィールドを検査
                r.name == "invalid-test-recognizer")
            .return_once(|_, _| {
                Ok(gax::response::Response::from(
                    speech::model::Recognizer::new().set_display_name("test-display-name"),
                ))
            });
```

これで、モックを使用した`Speech`クライアントを作成する準備ができました。

```rust
        let client = speech::client::Speech::from_stub(mock);
```

最後に、関数を呼び出す準備をします。

```rust
        let display_name = my_application_function(&client).await?;
```

そして、結果を検査します。

```rust
        assert_eq!(display_name, "test-display-name");
```

## エラーを模倣する

エラーを模倣することは、成功を模倣することと違いはありません。
モックによって返される結果を修正する必要があるだけです。

```rust
        mock.expect_get_recognizer().return_once(|_, _| {
            // 今度は、エラーを返します。
            use gax::error::Error;
            use gax::error::rpc::{Code, Status};
            let status = Status::default()
                .set_code(Code::NotFound)
                .set_message("Resource not found");
            Err(Err::service(status))
        })
```

`from_stub()`で構築されたクライアントは、内部的な再試行ループを持たないことに注意してください。
これは、スタブからのすべてのエラーをアプリケーションに直接返します。

---

## 完全なプログラム

すべてのコードをまとめた完全なプログラムは、次のようになります。

```rust
#[cfg(test)]
mod tests {
    use google_cloud_gax as gax;
    use google_cloud_speech_v2 as speech;
    type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

    // An example application function.
    //
    // It makes an RPC, setting some field. In this case, it is the `GetRecognizer`
    // RPC, setting the name field.
    //
    // It processes the response from the server. In this case, it extracts the
    // display name of the recognizer.
    async fn my_application_function(client: &speech::client::Speech) -> gax::Result<String> {
        client
            .get_recognizer()
            .set_name("invalid-test-recognizer")
            .send()
            .await
            .map(|r| r.display_name)
    }

    mockall::mock! {
        #[derive(Debug)]
        Speech {}
        impl speech::stub::Speech for Speech {
            async fn get_recognizer(&self, req: speech::model::GetRecognizerRequest, _options: gax::options::RequestOptions) -> gax::Result<gax::response::Response<speech::model::Recognizer>>;
        }
    }

    #[tokio::test]
    async fn basic_success() -> Result<()> {
        // Create a mock, and set expectations on it.
        let mut mock = MockSpeech::new();
        mock.expect_get_recognizer()
            .withf(move |r, _|
                // Optionally, verify fields in the request.
                r.name == "invalid-test-recognizer")
            .return_once(|_, _| {
                Ok(gax::response::Response::from(
                    speech::model::Recognizer::new().set_display_name("test-display-name"),
                ))
            });

        // Create a client, implemented by the mock.
        let client = speech::client::Speech::from_stub(mock);

        // Call our function.
        let display_name = my_application_function(&client).await?;

        // Verify the final result of the RPC.
        assert_eq!(display_name, "test-display-name");

        Ok(())
    }

    #[tokio::test]
    async fn basic_fail() -> Result<()> {
        let mut mock = MockSpeech::new();
        mock.expect_get_recognizer().return_once(|_, _| {
            // This time, return an error.
            use gax::error::Error;
            use gax::error::rpc::{Code, Status};
            let status = Status::default()
                .set_code(Code::NotFound)
                .set_message("Resource not found");
            Err(Error::service(status))
        });

        // Create a client, implemented by the mock.
        let client = speech::client::Speech::from_stub(mock);

        // Call our function.
        let display_name = my_application_function(&client).await;

        // Verify the final result of the RPC.
        assert!(display_name.is_err());

        Ok(())
    }
}
```
