# 11. エラーの詳細を調査する

<https://googleapis.github.io/google-cloud-rust/examine_error_details.html>

一部のGoogle Cloudサービスは、リクエストが失敗したとき、追加のエラーの詳細を含んでいます。
トラブルシューティングに役立てるために、Google Cloud Client Libraries for Rustは、`std::fmt::Display`を用いてフォーマットする際に、これらの詳細を含めるようにしています。
一部のアプリケーションは、これらの詳細を調べ、その内容に基づいて動作を変更したい場合があるでしょう。

このガイドは、Google Cloudサービスによって返されたエラーの詳細を調査する方法を紹介します。

## 事前条件

このガイドは[Cloud Natural Language API](https://cloud.google.com/natural-language)を使用し、エラーの詳細を紹介しますが、この概念は他のサービスでも広く適用できます。

そのサービスで認証を設定し、サービスを有効にする方法が記載されている[クイックスタート](https://cloud.google.com/natural-language/docs/setup)に従うことを推奨します。

Rustライブラリの完全なセットアップ手順は、[開発環境の準備](https://googleapis.github.io/google-cloud-rust/setting_up_your_development_environment.html)を参照してください。

### 依存関係

Rustでは一般的なことですが、`Cargo.toml`ファイル内で依存関係を宣言する必要があります。

```sh
cargo add google-cloud-language-v2
```

## エラーの詳細を調査する

意図的にエラーとなるようなリクエストを作成し、エラー内容を調べます。
まず、クライアントを作成します。

```rust
use google_cloud_language_v2 as lang;
let client = lang::client::LanguageService::builder().build().await?;
```

そして、リクエストを送信します。
この場合、必須なフィールドが欠落しています。

```rust
let result = client
    .analyze_sentiment()
    .set_document(
        lang::model::Document::new()
            // ドキュメントのコンテンツが欠落
            // .set_content("Hello World!")
            .set_type(lang::model::document::Type::PlainText),
    )
    .send()
    .await;
```

標準のRustの関数を使用して、結果からエラーを抽出します。
エラー型は人間が読める形式ですべてのエラーの詳細をプリントします。

```rust
let err = result.expect_err("the request should have failed");
println!("\nrequest failed with error {err:#?}");
```

これは、次のような出力を生成します。

```text
request failed with error Error {
    kind: Service {
        status_code: Some(
            400,
        ),
        headers: Some(
            {
                "vary": "X-Origin",
                "vary": "Referer",
                "vary": "Origin,Accept-Encoding",
                "content-type": "application/json; charset=UTF-8",
                "date": "Sat, 24 May 2025 17:19:49 GMT",
                "server": "scaffolding on HTTPServer2",
                "x-xss-protection": "0",
                "x-frame-options": "SAMEORIGIN",
                "x-content-type-options": "nosniff",
                "alt-svc": "h3=\":443\"; ma=2592000,h3-29=\":443\"; ma=2592000",
                "accept-ranges": "none",
                "transfer-encoding": "chunked",
            },
        ),
        status: Status {
            code: InvalidArgument,
            message: "One of content, or gcs_content_uri must be set.",
            details: [
                BadRequest(
                    BadRequest {
                        field_violations: [
                            FieldViolation {
                                field: "document.content",
                                description: "Must have some text content to annotate.",
                                reason: "",
                                localized_message: None,
                                _unknown_fields: {},
                            },
                        ],
                        _unknown_fields: {},
                    },
                ),
            ],
        },
    },
}
```

### エラーの詳細をプログラム的に調査する

時々、プログラム的にエラーの詳細を調査する必要があるかもしれません。
例の残りは、データ構造をたどり、最も関連するフィールドをプリントします。

詳細な情報を含むのはサービスによって返されたエラーのみであるため、まず、エラーが適切なエラー型を含んでいるか確認します。
もし、含まれている場合、エラーに関する最上位の情報を分解できます。

```rust
if let Some(status) = err.status() {
    println!(
        "  status.code={}, status.message={}",
        status.code, status.message,
    );
}
```

そして、次に詳細をすべて反復します。

```rust
    for detail in status.details.iter() {
        use google_cloud_gax::error::rpc::StatusDetail;
        match detail {
    }
```

クライアントライブラリは、さまざまな種類のエラーの詳細を含む[StatusDetails](https://docs.rs/google-cloud-gax/latest/google_cloud_gax/error/rpc/enum.StatusDetails.html)列挙型を返します。
この例では、`BadRequest`エラーのみを調査します。

```rust
            StatusDetails::BadRequest(bad) => {
```

`BadRequest`は違反しているフィールドのリストを含んでいます。
それぞれの詳細を反復してプリントできます。

```rust
                for f bad.field_violations.iter() {
                    println!(
                        "  the request field {} has a problem: \"{}\"",
                        f.field, f.description
                    );
                }
```

この情報は開発時に役に立ちます。
[QuotaFailure](https://docs.rs/google-cloud-gax/latest/google_cloud_gax/error/rpc/enum.StatusDetails.html#variant.QuotaFailure)のような`StatusDetails`の他のブランチは、アプリケーションを制限するためにランタイム時に役に立つかもしれません。

### 予期された出力

通常、エラー詳細からの出力は次のようになります。

```text
  status.code=400, status.message=One of content, or gcs_content_uri must be set., status.status=Some("INVALID_ARGUMENT")
  the request field document.content has a problem: "Must have some text content to annotate."
```

## 次に学ぶこと

- クライアントライブラリがHTTPリクエストに適合するURIを見つけられないときに発生する[バインディングエラーを処理](https://googleapis.github.io/google-cloud-rust/binding_errors.html)する方法を学ぶ
- [リスト操作を行う](https://googleapis.github.io/google-cloud-rust/pagination.html)方法を学ぶ

---

## エラーの詳細を調査する: 完全なコード

```rust
pub async fn examine_error_details() -> crate::Result<()> {
    use google_cloud_language_v2 as lang;
    let client = lang::client::LanguageService::builder().build().await?;

    let result = client
        .analyze_sentiment()
        .set_document(
            lang::model::Document::new()
                // Missing document contents
                // .set_content("Hello World!")
                .set_type(lang::model::document::Type::PlainText),
        )
        .send()
        .await;

    let err = result.expect_err("the request should have failed");
    println!("\nrequest failed with error {err:#?}");

    if let Some(status) = err.status() {
        println!(
            "  status.code={}, status.message={}",
            status.code, status.message,
        );
        for detail in status.details.iter() {
            use google_cloud_gax::error::rpc::StatusDetails;
            match detail {
                StatusDetails::BadRequest(bad) => {
                    for f in bad.field_violations.iter() {
                        println!(
                            "  the request field {} has a problem: \"{}\"",
                            f.field, f.description
                        );
                    }
                }
                _ => {
                    println!("  additional error details: {detail:?}");
                }
            }
        }
    }

    Ok(())
}
```
