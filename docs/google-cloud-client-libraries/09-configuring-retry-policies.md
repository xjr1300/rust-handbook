# 9. 再試行ポリシーを構成する

<https://googleapis.github.io/google-cloud-rust/configuring_retry_policies.html>

Google Cloud Client Libraries for Rustは、一時的なエラーによって失敗した操作を自動的に再試行します。

このガイドは、再試行ループをカスタマイズする方法を紹介します。
まず、クライアント内のすべてのリクエストに対して共通の再試行ポリシーを有効にする方法を学び、その後、特定のリクエストに対してこのデフォルトを上書きする方法を学びます。

## 事前条件

このガイドは[Secret Manager](https://cloud.google.com/secret-manager)サービスを使用します。
これにより、例がより具体的になり、理解しやすくなります。
そうは言っても、同じ考え方は他のサービスでも機能します。

Secret Managerサービスの[クイックスタート](https://cloud.google.com/secret-manager/docs/quickstart)を完了しておくことをお勧めします。
このガイドでは、サービスを有効にし、ログインしていること、アカウントに必要な権限があることを確認するために必要な手順について説明します。

## 依存関係

Rustでは通常、`Cargo.toml`ファイル内に依存関係を宣言する必要があります。

```sh
cargo add google-cloud-secretmanager-v1
```

## デフォルト再試行ポリシーを構成する

この例は[Aip194Strict](https://docs.rs/google-cloud-gax/latest/google_cloud_gax/retry_policy/struct.Aip194Strict.html)ポリシーを使用します。
このポリシーは、[AIP-194](https://aip.dev/194)のガイドラインに基づいており、それはGoogle APIクライアントがリクエストを自動的に再試行すべき条件を文書化しています。
そのポリシーはかなり保守的で、リクエストが冪等でない限り、サービスに届いたかもしれないことを示すエラーを再試行しません。
これにより、そのポリシーはデフォルトで使用しても安全です。
唯一の欠点は、サービスへの追加的なリクエストによって、クォータが消費され、課金が発生する可能性があることです。

> GCPでは、予期しない出費を抑えるため、ユーザーやプロジェクトが利用できるAPIのリソースを制限できる。
> クォータは、APIの利用料の上限、特にAPIのリクエスト回数の上限を示す。

サービスに対してデフォルトのポリシーを作成するために、クライアントを初期化している間にポリシーを設定します。

```rust
let client = secret_manager::client::SecretManagerService::builder()
    .with_retry_policy(Aip194Strict)
    .build()
    .await?;
```

そして、通常通りサービスを使用します。

```rust
let mut list = client
    .list_secrets()
    .set_parent(format!("projects/{project_id}"))
    .by_item();
while let Some(secret) = list.next().await {
    let secret = secret?;
    println!("   secret={}", secret.name);
}
```

完全なコードは[下を](https://googleapis.github.io/google-cloud-rust/configuring_retry_policies.html#configuring-the-default-retry-policy-complete-code)確認してください。

## 制限を持つデフォルトの再試行ポリシーを構成する

[Aip194Strict](https://docs.rs/google-cloud-gax/latest/google_cloud_gax/retry_policy/struct.Aip194Strict.html)ポリシーは、再試行を試みる回数、または再試行リクエストに費やす時間を制限しません。
しかし、それは制限を設定することで拡張できます。
例えば、試行回数と再試行ループに費やす時間の*両方を*制限できます。

```rust
let client = secret_manager::client::SecretManagerService::builder()
    .with_retry_policy(
        Aip194Strict
            .with_attempt_limit(5)
            .with_time_limit(Duration::from_secs(15))
    )
    .build()
    .await?;
```

リクエストも通常通り機能します。

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

完全なコードは[下を](https://googleapis.github.io/google-cloud-rust/configuring_retry_policies.html#configuring-the-default-retry-policy-complete-code)確認してください。

## 1つのリクエストに対して再試行ポリシーを上書きする

時々、アプリケーションは、特定のリクエストに対して再試行ポリシーをオーバーライドする必要があります。
例えば、アプリケーションの開発者は、サーバーまたはアプリケーションの詳細を把握しており、よりエラーを許容しても安全であると判断する場合があります。

例えば、シークレットの削除は冪等なので、それは一度だけ成功できます。
しかし、クライアントライブラリは、すべての削除操作は安全でないと想定します。
アプリケーションは、1つのリクエストに対してポリシーを上書きできます。

```rust
client
    .delete_secret()
    .set_name(format!("projects/{project_id}/secrets/{secret_id}"))
    .with_retry_policy(
        AlwaysRetry
            .with_attempt_limit(5)
            .with_time_limit(Duration::from_secs(15)),
    )
    .send()
    .await?;
```

完全なコードは[下を](https://googleapis.github.io/google-cloud-rust/configuring_retry_policies.html#configuring-the-default-retry-policy-complete-code)確認してください。

## デフォルトの再試行ポリシーを構成する: 完全なコード

```rust
pub async fn client_retry(project_id: &str) -> crate::Result<()> {
    use google_cloud_gax::paginator::ItemPaginator as _;
    use google_cloud_gax::retry_policy::Aip194Strict;
    use google_cloud_secretmanager_v1 as secret_manager;

    let client = secret_manager::client::SecretManagerService::builder()
        .with_retry_policy(Aip194Strict)
        .build()
        .await?;

    let mut list = client
        .list_secrets()
        .set_parent(format!("projects/{project_id}"))
        .by_item();
    while let Some(secret) = list.next().await {
        let secret = secret?;
        println!("  secret={}", secret.name);
    }

    Ok(())
}
```

## 制限が付いたデフォルトの再試行ポリシーを構成する: 完全なコード

```rust
pub async fn client_retry_full(project_id: &str) -> crate::Result<()> {
    use std::time::Duration;
    use google_cloud_gax::paginator::ItemPaginator as _;
    use google_cloud_gax::retry_policy::{Aip194Strict, RetryPolicyExt};
    use google_cloud_secretmanager_v1 as secret_manager;

    let client = secret_manager::client::SecretManagerService::builder()
        .with_retry_policy(
            Aip194Strict
                .with_attempt_limit(5)
                .with_time_limit(Duration::from_secs(15)),
        )
        .build()
        .await?;

    let mut list = client
        .list_secrets()
        .set_parent(format!("projects/{project_id}"))
        .by_item();
    while let Some(secret) = list.next().await {
        let secret = secret?;
        println!("  secret={}", secret.name);
    }

    Ok(())
}
```

## 1つのリクエストに対して再試行ポリシーを上書きする: 完全なコード

```rust
use google_cloud_secretmanager_v1 as secret_manager;

pub async fn request_retry(
    client: &secret_manager::client::SecretManagerService,
    project_id: &str,
    secret_id: &str,
) -> crate::Result<()> {
    use std::time::Duration;
    use google_cloud_gax::options::RequestOptionsBuilder;
    use google_cloud_gax::retry_policy::{AlwaysRetry, RetryPolicyExt};

    client
        .delete_secret()
        .set_name(format!("projects/{project_id}/secrets/{secret_id}"))
        .with_retry_policy(
            AlwaysRetry
                .with_attempt_limit(5)
                .with_time_limit(Duration::from_secs(15)),
        )
        .send()
        .await?;

    Ok(())
}
```
