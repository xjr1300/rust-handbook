# Cloud ShellでRustを設定する

<https://googleapis.github.io/google-cloud-rust/setting_up_rust_on_cloud_shell.html>

Cloud Shellは、小さな例とテストを実行する素晴らしい環境です。
このガイドは、Cloud ShellにRustを設置し、Cloud Clientライブラリの1つをインストールする方法を紹介します。

## Cloud Shellの開始

1.Google Cloud Consoleの[プロジェクトセレクター](https://console.cloud.google.com/projectselector2/home/dashboard)で、プロジェクトを選択します。
2.新しいシェルを開始するために<https://shell.cloud.google.com/>を開きます。Google Cloud APIの呼び出しでクレデンシャルを使用するために[Cloud Shellの認証](https://cloud.google.com/shell/docs/auth)が促されるかもしれません。

## Rustの設定

1.[Cloud Shell](https://cloud.google.com/shell)はプリインストールされた[rustup](https://rust-lang.github.io/rustup/)を含んで起動します。Rustのデフォルトバーションをインストールして設定するためにそれを使用できます。

```sh
rustup default stable
```

2.Rustの最も最新バージョンがインストールされているか確認してください。

```sh
cargo version
```

## Cloud ShellにRustクライアントライブラリをインストールする

1.新しいRustプロジェクトを作成します。

```sh
cargo new my-project
```

2.新しいプロジェクトにディレクトリを変更します。

```sh
cd my-project
```

3.新しいプロジェクトに[Secret Manager](https://cloud.google.com/secret-manager/docs/overview)クライアントライブラリを追加します。

```sh
cargo add google-cloud-secretmanager-v1
```

4.新しいプロジェクトに[google-cloud-gax](https://crates.io/crates/google-cloud-gax)クレートを追加します。

```sh
cargo add google-cloud-gax
```

5.新しいプロジェクトに[tokio](https://crates.io/crates/tokio)クレートを追加します。

```sh
cargo add tokio --features="macros"
```

6.Secret Managerクライアントライブラリを使用するために、プロジェクト内の`src/main.rs`を編集します。

```rust
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    use google_cloud_gax::paginator::ItemPaginator as _;
    use google_cloud_secretmanager_v1::client::SecretManagerService;
    let project_id = std::env::args().nth(1).unwrap();
    let client = SecretManagerService::builder().build().await?;

    let mut items = client
        .list_secrets()
        .set_parent(format!("projects/{project_id}"))
        .by_item();
    while let Some(item) = items.next().await {
        println!("{}", item?.name);
    }
    Ok(())
}
```

7.Google Cloud PlatformのプロジェクトIDを提供して、プログラムを実行します。

```sh
PROJECT_ID=$(gcloud config get project)
cargo run ${PROJECT_ID}
```

> 上記方法で環境変数`PROJECT_ID`にプロジェクトIDを設定する場合、Compute Engineにプロジェクトとアカウントのクレデンシャルが保存される([開発環境の準備](02-setting-up-your-development-environment.md#新しいプロジェクトにcloud-client-libraries-for-rustをインストールする))。
> したがって、プロジェクトIDを直接`cargo run`の引数に与えて実行することを推奨する。

プログラムはプロジェクトIDに関連したシークレットを出力します。
もし、シークレットを確認できない場合、Secret Managerに何もないのかもしれません。
[シークレットを作成](https://cloud.google.com/secret-manager/docs/creating-and-accessing-secrets)し、プログラムを再実行すると、出力にシークレットを確認出来るはずです。
