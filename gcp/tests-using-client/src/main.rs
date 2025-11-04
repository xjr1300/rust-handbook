fn main() {
    println!("Hello, world!");
}

#[cfg(test)]
mod tests {
    use google_cloud_gax as gax;
    use google_cloud_speech_v2 as speech;
    type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

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

    mockall::mock! {
        #[derive(Debug)]
        Speech {}

        impl speech::stub::Speech for Speech {
            async fn get_recognizer(&self, req: speech::model::GetRecognizerRequest, _options: gax::options::RequestOptions) -> gax::Result<gax::response::Response<speech::model::Recognizer>>;
        }
    }

    #[tokio::test]
    async fn basic_success() -> Result<()> {
        // モックを作成し、それに予期する振る舞いを設定
        let mut mock = MockSpeech::new();
        mock.expect_get_recognizer()
            .withf(move |r, _|
                // オプションとして、リクエスト内のフィールドを検査
                r.name == "invalid-test-recognizer")
            .return_once(|_, _| {
                Ok(gax::response::Response::from(
                    speech::model::Recognizer::new().set_display_name("test-display-name"),
                ))
            });

        // モックによって実装されたクライアントを作成
        let client = speech::client::Speech::from_stub(mock);

        // 関数を呼び出す
        let display_name = my_application_function(&client).await?;

        // RPC呼び出しの最終的な結果を検証
        assert_eq!(display_name, "test-display-name");

        Ok(())
    }

    #[tokio::test]
    async fn basic_fail() -> Result<()> {
        let mut mock = MockSpeech::new();
        mock.expect_get_recognizer().return_once(|_, _| {
            // 今度は、エラーを返す
            use gax::error::Error;
            use gax::error::rpc::{Code, Status};
            let status = Status::default()
                .set_code(Code::NotFound)
                .set_message("Resource not found");
            Err(Error::service(status))
        });

        // モックによって実装されたクライアントを作成
        let client = speech::client::Speech::from_stub(mock);

        // 関数を呼び出す
        let display_name = my_application_function(&client).await;

        // RPC呼び出しの最終的な結果を検査
        assert!(display_name.is_err());

        Ok(())
    }
}
