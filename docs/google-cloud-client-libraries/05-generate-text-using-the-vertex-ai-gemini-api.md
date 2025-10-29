# Vertex AI Gemini APIを使用してテキストを生成する

<https://googleapis.github.io/google-cloud-rust/generate_text_using_the_vertex_ai_gemini_api.html>

このガイドでは、テキストプロンプトリクエストを送信し、次にマルチモーダルプロンプト（プロンプトと画像）リクエストをVertex AI Gemini APIに送信して、応答を得ます。

## 前提条件

このガイドを完了するために、Vertex AI APIを有効にしたGoogle Cloudプロジェクトが必要です。
これらの手順を完了するために[Vertex AIセットアップガイド](https://cloud.google.com/vertex-ai/docs/start/cloud-environment)を使用できます。

## 依存関係としてVertex AI Client Libraryを追加する

Vertex AI Client Libraryは多くの機能を含んでいます。
それらをすべてコンパイルすることは、比較的時間がかかります。
コンパイル時間を短くするために、必要とする機能だけ有効にできます。

```sh
cargo add google-cloud-aiplatform-v1 --no-default-features --features prediction-service
```

## Vertex AI Gemini APIにプロンプトを送信する

まず、デフォルトの設定を使用してクライアントを初期化します。

```rust
use google_cloud_aiplatform_v1 as vertexai;
let clinet = vertexai::client::PredictionService::builder()
    .build()
    .await?;
```

次にモデル名を構築します。
簡潔にするために、この例は引数でプロジェクトIDを受け取り、固定された場所（`global`）とモデルID（`gemini-2.0-flash-001`）を使用します。

```rust
const MODEL: &str = "gemini-2.0-flash-001";
let model = format("projects/{project_id}/locations/global/publishers/google/models/{MODEL}");
```

自分のコードでこの関数を実行したい場合、前提条件を確認している間に選択したプロジェクトのプロジェクトID（接頭句の`projects/`なし）を使用します。

> Google Cloud Client Libraries for Rustを利用している場合、関数などが完全なリソース名を要求していない限り、`projects/`を付けなくて良い。
> ただし、gcloud CLI、REST APIの呼び出しなどでは、`projects/`を付けなければならない場合がある。
>
> 今回の場合、`projects/`を付ける必要があり、付けなかった場合は`BindingError`がレスポンスとして返される。

初期化されたクライアントで、リクエストを送信できます。

```rust
let response = client
    .generate_content().set_model(&model)
    .get_contents(\vertexai::model::Content::new().set_role("user").set_parts([
        vertexai::model::Part::new().set_text("What's a good name for flower shop that specializes in selling bouquets of dried flowers?"),
    ]))
    .send()
    .await;
```

そして、次にレスポンスをプリントします。
`:#?`書式指定子を使用して、ネストされたレスポンスオブジェクトを整形できます。

```rust
println!("RESPONSE = {response:#?}");
```

[下の](https://googleapis.github.io/google-cloud-rust/generate_text_using_the_vertex_ai_gemini_api.html#prompt-and-image-complete-code)最終的なコードを確認してください。

## Vertex AI Gemini APIにプロンプトと画像を送信する

前の例において、デフォルトの設定を使用してクライアントを初期化しました。

```rust
use google_cloud_aiplatform_v1 as vertexai;
let client = vertexai::client::PredictionService::builder()
    .build()
    .await?;
```

そして、次にモデル名を構築しました。

```rust
const MODEL: &str = "gemini-2.0-flash-001";
let model = format!("projects/{project_id}/locations/global/publishers/google/models/{MODEL}");
```

新しいリクエストは画像部分を含みます。

```rust
vertexai::model::Part::new().set_file_data(
    vertexai::model::FileData::new()
        .set_mime_type("image/jpeg")
        .set_file_uri("gs://generativeai-downloads/images/scones.jpg"),
),
```

プロンプト部分も含みます。

```rust
vertexai::model::Part::new().set_text("Describe this picture."),
```

リクエスト全体を送信します。

```rust
let response = client
    .generate_content()
    .set_model(&model)
    .set_contents(
        [vertexai::model::Content::new().set_role("user").set_parts([
            vertexai::model::Part::new().set_file_data(
                vertexai::model::FileData::new()
                    .set_mime_type("image/jpeg")
                    .set_file_uri("gs://generativeai-downloads/images/scones.jpg"),
            ),
            vertexai::model::Part::new().set_text("Describe this picture."),
        ])],
    )
    .send()
    .await;
```

前の例において、レスポンス全体をプリントしました。

```rust
println!("RESPONSE = {response:#?}");
```

[下の](https://googleapis.github.io/google-cloud-rust/generate_text_using_the_vertex_ai_gemini_api.html#prompt-and-image-complete-code)最終的なコードを確認してください。

## テキストプロンプト: 最終的なコード

```rust
pub async fn text_prompt(project_id: &str) -> crate::Result<()> {
    use google_cloud_aiplatform_v1 as vertexai;
    let client = vertexai::client::PredictionService::builder()
        .build()
        .await?;

    const MODEL: &str = "gemini-2.0-flash-001";
    let model = format!("projects/{project_id}/locations/global/publishers/google/models/{MODEL}");

    let response = client
        .generate_content().set_model(&model)
        .set_contents([vertexai::model::Content::new().set_role("user").set_parts([
            vertexai::model::Part::new().set_text("What's a good name for a flower shop that specializes in selling bouquets of dried flowers?"),
        ])])
        .send()
        .await;
    println!("RESPONSE = {response:#?}");

    Ok(())
}
```

## プロンプトと画像: 最終的なコード

```rust
pub async fn prompt_and_image(project_id: &str) -> crate::Result<()> {
    use google_cloud_aiplatform_v1 as vertexai;
    let client = vertexai::client::PredictionService::builder()
        .build()
        .await?;

    const MODEL: &str = "gemini-2.0-flash-001";
    let model = format!("projects/{project_id}/locations/global/publishers/google/models/{MODEL}");

    let response = client
        .generate_content()
        .set_model(&model)
        .set_contents(
            [vertexai::model::Content::new().set_role("user").set_parts([
                vertexai::model::Part::new().set_file_data(
                    vertexai::model::FileData::new()
                        .set_mime_type("image/jpeg")
                        .set_file_uri("gs://generativeai-downloads/images/scones.jpg"),
                ),
                vertexai::model::Part::new().set_text("Describe this picture."),
            ])],
        )
        .send()
        .await;
    println!("RESPONSE = {response:#?}");

    Ok(())
}
```

> 次のようなエラーが返されたら、「Service agents are being provisioned」なため、数分後に再度実行すること。
>
> ```text
> status: Status {
>     code: FailedPrecondition,
>     message: "Service agents are being provisioned (https://cloud.google.com/vertex-ai/docs/general/access-control#service-agents). Service agents are needed to read the Cloud Storage file provided. So please try again in a few minutes.",
>     details: [],
> },
> ```
