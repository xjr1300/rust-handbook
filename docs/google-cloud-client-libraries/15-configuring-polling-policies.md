# 15. ポーリングポリシーを構成する

<https://googleapis.github.io/google-cloud-rust/configuring_polling_policies.html>

Google Cloud Client Libraries for Rustは、[LROs (Long-Running Operations)](https://googleapis.github.io/google-cloud-rust/working_with_long_running_operations.html)の進捗を監視し、完了まで待機するためのヘルパー関数を提供します。
これらのヘルパーは、ポーリングの頻度を構成したり、発生したポーリングエラーが一時的なもので次のポーリングイベントまで無視すべきかを判断したりするために、ポリシーを使用します。

このガイドでは、クライアントで開始されるすべての長時間実行操作に対するポリシーを構成する方法と、特定のリクエストのポリシーを上書きする方法について説明します。

LROループの動作を制御する2つの異なるポリシーがあります。

- 進行中のLROのステータスをポーリングする前に、どれくらいの時間ループを待機するかを制御するポーリングバックオフポリシー
- ポーリングエラーが発生したときに何をするかを制御するポーリングエラーポリシーです。
  一部のポーリングエラーは回復不可能で、操作が中止されたこと、または呼び出し側がLROの状態を確認する権限がないことを示します。
  他のポーリングエラーは一時的で、クライアントネットーワークまたはサービスで発生した一時的な問題を示します。

これらの各ポリシーは独立して設定でき、クライアントで開始されたすべての LRO に対して設定することも、特定のリクエストに対してのみ変更することも可能です。

## 事前条件

このガイドは[Cloud Storage](https://cloud.google.com/storage)サービスを使用して、具体的なコードスニペットを維持します。
この同じ考えは、LROを使用する他のサービスでも機能します。

このガイドは、[支払いが有効](https://cloud.google.com/billing/docs/how-to/verify-billing-enabled#confirm_billing_is_enabled_on_a_project)な[Google Cloudプロジェクト](https://cloud.google.com/resource-manager/docs/creating-managing-projects)があることを想定しています。

Rustライブラリの完全なセットアップ手順は、[開発環境のセットアップ](https://googleapis.github.io/google-cloud-rust/setting_up_your_development_environment.html)を参照してください。

## 依存関係

Rustでいつものことのように、`Cargo.toml`ファイルに使用する依存関係を宣言する必要があります。

```sh
cargo add google-cloud-storage google-cloud-lro
```

## クライアントですべてのリクエストのポーリング頻度を構成する

同じクライアントで、すべて（またはほとんど）のリクエストに対して同じポーリングバックオフポリシーを使用することを計画している場合は、これをクライアントオプションとして設定することを検討してください。

ポーリング頻度を構成するために、[PollingBackoffPolicy](https://docs.rs/google-cloud-gax/latest/google_cloud_gax/polling_backoff_policy/trait.PollingBackoffPolicy.html)トレイトを実装している型を使用します。
クライアントライブラリは、[ExponentialBackoff](https://docs.rs/google-cloud-gax/latest/google_cloud_gax/exponential_backoff/struct.ExponentialBackoff.html)も提供しています。

```rust
    use google_cloud_gax::exponential_backoff::ExponentialBackoffBuilder;
```

次に、希望する構成でクライアントを初期化します。

```rust
    let client = StorageControl::builder()
        .with_polling_backoff_policy(
              ExponentialBackoffBuilder::new()
                  .with_initial_delay(Duration::from_millis(250))
                  .with_maximum_delay(Duration::from_secs(10))
                  .build()?;
        )
        .build()
        .await?;
```

[リクエストごとの設定](https://googleapis.github.io/google-cloud-rust/configuring_polling_policies.html#configuring-the-polling-frequency-for-a-specific-request)でポリシーを上書きしない限り、このポリシーはクライアントで開始された長時間実行操作すべてに適用されます。
この例では、次のような呼び出しをします。

```rust
    let mut operation = client
        .rename_folder()
        /* より多くのこと */
        .send()
        .await?;
```

クライアントライブラリは、最初に500ミリ秒待機してから最初のポーリングを試行し、2回目の試行では1,000ミリ秒（または1秒）、後続の試行では2秒、4秒、8秒待機し、その後は最大10秒待機します。

完全なコードは[下を](https://googleapis.github.io/google-cloud-rust/configuring_polling_policies.html#configuring-the-polling-frequency-for-all-requests-in-a-client-complete-code)参照してください。

## 特定のリクエストに対してポーリング頻度を構成する

前のセクションで説明した通り、ポーリング頻度を構成するために、[PollingBackoffPolicy](https://docs.rs/google-cloud-gax/latest/google_cloud_gax/polling_backoff_policy/trait.PollingBackoffPolicy.html)トレイトを実装している型を必要としています。
この例で、また[ExponentialBackoff](https://docs.rs/google-cloud-gax/latest/google_cloud_gax/exponential_backoff/struct.ExponentialBackoff.html)を使用します。

```rust
    use google_cloud_gax::exponential_backoff::ExponentialBackoffBuilder;
    use std::time::Duration;
```

リクエストの構成には、スコープ内にトレイトをインポートすることが求められます。

```rust
    use google_cloud_gax::options::RequestOptionsBuilder;
```

リクエストビルダーを作成します。

```rust
    let response = client
        .rename_folder()
```

そして次に、ポーリングバックオフポリシーを構成します。

```rust
        .with_polling_backoff_policy(
            ExponentialBackoffBuilder::new()
                .with_initial_delay(Duration::from_millis(250))
                .with_maximum_delay(Duration::from_secs(10))
                .build()?,
        )
```

通常通りリクエストを発行できます。例えば:

```rust
        .poller()
        .until_done()
        .await?;

    println!("LRO completed, response={response:?}");
```

完全なコードは[下を](https://googleapis.github.io/google-cloud-rust/configuring_polling_policies.html#configuring-the-polling-frequency-for-a-specific-request-complete-code)参照してください。

## クライアントですべてのリクエストに対して再試行ポーリングエラーを構成する

再試行可能エラーを構成するために、[PollingErrorPolicy](https://docs.rs/google-cloud-gax/latest/google_cloud_gax/polling_error_policy/trait.PollingErrorPolicy.html)トレイトを実装している型を使用する必要があります。
クライアントライブラリは、それらを多く提供していますが、保守的な選択肢は[Aip194Strict](https://docs.rs/google-cloud-gax/latest/google_cloud_gax/polling_error_policy/struct.Aip194Strict.html)です。

```rust
    use google_cloud_gax::polling_error_policy::Aip194Strict;
    use google_cloud_gax::polling_error_policy::PollingErrorPolicyExt;
    use google_cloud_gax::retry_policy;
    use google_cloud_gax::retry_policy::RetryPolicyExt;
    use std::time::Duration;
```

同じクライアントで、すべて（またはほとんど）のリクエストに対して同じポーリングエラーポリシーを使用することを計画している場合は、クライアントオプションとしてこれを設定することを検討してください。

すべての長時間かかる操作に対して使用するポーリングエラーポリシーを追加します。

```rust
    let builder = StorageControl::builder()
        .with_polling_error_policy(
            Aip194Strict
                .with_attempt_limit(100)
                .with_time_limit(Duration::from_secs(300)),
        );
```

最初のリクエストで発生したエラーを処理するための再試行ポリシーを追加することもできます。

```rust
    let client = builder
        .with_retry_policy(
            retry_policy::Aip194Strict
                .with_attempt_limit(100)
                .with_time_limit(Duration::from_secs(300)),
        );
```

[リクエストごとの設定](https://googleapis.github.io/google-cloud-rust/configuring_polling_policies.html#configuring-the-polling-frequency-for-a-specific-request)でポリシーを上書きしない限り、このポリシーはクライアントで開始された長時間かかる処理に影響します。
例えば、次のように呼び出しできます。

```rust
    let mut operation = client
        .batch_recognize(/* stuff */)
        /* more stuff */
        .send()
        .await?;
```

クライアントライブラリは、再試行可能エラーとして`UNAVAILABLE`（[AIP-194](https://google.aip.dev/194)を参照してください）のみを取り扱うため、100回の試行または300秒のどちらかが最初に成立した後、ポーリングを停止します。

完全なコードは[下を](https://googleapis.github.io/google-cloud-rust/configuring_polling_policies.html#configuring-the-retryable-polling-errors-for-all-requests-in-a-client-complete-code)参照してください。

## 特定のリクエストに対して再試行可能ポーリングエラーを構成する

再試行可能エラーを構成するために、[PollingErrorPolicy](https://docs.rs/google-cloud-gax/latest/google_cloud_gax/polling_error_policy/trait.PollingErrorPolicy.html)トレイトを実装している型を使用します。
クライアントライブラリはそれらの多くを提供していますが、保守的な選択は[Aip194Strict](https://docs.rs/google-cloud-gax/latest/google_cloud_gax/polling_error_policy/struct.Aip194Strict.html)です。

```rust
    use google_cloud_gax::polling_error_policy::Aip194Strict;
    use google_cloud_gax::polling_error_policy::PollingErrorPolicyExt;
    use google_cloud_gax::retry_policy;
    use google_cloud_gax::retry_policy::RetryPolicyExt;
    use std::time::Duration;
```

リクエストの構成には、スコープ内にトレイトをインポートすることが求められます。

```rust
    use google_cloud_gax::options::RequestOptionsBuilder;
```

通常通り、リクエストビルダーを作成します。

```rust
    let response = client
        .rename_folder()
```

そして、次にポーリングバックオフポリシーを構成します。

```rust
        .with_polling_error_policy(
            Aip194Strict
                .with_attempt_limit(100)
                .with_time_limit(Duration::from_secs(300)),
        )
```

通常通り、このリクエストを発行できます。例えば:

```rust
        .poller()
        .until_done()
        .await?;

    println!("LRO completed, response={response:?}");
```

LROを開始するために、最初の要求が失敗した場合に備えて、再試行ポリシーを追加することを検討してください。

```rust
    let client = StorageControl::builder()
        .with_retry_policy(
            retry_policy::Aip194STrict
                .with_attempt_limit(100)
                .with_time_limit(Duration::from_secs(300)),
        )
        .build()
        .await?;
```

完全なコードは[下を](https://googleapis.github.io/google-cloud-rust/configuring_polling_policies.html#configuring-the-retryable-polling-errors-for-a-specific-request-complete-code)を参照してください。

## クライアント内ですべてのリクエストに対してポーリング頻度を構成する: 完全なコード

```rust
pub async fn client_backoff(bucket: &str, folder: &str, dest: &str) -> Result<()> {
    use google_cloud_gax::exponential_backoff::ExponentialBackoffBuilder;
    use google_cloud_lro::Poller;
    use std::time::Duration;

    let client = StorageControl::builder()
        .with_polling_backoff_policy(
            ExponentialBackoffBuilder::new()
                .with_initial_delay(Duration::from_millis(250))
                .with_maximum_delay(Duration::from_secs(10))
                .build()?,
        )
        .build()
        .await?;

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

## 特定のリクエストに対してポーリング頻度を構成する: 完全なコード

```rust
pub async fn rpc_backoff(bucket: &str, folder: &str, dest: &str) -> Result<()> {
    use google_cloud_gax::exponential_backoff::ExponentialBackoffBuilder;
    use std::time::Duration;
    use google_cloud_gax::options::RequestOptionsBuilder;
    use google_cloud_lro::Poller;

    let client = StorageControl::builder().build().await?;

    let response = client
        .rename_folder()
        .with_polling_backoff_policy(
            ExponentialBackoffBuilder::new()
                .with_initial_delay(Duration::from_millis(250))
                .with_maximum_delay(Duration::from_secs(10))
                .build()?,
        )
        .set_name(format!("projects/_/buckets/{bucket}/folders/{folder}"))
        .set_destination_folder_id(dest)
        .poller()
        .until_done()
        .await?;

    println!("LRO completed, response={response:?}");

    Ok(())
}
```

## クライアント内ですべてのリクエストに対して際し効果のポーリングエラーを構成する: 完全なコード

```rust
pub async fn client_backoff(bucket: &str, folder: &str, dest: &str) -> Result<()> {
    use google_cloud_gax::exponential_backoff::ExponentialBackoffBuilder;
    use google_cloud_lro::Poller;
    use std::time::Duration;

    let client = StorageControl::builder()
        .with_polling_backoff_policy(
            ExponentialBackoffBuilder::new()
                .with_initial_delay(Duration::from_millis(250))
                .with_maximum_delay(Duration::from_secs(10))
                .build()?,
        )
        .build()
        .await?;

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

## 特定のリクエストに対して再試行可能ポーリングエラーを構成する: 完全なコード

```rust
pub async fn rpc_backoff(bucket: &str, folder: &str, dest: &str) -> Result<()> {
    use google_cloud_gax::exponential_backoff::ExponentialBackoffBuilder;
    use std::time::Duration;
    use google_cloud_gax::options::RequestOptionsBuilder;
    use google_cloud_lro::Poller;

    let client = StorageControl::builder().build().await?;

    let response = client
        .rename_folder()
        .with_polling_backoff_policy(
            ExponentialBackoffBuilder::new()
                .with_initial_delay(Duration::from_millis(250))
                .with_maximum_delay(Duration::from_secs(10))
                .build()?,
        )
        .set_name(format!("projects/_/buckets/{bucket}/folders/{folder}"))
        .set_destination_folder_id(dest)
        .poller()
        .until_done()
        .await?;

    println!("LRO completed, response={response:?}");

    Ok(())
}
```
