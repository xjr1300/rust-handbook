# 10. エラー処理

<https://googleapis.github.io/google-cloud-rust/error_handling.html>

時々、アプリケーションは、クライアントライブラリによって返されたエラーの型と詳細に基づいて分岐させる必要があります。
このガイドは、そのようなエラーを処理するコードを記述する方法を紹介します。

> **再試行可能エラー:** 分散システムでエラーを処理する最も一般的な理由のひとつは、一時的なエラーのために失敗したリクエストを再試行することです。
> Google Cloud Client Libraries for Rustは、ポリシーに基づく再試行ループを実装しています。
> 再試行ループを有効にするためにポリシーを構成することのみを必要とし、そしてライブラリが一般的な再試行ポリシーを実装します。
> 独自の再試行ループを実装する前に、[再試行ポリシーを構成する](https://googleapis.github.io/configuring_retry_policies.html)を参照してください。

## 事前条件

このガイドは、[Secret Manager](https://cloud.google.com/secret-manager)サービスを使用して、エラー処理を説明しますが、この概念は他のサービスにも十分に適用できます。

Secret Managerサービスの[クイックスタート](https://cloud.google.com/secret-manager/docs/quickstart)を完了しておくことをお勧めします。
そのガイドでは、サービスを有効にし、ログインしていること、アカウントに必要な権限があることを確認するために必要な手順について説明します。

Rustライブラリの完全なセットアップ手順については、[開発環境の準備](https://googleapis.github.io/google-cloud-rust/setting_up_your_development_environment.html)を参照してください。

## 依存関係

`Cargo.toml`ファイルにSecret Managerライブラリを追加してください。

```sh
cargo add google-cloud-secretmanager-v1
```

加えて、このガイドはチェックサムを計算するために`crc32c`を使用します。

```sh
cargo add crc32c
```

## 動機

このガイドでは、新しい[シークレットバージョン](https://cloud.google.com/secret-manager/docs/add-secret-version)を作成します。
シークレットバージョンは[シークレット](https://cloud.google.com/secret-manager/docs/creating-and-accessing-secrets)内に含まれます。
シークレットバージョンを追加する前に、シークレットを作成する必要があります。
クラウドサービスにおける一般的なパターンは、リソースをそのコンテナが存在するかのように使用し、エラーが発生した場合にのみコンテナを作成することです。
もし、ほとんどのときにコンテナが存在する場合、そのような手法はリクエストをする前にコンテナが存在するかどうか確認するよりも効率的です。
コンテナが存在するかを確認することは、よりクォータを消費し、よりRPC料金が増加する結果となり、コンテナがすでに存在する場合は遅くなります。

## エラー処理

このセクションは、シークレットを更新する関数を説明します。
完全なコード例は、[update_secret](https://googleapis.github.io/google-cloud-rust/error_handling.html#update_secret)を参照してください。

まず、新しいシークレットバージョンを作成することを試みます。

```rust
match update_attempt(&client, project_id, secret_id, data.clone()).await {
```

もし、[update_attempt](https://googleapis.github.io/google-cloud-rust/error_handling.html#update_attempt)が成功した場合、単に成功結果をプリントして戻ることができます。

```rust
Ok(version) => {
    println!("new version is {}", version.name);
    Ok(version)
}
```

リクエストを完全に送信する前に接続が切断したり、レスポンスを受け取る前に接続が切断したり、認証トークンを作成することができなかった場合など、リクエストは多くの理由で失敗するかもしれません。

再試行ポリシーはこれらエラーのほとんどを扱うことができます。
ここでは、サービスによって返されたエラーのみに興味があります。

```rust
Err(e) => {
    if let Some(status) = e.status() {
```

そして、シークレットがないことに対応するエラーは次のようになります。

```rust
use gax::error::rpc::Code;
if status.code == Code::NotFound {
```

もし、これが"not found"エラーの場合、シークレットの作成を試すことができます。
その後、元のエラーを返します。

```rust
let _ = create_secret(&client, project_id, secret_id).await?;
```

[create_secret](https://googleapis.github.io/google-cloud-rust/error_handling.html#create_secret)が成功したと仮定すると、再度シークレットバージョンを追加することを試すことができ、もし何かが失敗したら、今回は単にエラーが返されます。

```rust
let version = update_attempt(&client, project_id, secret_id, data).await?;
println!("new version is {}", version.name);
return Ok(version);
```

## 次に学ぶこと

エラー処理をより学ぶために次を参照してください。

- [エラーの詳細を調べる](https://googleapis.github.io/google-cloud-rust/examine_error_details.html)
- [バインディングエラーの処理](https://googleapis.github.io/google-cloud-rust/binding_errors.html)

---

## コード例

### `update_secret`

```rust
pub async fn update_secret(
    project_id: &str,
    secret_id: &str,
    data: Vec<u8>,
) -> crate::Result<sm::model::SecretVersion> {
    let client = sm::client::SecretManagerService::builder().build().await?;

    match update_attempt(&client, project_id, secret_id, data.clone()).await {
        Ok(version) => {
            println!("new version is {}", version.name);
            Ok(version)
        }
        Err(e) => {
            if let Some(status) = e.status() {
                use gax::error::rpc::Code;
                if status.code == Code::NotFound {
                    let _ = create_secret(&client, project_id, secret_id).await?;
                    let version = update_attempt(&client, project_id, secret_id, data).await?;
                    println!("new version is {}", version.name);
                    return Ok(version);
                }
            }
            Err(e.into())
        }
    }
}
```

### `update_attempt`

```rust
async fn update_attempt(
    client: &sm::client::SecretManagerService,
    project_id: &str,
    secret_id: &str,
    data: Vec<u8>,
) -> gax::Result<sm::model::SecretVersion> {
    let checksum = crc32c::crc32c(&data) as i64;
    client
        .add_secret_version()
        .set_parent(format!("projects/{project_id}/secrets/{secret_id}"))
        .set_payload(
            sm::model::SecretPayload::new()
                .set_data(data)
                .set_data_crc32c(checksum),
        )
        .send()
        .await
}
```

### `create_secret`

```rust
pub async fn create_secret(
    client: &sm::client::SecretManagerService,
    project_id: &str,
    secret_id: &str,
) -> gax::Result<sm::model::Secret> {
    use google_cloud_gax::options::RequestOptionsBuilder;
    use google_cloud_gax::retry_policy::AlwaysRetry;
    use google_cloud_gax::retry_policy::RetryPolicyExt;
    use std::time::Duration;

    client
        .create_secret()
        .set_parent(format!("projects/{project_id}"))
        .with_retry_policy(
            AlwaysRetry
                .with_attempt_limit(5)
                .with_time_limit(Duration::from_secs(15)),
        )
        .set_secret_id(secret_id)
        .set_secret(
            sm::model::Secret::new()
                .set_replication(sm::model::Replication::new().set_replication(
                    sm::model::replication::Replication::Automatic(
                        sm::model::replication::Automatic::new().into(),
                    ),
                ))
                .set_labels([("integration-test", "true")]),
        )
        .send()
        .await
}
```
