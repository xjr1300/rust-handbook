# 13. リスト操作を実行する

<https://googleapis.github.io/google-cloud-rust/pagination.html>

一部のサービスは、行またはリソースの詳細のようなアイテムの大きなリストを返す可能性があります。
CPUとメモリ使用を制御下に維持するために、サービスは**ページ**でこれらのリソースを返します。
ページは、小さなアイテムの部分集合で、次の部分集合をリクエストするための継続トークンがあります。

ページによってアイテムを反復処理することは、面倒な場合があります。
クライアントライブラリは、アダプターを提供し、ページを非同期イテレーターに変換します。
このガイドは、これらのアダプターを使用して実行する方法を紹介します。

## 事前条件

このガイドは、[Secret Manager](https://cloud.google.com/secret-manager)サービスを使用して、エラー処理を説明しますが、この概念は他のサービスにも広く適用できます。

Secret Managerサービスの[クイックスタート](https://cloud.google.com/secret-manager/docs/quickstart)を完了しておくことをお勧めします。
そのガイドでは、サービスを有効にし、ログインしていること、アカウントに必要な権限があることを確認するために必要な手順について説明します。

Rustライブラリの完全なセットアップ手順については、[開発環境の準備](https://googleapis.github.io/google-cloud-rust/setting_up_your_development_environment.html)を参照してください。

## 依存関係

`Cargo.toml`ファイルにSecret Managerライブラリを追加します。

```sh
cargo add google-cloud-secretmanager-v1
```

## リストを反復処理するメソッド

リストメソッドでアイテムを反復処理することに役に立つように、APIは`ItemPaginator`トレイトの実装を返します。
`use`宣言を使用して、スコープ内にそれを導入します。

```rust
use google_cloud_gax::paginator::ItemPaginator as _;
```

アイテムを反復処理するために、`by_item`メソッドを使用します。

```rust
  let mut list = client
      .list_secrets()
      .set_parent(format!("projects/{project_id}"))
      .by_item();
  while let Some(secret) = list.next().await {
      let secret = secret?;
      println!("  secret={}", secret.name);
  }
```

レアケースで、ページはアクセスする必要がある追加情報を含んでいる可能性があります。
または、処理全体にわたって進行状況をチェックポイントする必要がある場合もあります。
これらの場合、個別のアイテムでなく全ページを反復処理することができます。

まず、`use`宣言を介してスコープ内に`Paginator`を導入します。

```rust
use google_cloud_gax::paginator::Paginator as _;
```

次に、`by_page`を使用してページを反復処理します。

```rust
  let mut list = client
      .list_secrets()
      .set_parent(format!("projects/{project_id}"))
      .by_page();
  while let Some(page) = list.next().await {
      let page = page?;
      println!("  next_page_token={}", page.next_page_token);
      page.secrets
          .into_iter()
          .for_each(|secret| println!("    secret={}", secret.name));
  }
```

### `futures::Stream`で実行する

`tokio::Stream`のような巨大なRustエコシステムの非同期ストリームで、これらのAPIを使用する場合があります。
これはすぐにできますが、まず`google_cloud_gax`クレートで`unstable-streams`フィーチャを有効にする必要があります。

```sh
cargo add google-cloud-gax --features="unstable-stream"
```

このフィーチャの名前は、これらのAPIが不安定だと考えていることを伝えることを意図しており、実際に不安定です。
[futures::Stream](https://docs.rs/futures/latest/futures/stream/)トレイトへの互換性のない変更によって生じるあらゆる破壊に対処する準備ができている場合にのみ、これらを使用する必要があります。

また、次の例は`futures::stream::StreamExt`トレイトを使用し、それは`futures`クレートを追加することで有効になります。

```sh
cargo add futures
```

`use`宣言で必要なものを追加します。

```rust
use futures::stream::StreamExt as _;
use google_cloud_gax::paginator::ItemPaginator as _;
```

そして、`into_stream`メソッドを使用して、`ItemPaginator`を`futures::Stream`のアイテムに変換します。

```rust
  let list = client
      .list_secrets()
      .set_parent(format!("projects/{project_id}"))
      .by_item()
      .into_stream();
  list.map(|secret| -> gax::Result<()> {
      println!("  secret={}", secret?.name);
      Ok(())
  })
  .fold(Ok(()), async |acc, result| -> gax::Result<()> {
      acc.and(result)
  })
  .await?;
```

## 次のページトークンを設定してリストメソッドを再開する

いくつかの場面で、リスト操作の中断後など、次のページトークンを設定して、特定のページからページ送りを再開できます。

```rust
    let page = client
        .list_secrets()
        .set_parent(format!("projects/{project_id}"))
        .send()
        .await;
    let page = page?;
    let mut next_page_token = page.next_page_token.clone();
    page.secrets
        .into_iter()
        .for_each(|secret| println!("    secret={}", secret.name));

    while !next_page_token.is_empty() {
        println!("  next_page_token={next_page_token}");

        let page = client
            .list_secrets()
            .set_parent(format!("projects/{project_id}"))
            .set_page_token(next_page_token)
            .send()
            .await;
        let page = page?;
        next_page_token = page.next_page_token.clone();

        page.secrets
            .into_iter()
            .for_each(|secret| println!("    secret={}", secret.name));
    }
```

## 追加のページネーターの技術的詳細

標準的な[Google APIリスト](https://google.aip.dev/132)メソッドは、[AIP-158](https://google.aip.dev/158)で定義されたページネーションガイドラインに従っています。
リソースに対するリストメソッドのそれぞれの呼び出しは、次のページを取得するリストメソッドに渡すことができる次のページのトークンと一緒にリソースのアイテムのページを返します。

Google Cloud Client Libraries for Rustは、[AIP-4233](https://google.aip.dev/client-libraries/4233)で定義されたリストRPCを、非同期方式で反復処理できるストリームに変換するアダプタを提供しています。

## 次に学ぶこと

Google Cloud Client Libraries for Rustを使用した作業についてより学びたい場合は次を参照してください。

- [長い時間がかかる操作を実行する](https://googleapis.github.io/google-cloud-rust/working_with_long_running_operations.html)
- [ポーリングポリシーを構成する](https://googleapis.github.io/google-cloud-rust/configuring_polling_policies.html)
