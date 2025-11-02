# 12. バインディングエラーを処理する

<https://googleapis.github.io/google-cloud-rust/binding_errors.html>

リクエストを実行すると、次のようなエラーに遭遇することがあります。

```text
Error: cannot find a matching binding to send the request: at least one of the
conditions must be met: (1) field `name` needs to be set and match the template:
'projects/*/secrets/*' OR (2) field `name` needs to be set and match the
template: 'projects/*/locations/*/secrets/*'
```

これは*バインディングエラー*で、このガイドはバインディングエラーをトラブルシューティングする方法を説明します。

## バインディングエラーの原因

Google Cloud Client Libraries for Rustは、主にHTTPを使用して、Google Cloudサービスにリクエストを送信します。
HTTPリクエストは、Uniform Resource Identifier（[URI](https://clouddocs.f5.com/api/irules/HTTP__uri.html)）を使用して、リソースを特定します。

一部のRPCは複数のURIに対応しています。
リクエストの内容によって、どのURIが使用されるか決まります。

クライアントライブラリは、可能性のあるすべてのURIを考慮し、どのURIも機能しない場合のみ、バインディングエラーを返します。
通常、フィールドが欠落しているか、または不正なフォーマットになっている場合に、これが発生します。

上記のエラー例は、リソース名なしでリソースを取得しようとしたことによって生成されました。
具体的に、[GetSecretRequest](https://docs.rs/google-cloud-secretmanager-v1/latest/google_cloud_secretmanager_v1/model/struct.GetSecretRequest.html)の`name`フィールドは必須ですが、設定されていません。

```rust
    let secret = client
        .get_secret()
        // .set_name("projects/my-project/secrets/my-secret")
        .send()
        .await;
```

## 修正方法

この場合、エラーを修正するためには、`name`フィールドを設定し、エラーメッセージ内に表示されたテンプレートのいずれかに一致させる必要があります。

- `'projects/*/secrets/*`
- `'projects/*/locations/*/secrets/*'`

どちらも、クライアントライブラリがサーバーにリクエストを送信できるようにします。

```rust
    let secret = client
        .get_secret()
        .set_name("projects/my-project/secrets/my-secret")
        .send()
        .await;
```

または次です。

```rust
    let secret = client
        .get_secret()
        .set_name("projects/my-project/locations/us-central1/secrets/my-secret")
        .send()
        .await;
```

## テンプレートの解釈

バインディングエラー用のエラーメッセージは、リクエストフィールドに対する可能性のある値を示す多くのテンプレート文字列を含んでいます。
ほとんどのテンプレート文字列は、ワイルドカードとして`*`と`**`を含み、フィールド値に適合します。

### 単独のワイルドカード

単独の`*`ワイルドカードは、`/`のないからでない文字列を意味します。
それは正規表現`[^/]+`として考えることができます。

次に一部の例を示します。

| テンプレート | 入力 | マッチ？ |
| --- | --- | --- |
| `"*"` | `"simple-string-123"` | true |
| `"projects/*"` | `"projects/p"` | true |
| `"projects/*/locations"` | `"projects/p/locations"` | true |
| `"projects/*/locations/*"` | `"projects/p/locations/l"` | true |
| `"*"` | `""` (empty) | false |
| `"*"` | `"string/with/slashes"` | false |
| `"projects/*"` | `"projects/"` (empty) | false |
| `"projects/*"` | `"projects/p/"` (extra slash) | false |
| `"projects/*"` | `"projects/p/locations/l"` | false |
| `"projects/*/locations"` | `"projects/p"` | false |
| `"projects/*/locations"` | `"projects/p/locations/l"` | false |

### 2つのワイルドカード

一般的でないのは`**`ワイルドカードで、それは任意の文字列を意味します。
文字列は空、または任意の数の`/`を含めることができます。
それは、正規表現`.*`と考えることができます。

また、テンプレートが`/**`で終了しているとき、最初のスラッシュはオプションで含まれます。

| テンプレート | 入力 | マッチ？ |
| --- | --- | --- |
| `"**"` | `""` | true |
| `"**"` | `"simple-string-123"` | true |
| `"**"` | `"string/with/slashes"` | true |
| `"projects/*/**"` | `"projects/p"` | true |
| `"projects/*/**"` | `"projects/p/locations"` | true |
| `"projects/*/**"` | `"projects/p/locations/l"` | true |
| `"projects/*/**"` | `"locations/l"` | false |
| `"projects/*/**"` | `"projects//locations/l"` | false |

## エラーの検査

もし、プログラム的にエラーを検査する必要がある場合、それがバインディングエラーであるか確認して、次にそれを`BindingError`にダウンキャストすることでできます。

```rust
    let secret = client
        .get_secret()
        // .set_name("projects/my-project/secrets/my-secret")
        .send()
        .await;

    use gax::error::binding::BindingError;
    let e = secret.unwrap_err();
    assert!(e.is_binding(), "{e:?}");
    assert!(e.source().is_some(), "{e:?}");
    let _ = e
        .source()
        .and_then(|e| e.downcast_ref::<BindingError>())
        .expect("should be a BindingError");
```
