# 4. クライアントを初期化する方法

<https://googleapis.github.io/google-cloud-rust/initialize_a_client.html>

Google Cloud Client Libraries for Rustは、特定のサービスと連携するための主要な抽象化として、**クライアント**を使用します。
クライアントはRust構造体として実装され、サービスによって提供されるそれぞれのRPCに対応するメソッドがあります。
Rustクライアントライブラリを使用してGoogle Cloudサービスを使用するためには、最初にクライアントを初期化する必要があります。

## 前提条件

このガイドでは、クライアントを初期化し、クライアントを使用して単純なRPCを実行します。
同じ手順は、Google Cloudの他のサービスにも適用できます。

クライアントライブラリを使用する前に、[シークレットを作成する](https://cloud.google.com/secret-manager/docs/creating-and-accessing-secrets)方法のような、Secret Managerのスタートガイドのいずれかに従うことを推奨します。
これらのガイドはより詳細にサービス固有の概念を説明し、プロジェクトの前提条件に関する詳細なガイダンスを提供しています。

また、[クライアントライブラリを使用した認証](https://cloud.google.com/docs/authentication/client-libraries)にある指示に従うことも推奨します。
このガイドは、このガイドで使用される[アプリケーションデフォルトクレデンシャル](https://cloud.google.com/docs/authentication/application-default-credentials)を構成するために、ログインする方法を説明します。

## 依存関係

Rustではいつものことですが、`Cargo.toml`ファイルに依存関係を宣言しなければなりません。

```sh
cargo add google-cloud-secretmanager-v1
```

クライアントを初期化するために、`Client::builder()`を最初に呼び出し、適切な[ClientBuilder](https://docs.rs/google-cloud-gax/latest/google_cloud_gax/client_builder/struct.ClientBuilder.html)を取得し、次にビルダーの`build()`を呼び出してクライアントを作成します。

次は、ほとんどのユースケースに適合するように設計された、デフォルトの構成でクライアントを作成します。

```rust
let client = SecretManagerService::builder().build().await?;
```

クライアントが正しく初期化されると、RPCを実行するためにそれを利用できます。

```rust
use google_cloud_gax::paginator::Paginator as _;
let mut items = client
    .list_locations()
    .set_name(format!("projects/{project_id}"))
    .by_page();
while let Some(page) = items.next().await {
    let page = page?;
    for location in page.locations {
        println!("{}", location.name);
    }
}
```

この例は、サービス（この場合はSecret Manager）でサポートされている場所に関する情報を返す`list_locations`の呼び出しを示しています。
例の出力は、次のようになるはずです。

```text
projects/123456789012/locations/europe-west8
projects/123456789012/locations/europe-west9
projects/123456789012/locations/us-east5
...
```

---

## 完全なプログラム

完全なプログラムとしてすべてのコードをまとめると、次のようになります。

```rust
pub type Result = std::result::Result<(), Box<dyn std::error::Error>>;

pub async fn initialize_client(program_id: &str) -> Result {
    use google_cloud_secretmanager_v1::client::SecretManagerService;

    // デフォルトの構成でクライアントを初期化します。
    // これは、アクセストークンを取得することを要求する非同期操作で、失敗する可能性があります。
    let client = SecretManagerService::builder().build().await?;

    // 初期化されると、リクエストを作成するためにクライアントを使用できます。
    use google_cloud_gax::paginator::Paginator as _;
    let mut items = client
        .list_locations()
        .set_name(format!("projects/{project_id}"))
        .by_page();
    while let Some(page) = items.next().await {
        let page = page?;
        for location in page.locations {
            println!("{}", location.name);
        }
    }

    Ok(())
}
```

## 次は何をするべきか

このガイドは、Google Cloud Client Libraries for Rustを使用してクライアントを初期化する方法を紹介しました。
サービスと動作するより複雑な例は、[Vertex AI Gemini APIを使用してテキストを生成する](https://googleapis.github.io/google-cloud-rust/generate_text_using_the_vertex_ai_gemini_api.html)を確認してください。
