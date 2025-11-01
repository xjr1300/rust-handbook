# 8. フィールドマスクを使用してリソースをアップデートする

<https://googleapis.github.io/google-cloud-rust/update_resource.html>

このガイドは、フィールドマスクを使用してリソースを更新する方法を紹介します。これにより、リソースのどのフィールドを更新するか制御できるようになります。
このガイドは、リソースとして[Secret Manager](https://cloud.google.com/secret-manager/docs/overview)のシークレットを使用しますが、その概念は他のリソースやサービスに十分に適用できます。

```sh
cargo add google-cloud-secretmanager-v1
```

## 事前条件

このチュートリアルを完了するために、次の依存関係がインストールされたRust開発環境が必要です。

- Secret Managerクライアントライブラリ
- [Tokio](https://tokio.rs/)

準備するために、[開発環境の準備](https://googleapis.github.io/google-cloud-rust/setting_up_your_development_environment.html)の手順に従ってください。

## Well-Known Types(WKT)のインストール

[google_cloud_wkt](https://docs.rs/google-cloud-wkt/latest/google_cloud_wkt/index.html)クレートは、Google Cloud API用の**Well-Known Types（よく知られた型）**を含んでいます。
これらの型は、通常独自のJSONで符号化されており、ネイティブ型または一般的に使用されるRustの型との変換関数が提供されることがあります。

`google_cloud_wkt`は`FieldMask`のようなフィールドマスク型を含んでおり、依存関係としてそのクレートを追加する必要があります。

```sh
cargo add google-cloud-wkt
```

## FieldMask

[FieldMask](https://docs.rs/google-cloud-wkt/latest/google_cloud_wkt/struct.FieldMask.html)は、フィールドパスの集合を表現します。
フィールドマスクは、取得操作によって返されるフィールド、または更新操作によって変更されるフィールドの部分集合を指定するために使用されます。

更新操作で使用されるフィールドマスクは、更新対象となるリソースのフィールドを指定します。
APIは、マスクで指定されたフィールドの値のみを変更し、その他の残りフィールドはそのままにしておくことが要求されます。
もし、更新された値を記述するためにリソースが渡された場合、APIはマスクに含まれていないすべてのフィールドの値を無視します。
もし、更新でフィールドマスクが存在しない場合、その操作はすべてのフィールドに適用されます（すべてのフィールドのフィールドマスクが指定されたかのように）。

デフォルトにフィールドをリセットするために、マスク内にそのフィールドを含め、提供されたリソース内にデフォルト値を設定する必要があります。
したがって、リソースのすべてのフィールドをリセットするために、リソースのデフォルトのインスタンスを提供し、マスク内ですべてのフィールドを設定するか、マスクを提供しないようにします。

## リソースのフィールドを更新する

まず、Secret Managerを初期化し、シークレットを作成します。

```rust
let client = SecretMangerService::builder().build().await?;

let secret = client
    .create_secret()
    .set_parent(format!("projects/{project_id}"))
    .set_secret_id("your-secret")
    .set_secret(model::Secret::new().set_replication(
        model::Replication::new().set_automatic(model::replication::Automatic::new()),
    ))
    .send()
    .await?;
println!("CREATE = {secret:?}");
```

もし、作成操作の出力を確認する場合、`labels`と`annotations`フィールド両方が空であることを確認します。

次のコードは、`labels`と`annotations`フィールドを更新します。

```rust
let tag = |mut labels: HashMap<_, _>, msg: &str| {
    labels.insert("updated".to_string(), msg.to_string());
    labels
};

let update = client
    .update_secret()
    .set_secret(
        model::Secret::new()
            .set_name(&secret.name)
            .set_etag(secret.etag)
            .set_labels(tag(secret.labels, "your-label"))
            .set_annotations(tag(secret.annotations, "your-annotations")),
    )
    .set_update_mask(
        google_cloud_wkt::FieldMask::default().set_paths(["annotations", "labels"]),
    )
    .send()
    .await?;
println!("UPDATE = {update:?}");
```

`set_etag`メソッドは、シークレットに[etag](https://cloud.google.com/secret-manager/docs/etags)を設定し、それは同時更新による上書きを防止します。

ラベルと注釈を更新されたシークレットに設定することは、フィールドマスクを更新するためにフィールドのパスを記述する`set_update_mask`にフィールドマスクを渡すことを意味します。

```rust
    .set_update_mask(
        google_cloud_wkt::FieldMask::default().set_path(["annotations", "labels"]),
    )
```

更新操作の出力内で、更新されたフィールドを確認できます。

```text
 labels: {"updated": "your-label"},
...
annotations: {"updated": "your-annotations"},
```

完全なコードを[下で](https://googleapis.github.io/google-cloud-rust/update_resource.html#update-field-complete-code)確認してください。

## 次に学ぶこと

このガイドで、フィールドマスクを使用してリソースを更新しました。
サンプルコードはSecret Manager APIを使用しましたが、他のクライアントでもフィールドマスクを使用できます。
他のCloud Client Libraries for Rustの1つを試してください。

- [Vertex AI Gemini APIを使用してテキストを生成する](https://googleapis.github.io/google-cloud-rust/generate_text_using_the_vertex_ai_gemini_api.html)
- [Google Cloud Storageを使用する: オブジェクトをアップロードしてデータをプッシュする](https://googleapis.github.io/google-cloud-rust/storage/queue.html)

---

フィールドの更新: 完全なコード

```rust
pub async fn update_field(project_id: &str) -> anyhow::Result<()> {
    use google_cloud_secretmanager_v1::client::SecretManagerService;
    use std::collections::HashMap;

    let client = SecretManagerService::builder().build().await?;

    let secret = client
        .create_secret()
        .set_parent(format!("projects/{project_id}"))
        .set_secret_id("your-secret")
        .set_secret(model::Secret::new().set_replication(
            model::Replication::new().set_automatic(model::replication::Automatic::new()),
        ))
        .send()
        .await?;
    println!("CREATE = {secret:?}");

    let tag = |mut labels: HashMap<_, _>, msg: &str| {
        labels.insert("updated".to_string(), msg.to_string());
        labels
    };

    let update = client
        .update_secret()
        .set_secret(
            model::Secret::new()
                .set_name(&secret.name)
                .set_etag(secret.etag)
                .set_labels(tag(secret.labels, "your-label"))
                .set_annotations(tag(secret.annotations, "your-annotations")),
        )
        .set_update_mask(
            google_cloud_wkt::FieldMask::default().set_path(["annotations", "labels"]),
        )
        .send()
        .await?;
    println!("UPDATE = {update:?}");

    Ok(())
}
```
