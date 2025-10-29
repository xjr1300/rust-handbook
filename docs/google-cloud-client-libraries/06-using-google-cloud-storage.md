# 6. Google Cloud Storageを使用する

<https://googleapis.github.io/google-cloud-rust/storage.html>

Google [Cloud Storage](https://cloud.google.com/storage)は、非構造データを保管するための管理されたサービスです。

Rustクライアントライブラリは、このサービスにアクセスする慣用的なAPIを提供しています。
クライアントライブラリは中断されたダウンロードとアップロードを再開し、データに対する整合性検査を自動的に実行します。
メタデータ操作用に、クライアントライブラリは失敗したリクエストを再試行でき、長い時間のかかる操作を自動的に定期的に問い合わせします（`poll`）。

## クイックスタート

このガイドは、Cloud Storageバケットを作成し、そのバケットにオブジェクトをアップロードし、そしてそのオブジェクトを読み込む方法を紹介します。

### 事前条件

このガイドは、[支払いが有効](https://cloud.google.com/billing/docs/how-to/verify-billing-enabled#confirm_billing_is_enabled_on_a_project)な、[Google Cloudプロジェクト](https://cloud.google.com/resource-manager/docs/creating-managing-projects)が存在することを想定しています。

### 依存関係としてクライアントライブラリを追加する

```sh
cargo add google-cloud-storage
```

### ストレージのバケットの作成

バケットまたはオブジェクトのメタデータに対する操作を実行するクライアントは、`StorageControl`と呼ばれます。

```rust
use google_cloud_storage as gcs;
use google_cloud_storage::client::StorageControl;
let control = StorageControl::builder().build().await?;
```

バケットを作成するために、プロジェクト名と希望するバケットのIDを提供しなければなりません。

```rust
let bucket = control
    .create_bucket()
    .set_parent("projects/_")
    .set_bucket_id(bucket_id)
    .set_bucket(
        gcs::model::Bucket::new()
            .set_project(format!("projects/{project_id}"))
    )
```

> `.set_parent("projects/_")`は、Cloud Storage API 仕様上の特殊な指定であり、
> バケット作成時に親リソース（parent）を省略したい場合に使用する予約値である。
> この値を指定した場合、実際の作成先プロジェクトは`.set_project("projects/{project_id}")`で指定されたプロジェクトになる。
> バケット同士に階層関係はなく、プロジェクト内にフラットな関係でバケットが存在する。
>
> `.set_parent("projects/_")`は、Cloud Storage API仕様上の特殊な指定で、
バケット作成時に親リソース（parent）を省略したい場合に使う予約値である。
> したがって、`project_id`で識別されるプロジェクトの最上位にバケットを作成することになる。
>
> バケットIDはバケット名と同義であり、バケット名はCloud Storage全体で（グローバルで）一意でなくてはならない。

また、バケットに対して他の属性を提供することもできます。
例えば、もしバケット内のすべてのオブジェクトを同じ権限で使いたい場合、[統一バケットレベルアクセス](https://cloud.google.com/storage/docs/uniform-bucket-level-access)を有効にできます。

```rust
gcs::model::Bucket::new()
    .set_project(format!("projects/{project_id}"))
    .set_iam_config(
        gcs::model::bucket::IamConfig::new().set_uniform_bucket_level_access(
            gcs::model::bucket::iam_config::UniformBucketLevelAccess::new()
                .set_enable(true),
        ),
    ),
```

そして、このリクエストを送信して、レスポンスを待ちます。

```rust
    .send()
    .await?;
println!("bucket successfully created {bucket:?}");
```

### オブジェクトをアップロードする

オブジェクトデータに対して操作を実行するクライアントは、`Storage`と呼ばれます。

```rust
use google_cloud_storage::client::Storage;
let client = Storage::builder().build().await?;
```

この例の場合、プログラミングチュートリアルにおいて伝統的な挨拶で、`hello.txt`と呼ばれるオブジェクトを作成します。

```rust
let object = client
    .write_object(&bucket.name, "hello.txt", "Hello, World!")
    .send_buffered()
    .await?;
println!("object successfully uploaded {object:?}");
```

> バケット内のオブジェクトも厳密的には階層関係はなく、オブジェクトに`/`区切りの名前を与えることで、あたかもディレクトリ階層のように見せることができる（例：`logs/2025/10/29/log.txt`）。

### オブジェクトをダウンロードする

オブジェクトの内容をダウンロードするために`read_object()`を使用します。

```rust
let mut reader = client.read_object(&bucket.name, "hello.txt").send().await?;
let mut contents = Vec::new();
while let Some(chunk) = reader.next().await.transpose()? {
    contents.extended_from_slice(&chunk);
}
println!(
    "object contents successfully downloaded {:?}",
    bytes::Bytes::from_owner(contents)
);
```

> 上記コードは[bytes](https://docs.rs/bytes/latest/bytes/index.html)クレートを依存関係に追加する必要がある。

### クリーンアップ

最後に、オブジェクトとバケットを削除して、このガイドで使用したすべてのリソースをクリーンアップします。

```rust
control
    .delete_object()
    .set_bucket(&bucket.name)
    .set_object(&object.name)
    .set_generation(object.generation)
    .send()
    .await?;
control
    .delete_bucket()
    .set_name(&bucket.name)
    .send()
    .await?;
```

### 次のステップ

- [オブジェクトのアップロード時にデータをプッシュする](https://googleapis.github.io/google-cloud-rust/storage/queue.html)
- [オブジェクトの再書き込み](https://googleapis.github.io/google-cloud-rust/storage/rewrite_object.html)
- [大きなオブジェクトのダウンロードを高速化する](https://googleapis.github.io/google-cloud-rust/storage/striped_downloads.html)

### 完全なプログラム

```rust
pub async fn quickstart(project_id: &str, bucket_id: &str) -> anyhow::Result<()> {
    use google_cloud_storage as gcs;
    use google_cloud_storage::client::StorageControl;
    let control = StorageControl::builder().build().await?;
    let bucket = control
        .create_bucket()
        .set_parent("projects/_")
        .set_bucket_id(bucket_id)
        .set_bucket(
            gcs::model::Bucket::new()
                .set_project(format!("projects/{project_id}"))
                .set_iam_config(
                    gcs::model::bucket::IamConfig::new().set_uniform_bucket_level_access(
                        gcs::model::bucket::iam_config::UniformBucketLevelAccess::new()
                            .set_enabled(true),
                    ),
                ),
        )
        .send()
        .await?;
    println!("bucket successfully created {bucket:?}");

    use google_cloud_storage::client::Storage;
    let client = Storage::builder().build().await?;

    let object = client
        .write_object(&bucket.name, "hello.txt", "Hello World!")
        .send_buffered()
        .await?;
    println!("object successfully uploaded {object:?}");

    let mut reader = client.read_object(&bucket.name, "hello.txt").send().await?;
    let mut contents = Vec::new();
    while let Some(chunk) = reader.next().await.transpose()? {
        contents.extend_from_slice(&chunk);
    }
    println!(
        "object contents successfully downloaded {:?}",
        bytes::Bytes::from_owner(contents)
    );

    control
        .delete_object()
        .set_bucket(&bucket.name)
        .set_object(&object.name)
        .set_generation(object.generation)
        .send()
        .await?;
    control
        .delete_bucket()
        .set_name(&bucket.name)
        .send()
        .await?;

    Ok(())
}
```

> 上記コードでオブジェクトを削除しているときにジェネレーションを設定（`set_generation(object.generation)`）している。
> これは、オブジェクトのバージョニングが有効になっている場合に、誤って異なる世代のオブジェクトを削除してしまうことを防ぐための安全機構である。
>
> Cloud Storageにおいて、オブジェクトのバージョニングを有効にした場合、そのオブジェクトに対して何回も上書き（更新）できる。
> このとき、オブジェクトの世代を識別するために**ジェネレーション番号**が自動に付与される。
>
> 例えば、削除しようとしたオブジェクトが、すでに新しい世代に置き換わっていた場合でも、古いジェネレーションのオブジェクトのみを削除でき、最新のジェネレーションのオブジェクトは削除されない。
>
> 一方で、オブジェクトを削除するときにジェネレーションを指定しない場合、**現行世代（最新のオブジェクト）**が削除され、**非現行オブジェクト（古い世代）**が残る。
> ただし、古い世代は通常のオブジェクト一覧には表示されない。
> また、古い世代が残っていれば、ジェネレーションを指定してそのオブジェクトを復元できる。
>
> **削除前**
>
> | オブジェクト名 | ジェネレーション | 状態 | 備考 |
> | --- | --- | --- | --- |
> | report.csv | 1700000000000000 | 非現行 | 古い版 |
> | report.csv | 1700000000001234 | 非現行 | 古い版 |
> | report.csv | 1700000000005678 | 現行 | 最新版 |
>
> **最新版を削除した後**
>
> | オブジェクト名 | ジェネレーション | 状態 | 備考 |
> | --- | --- | --- | --- |
> | report.csv | 1700000000000000 | 非現行 | 古い版 |
> | report.csv | 1700000000001234 | 非現行 | 古い版 |
> | report.csv | 1700000000005678 | 非現行（削除された現行） | - |
>
> ```rust
> // ジェネレーションを指定していないため、最新のオブジェクトが削除される。
> control
>     .delete_object()
>     .set_bucket("my-bucket")
>     .set_object("report.csv")
>     .send()
> ```
>
> Cloud Storageでバケットを削除すると、そのバケットに含まれるすべてのオブジェクトと、そのすべての世代（ジェネレーション）も完全に削除される。
>
> Cloud Storageを使用するときの注意事項を次に示す。
>
> - 古い世代も課金対象になる
> - ライフライクルルールを設定ない限り、古い世代は自動的に削除されない。
> - 世代を含めてオブジェクトを完全に削除するためには、すべての世代を取得して、それらを削除する必要がある。
> - バージョニングはバケット単位の設定であり、オブジェクト単位で有効・無効を切り替えることはできない。
>
