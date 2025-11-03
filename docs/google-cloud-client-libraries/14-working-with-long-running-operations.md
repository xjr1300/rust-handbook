# 14. 長時間かかる操作の実行

<https://googleapis.github.io/google-cloud-rust/working_with_long_running_operations.html>

たまに、APIは完成するまでに非常に時間がかかるメソッドを公開することがあります。
これらの状況では、タスクが実行している間単にブロックすることは、ユーザーエクスペリエンスを低下させます。
通常、ユーザーに一種の「約束」（プロミス）を返し、後でユーザーが結果を確認できるようにすることが適切です。

Google Cloud Client Libraries for Rust は、LRO（Long Running Operations）と呼ばれる、これらのような長時間かかる作業のヘルパー機能を提供しています。
このガイドは、LRO を開始して、それらの完了を待機する方法を紹介します。

## 事前条件

このガイドは、コードスニペットの具体的な例として[Cloud Storage](https://cloud.google.com/storage)サービスを使用します。
この考え方は LRO を使用する他のサービスでも機能します。

このガイドは、[支払いが有効化](https://cloud.google.com/billing/docs/how-to/verify-billing-enabled#confirm_billing_is_enabled_on_a_project)された[Google Cloudプロジェクト](https://cloud.google.com/resource-manager/docs/creating-managing-projects)があることを想定しています。

Rustライブラリの完全なセットアップ手順は、[開発環境の準備](https://googleapis.github.io/google-cloud-rust/setting_up_your_development_environment.html)を参照してください。

## 依存関係

`Cargo.toml`ファイルにGoogle Cloudの依存関係を宣言します。

```sh
cargo add google-cloud-storage google-cloud-lro google-cloud-longrunning
```

また、いくつかの`tokio`フィーチャも必要です。

```sh
cargo add tokio --features="full,macros"
```

## 長時間かかる操作の開始

長時間かかる操作を開始するために、[クライアントを初期化](https://googleapis.github.io/google-cloud-rust/initialize_a_client.html)し、その後でRPCを呼び出します。
しかし、まず、いくつかの宣言を使用して、長いパッケージ名を避けます。

```rust
use anyhow::anyhow;
use google_cloud_longrunning as longrunning;
use google_cloud_storage::client::StorageControl;
```

ここでクライアントを作成します。

```rust
    let client = StorageControl::builder().build().await?;
```

この例のために[リネームフォルダ](https://cloud.google.com/storage/docs/rename-hns-folders)を使用します。
この操作は、大きなフォルダを使用するとき長い時間を費やすかもしれませんが、小さなフォルダでは比較的高速です。

Rustクライアントライブラリにおいて、それぞれのリクエストは、リクエストビルダーを返すメソッドによって表現されます。
まず、リクエストビルダーを作成するためにクライアントの適切なメソッドを呼び出します。

```rust
    let operation = client
        .rename_folder()
        .set_name(format!("projects/_/buckets/{bucket}/folders/{folder}"))
        .set_destination_folder_id(dest)
```

サンプル関数は引数としてバケットとフォルダの名前を受け取ります。

```rust
pub async fn manual(bucket: &str, folder: &str, dest: &str) -> anyhow::Result<()> {
```

リクエストを送信し、戻り値となる[Operation（操作）](https://docs.rs/google-cloud-longrunning/latest/google_cloud_longrunning/model/struct.Operation.html)を待ちます。
この`Operation`は、長時間かかるリクエストの結果のためのプロミスのように機能します。

```rust
    let operation =
        // ... ...
        .send()
        .await?;
```

このリクエストはバックグラウンドで操作を開始する一方、操作が成功したか失敗したかを確認するために、操作が完了するまで待機しなければなりません。
クライアントライブラリのポーリングループを使用する方法、または独自に記述する方法を学ぶために、引き続き読み続けてください。

[完全な関数](https://googleapis.github.io/google-cloud-rust/working_with_long_running_operations.html#automatically-polling-a-long-running-operation-complete-code)を下で見つけることができます。

## 長時間かかる操作を自動的にポーリング

自動ポーリングを構成するために、長時間かかる操作を開始するリクエストを準備します。
違いは最後にあり、それはリクエストを送信する代わりに、`Poller`を作成し、`.send().await`を呼び出す代わりに、それが終了するまで待機します。

```rust
        .poller()
        .until_done()
        .await?;
```

コードをひとつずつレビューしましょう。
まず、`use`宣言を介してスコープ内に`Poller`トレイトを導入します。

```rust
use google_cloud_lro::Poller;
```

次に、クライアントを初期化し、前のようにリクエストを準備します。

```rust
    let response = client
        .rename_folder()
        .set_name(format!("projects/_/buckets/{bucket}/folders/{folder}"))
        .set_destination_folder_id(dest)
```

そして、次に操作が完了するまでポーリングして、結果をプリントします。

```rust
        .poller()
        .until_done()
        .await?;

    println!("LRO completed, response={response:?}");
```

## 中間結果を持つ長時間かかる操作のポーリング

`.until_done()`が便利である一方、それはいくつかの情報を省略します。
長時間かかる操作は、「メタデータ」属性を介して部分的な進捗を報告する場合があります。
もし、アプリケーションがそのような情報を要求する場合、直接ポーラーを使用します。

```rust
    let mut poller = client
        .rename_object()
        /* もっと多くのもの */
        .poller();
```

そして、ループ内でポーラーを使用します。

```rust
    while let Some(p) = poller.poll().await {
        match p {
            PollingResult::Completed(r) => {
                println!("LRO completed, response={r:?}");
            }
            PollingResult::InProgress(m) => {
                println!("LRO in progress, metadata={m:?}");
            }
            PollingResult::PollingError(e) => {
                println!("Transient error polling the LRO: {e}");
            }
        }
        tokio::time::sleep(std::time::Duration::from_millis(500)).await;
    }
```

このループが再度ポーリングする前に、明示的に待機する方法に注意してください。
ポーリング期間は、特定の操作とそのペイロードに依存します。
適切な値を決定するために、サービスのドキュメントと独自データとの実験で考慮するべきです。

ポーラーはポリシーを使用して、問い合わせエラーが一時的で、それら自身で解決できることを決定します。
[ポーリングポリシーの構成](https://googleapis.github.io/google-cloud-rust/configuring_polling_policies.html)チャプターは、詳細にこの話題を説明しています。

下で[完全な関数](https://googleapis.github.io/google-cloud-rust/working_with_long_running_operations.html#polling-a-long-running-operation-complete-code)を見つけることができます。

## 長時間かかる操作を手動でポーリング

一般的に、アプリケーションで前の2つの手法を使用することを推奨します。
代わりに、長時間かかる操作に手動でポーリングできますが、これはとても面倒になり、簡単に型を間違えるかもしれません。
もし、手動で長時間かかる操作をポーリングする場合、このセクションで必要な手順を説明します。
一部のフィールドと型が下で使用されいるため、[Operation](https://docs.rs/google-cloud-longrunning/latest/google_cloud_longrunning/model/struct.Operation.html)メッセージのリファレンスドキュメントを読むことを推奨します。

クライアントを使用して長時間かかる操作を開始したことを思い出してください。

```rust
    let mut operation = client
        .rename_folder()
        /* もっと多くのもの */
        .send()
        .await?;
```

`operation`をポーリングするループを開始し、`done`フィールドを使用して操作が完了したか確認します。

```rust
    let response: anyhow::Result<Folder> = loop {
        if operation.done {
```

大抵の場合、操作が完了したとき、それは結果を含みます。
しかし、フィールドはオプションであるため、サービスは`true`と結果なしで`done`を返す可能性があります。
例えば、操作がリソースを削除したり、成功した完了は値を返しません。
この例で、ストレージサービスを使用して、この意味合いを無視して、値があることを仮定できます。

```rust
            match &operation.result {
                None => {
                    break Err(anyhow!("missing result for finished operation"));
                }
```

長時間かかる操作の開始を正常に開始しても、正常に完了することが保証されるわけではありません。
結果は、エラーまたは有効なレスポンスになる場合があります。
両方を確認する必要があります。
まず、エラーを確認します。

```rust
                Some(r) => {
                    break match r {
                        longrunning::model::operation::Result::Error(s) => {
                            Err(anyhow!("operation completed with error {s:?}"))
                        }
```

エラー型は[Status](https://docs.rs/google-cloud-rpc/latest/google_cloud_rpc/model/struct.Status.html)メッセージ型です。
これは、標準の`Error`インターフェイスを**実装していません**。
それを有効なエラーに手動で変換する必要があります。
この変換を実行するために、[Error::service](https://docs.rs/google-cloud-gax/latest/google_cloud_gax/error/struct.Error.html)を使用できます。

結果が成功だと仮定するために、レスポンス型を抽出する必要があります。
LROメソッドのドキュメント内、またはサービスのAPIドキュメントを読むことで、この型を見つけることができます。

```rust
                        longrunning::model::operation::Reult::Response(any) => {
                            let response = any.to_msg::<Folder>()?;
                            Ok(response)
                        }
```

型がサービスが送信したものと一致しない場合、値の抽出は失敗するかもしれません。

Google Cloud内のすべての型は、将来フィールドが追加され、型が分岐するかもしれません。
これは`Operation`のような一般的な型とは異なり、それはほとんどサービスメッセージに対して頻繁に発生します。
Google Cloud Client Libraries for Rustは、そのような変更が発生する可能性があることを示すために、すべての構造体と列挙型に`#[non_exhaustive]`と印を付けています。
この場合、予期しないケースを処理する必要があります。

```rust
                        _ => Err(anyhow!("unexpected result branch {r:?}")),
```

操作が完了していない場合、それは何らかのメタデータを含んでいるかもしれません。
一部のサービスはリクエストについての初期情報を含むだけですが、他のサービスは部分的な進捗リポートを含んでいます。
このメタデータを抽出してリポートすることを選択できます。

```rust
        if let Some(any) = &operation.metadata {
            let metadata = any.to_msg::<RenameFolderMetadata>()?;
            println!("LRO in progress, metadata={metadata:?}");
        }
```

操作が完了していないため、再度ポーリングする前に待機する必要があります。
切り捨て形式の[指数バックオフ](https://en.wikipedia.org/wiki/Exponential_backoff)の形式を使用して、ポーリング期間を調整することを検討してください。
この例は単純に500ミリ秒ごとにポーリングします。

```rust
        tokio::time::sleep(std::time::Duration::from_millis(500)).await;
```

もし、操作が完了していない場合、その状態を問い合わせする必要があります。

```rust
        if let Ok(attempt) = client
            .get_operation()
            .set_name(&operation.name)
            .send()
            .await
        {
            operation = attempt;
        }
```

単純化するために、この例はすべてのエラーを無視します。
アプリケーションでは、回復不可能としてエラーの部分集合のみを取り扱うことを選択し、これらが失敗した場合にポーリングを試みる回数を制限する場合があります。

[完全な関数](https://googleapis.github.io/google-cloud-rust/working_with_long_running_operations.html#manually-polling-a-long-running-operation-complete-code)を下で見つけることができます。

## 次に学ぶこと

- LRO用のエラー処理とバックオフ期間をカスタマイズする方法を学ぶために、[ポーリングポリシーの構成](https://googleapis.github.io/configuring_polling_policies.html)を確認してください。
- ユニットテストでLROを実行する方法を学ぶために、[長時間かかる操作のテストを記述する方法](https://googleapis.github.io/mocking_lros.html)を参照してください。

## 長時間かかる操作の開始: 完全なコード

```rust
pub async fn test(control: &StorageControl, bucket: &str) -> anyhow::Result<()> {
    for id in ["manual/", "automatic/", "polling/"] {
        let folder = control
            .create_folder()
            .set_parent(bucket)
            .set_folder_id(id)
            .send()
            .await?;
        println!("created folder {id}: {folder:?}");
    }
    let bucket_id = bucket.strip_prefix("projects/_/buckets/").ok_or(anyhow!(
        "bad bucket name format {bucket}, should start with `projects/_/buckets/`"
    ))?;
    println!("running manual LRO example");
    manual(bucket_id, "manual", "manual-renamed").await?;
    println!("running automatic LRO example");
    automatic(bucket_id, "automatic", "automatic-renamed").await?;
    println!("running automatic LRO with polling example");
    polling(bucket_id, "polling", "polling-renamed").await?;
    Ok(())
}
```

## 長時間かかる操作の自動的なポーリング: 完全なコード

```rust
pub async fn automatic(bucket: &str, folder: &str, dest: &str) -> anyhow::Result<()> {
    use google_cloud_lro::Poller;

    let client = StorageControl::builder().build().await?;

    let response = client
        .rename_folder()
        .set_name(format!("projects/_/buckets/{bucket}/folders/{folder}"))
        .set_destination_folder_id(dest)
        .poller()
        .until_done()
        .await?;

    println!("LRO completed, response={response:?}");

    Ok(())
}
```

## 長時間かかる操作のポーリング: 完全なコード

```rust
pub async fn polling(bucket: &str, folder: &str, dest: &str) -> anyhow::Result<()> {
    use google_cloud_lro::{Poller, PollingResult};

    let client = StorageControl::builder().build().await?;

    let mut poller = client
        .rename_folder()
        .set_name(format!("projects/_/buckets/{bucket}/folders/{folder}"))
        .set_destination_folder_id(dest)
        .poller();

    while let Some(p) = poller.poll().await {
        match p {
            PollingResult::Completed(r) => {
                println!("LRO completed, response={r:?}");
            }
            PollingResult::InProgress(m) => {
                println!("LRO in progress, metadata={m:?}");
            }
            PollingResult::PollingError(e) => {
                println!("Transient error polling the LRO: {e}");
            }
        }
        tokio::time::sleep(std::time::Duration::from_millis(500)).await;
    }

    Ok(())
}
```

## 長時間かかる操作を手動でポーリング: 完全なコード

```rust
pub async fn manual(bucket: &str, folder: &str, dest: &str) -> anyhow::Result<()> {
    use google_cloud_storage::model::Folder;
    use google_cloud_storage::model::RenameFolderMetadata;

    let client = StorageControl::builder().build().await?;

    let operation = client
        .rename_folder()
        .set_name(format!("projects/_/buckets/{bucket}/folders/{folder}"))
        .set_destination_folder_id(dest)
        .send()
        .await?;
    println!("LRO started, response={operation:?}");

    let mut operation = operation;
    let response: anyhow::Result<Folder> = loop {
        if operation.done {
            match &operation.result {
                None => {
                    break Err(anyhow!("missing result for finished operation"));
                }
                Some(r) => {
                    break match r {
                        longrunning::model::operation::Result::Error(s) => {
                            Err(anyhow!("operation completed with error {s:?}"))
                        }
                        longrunning::model::operation::Result::Response(any) => {
                            let response = any.to_msg::<Folder>()?;
                            Ok(response)
                        }
                        _ => Err(anyhow!("unexpected result branch {r:?}")),
                    };
                }
            }
        }
        if let Some(any) = &operation.metadata {
            let metadata = any.to_msg::<RenameFolderMetadata>()?;
            println!("LRO in progress, metadata={metadata:?}");
        }
        tokio::time::sleep(std::time::Duration::from_millis(500)).await;
        if let Ok(attempt) = client
            .get_operation()
            .set_name(&operation.name)
            .send()
            .await
        {
            operation = attempt;
        }
    };
    println!("LRO completed, response={response:?}");

    Ok(())
}
```
