# 2. 開発環境の準備

<https://googleapis.github.io/google-cloud-rust/setting_up_your_development_environment.html>

次のツールをインストールすることで、Google Cloudで[Rust](https://www.rust-lang.org/)アプリ開発と配置用の環境を準備します。

## Rustのインストール

1. Rustをインストールするために、[始めるには](https://www.rust-lang.org/learn/get-started)を参照してください。
2. Rustの最も最新のバージョンがインストールされていることを確認してください。

```sh
cargo --version
```

## エディタのインストール

[始めるには](https://www.rust-lang.org/learn/get-started)ガイドは、人気のあるエディタープラグインとIDEのリンクがあり、次の機能を提供します。

- 完全に統合されたデバック機能
- シンタックスハイライト
- コード補完

## Google Cloud CLIのインストール

[Google Cloud CLI](https://cloud.google.com/sdk/)はGoogle Cloud用のツールのセットです。
それは、コマンドラインからCompute Engine、Cloud Storage、BigQueryそして他のサービスにアクセスするために使用する[gcloud](https://cloud.google.com/sdk/gcloud/)と[bq](https://cloud.google.com/bigquery/docs/bq-command-line-tool)コマンドを含んでいます。
対話的にまたは自動的なスクリプトでこれらのツールを実行できます。

gcloud CLIをインストールするために[gcloud CLIのインストール](https://cloud.google.com/sdk/install)を参照してください。

## 新しいプロジェクトにCloud Client Libraries for Rustをインストールする

Cloud Client Libraries for Rustは、Secret ManagerやWorkflowのようなGoogle Cloud Serviceと統合するRust
開発者にとって慣例的な方法です。

例えば、Secret Manager APIのように、個々のAPI用のパッケージを使用するために次をします。

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

> 上記を実行したとき、次のエラーが表示されたら、GCPでプロジェクトを作成（既存のプロジェクトを使用する場合は作成しなくてもよい）して、プロジェクトのIDを控えておく。
> なお、プロジェクトは支払い（billing）が有効になっていないと、次で実行するSecrete Manager APIの有効かに失敗する。
>
> ```text
> ERROR: (gcloud.services.enable) The required property [project] is not currently set.
> ```
>
> そして、次のコマンドを実行する。
>
> ```sh
> gcloud config set project <project-id>
> ```

もし、Secret Manager APIが有効になっていない場合、次のコマンドを実行することで[APIとサービス](https://console.cloud.google.com/apis)でそれが有効になります。

```sh
gcloud services enable secretmanager.googleapis.com
```

> 上記を実行したときに次のエラーが発生する可能性がある。
>
> ```text
> ERROR: (gcloud.services.enable) You do not currently have an active account selected.
> ```
>
> この場合、次のコマンドを実行するとブラウザに認証ページが表示される。
> この認証ページでGoogleアカウントを選択して認証を受ける。
>
> ```sh
> gcloud auth login
> ```

4.新しいプロジェクトに[google-cloud-gax](https://crates.io/crates/google-cloud-gax)クレートを追加します。

```sh
cargo add google-cloud-gax
```

5.新しいプロジェクトに[tokio](https://crates.io/crates/tokio)クレートを追加します。

```sh
cargo add tokio --features macros
```

6.Secret Managerクライアントライブラリを使用するために、プロジェクトの`src/main.rs`を編集します。

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

6.プログラムをビルドします。

```sh
cargo build
```

プログラムはエラーなしでビルドされるはずです。

注意: Cloud Client Libraries for Rustのソースは[GitHub](https://github.com/googleapis/google-cloud-rust)にあります。

### プログラムの実行

1.ローカルな開発環境でCloud Client Librariesを使用するために、アプリケーションデフォルトクレデンシャルを準備します。

```sh
gcloud auth application-default login
```

詳細は[クライアントライブラリ使用の認証](https://cloud.google.com/docs/authentication/client-libraries)を参照してください。

> 上記を実行すると、`${HOME}/.config/gcloud/application_default_credentials.json`ファイルに、上記で選択したプロジェクトとアカウントを紐づけたクレデンシャルが記録される。

2.プログラムを実行し、Google Cloud PlatformのプロジェクトIDを提供します。

```sh
PROJECT_ID=$(gcloud config get project)
cargo run ${PROJECT_ID}
```

プログラムはプロジェクトIDに関連したシークレットを出力します。
もし、シークレットを確認できない場合、Secret Managerに何もないのかもしれません。
[シークレットを作成](https://cloud.google.com/secret-manager/docs/creating-and-accessing-secrets)し、プログラムを再実行すると、出力にシークレットを確認出来るはずです。

> シークレットは次の通り作成する。なお作成するシークレットの名前は`fullname`で、シークレットの値は`kuroyasu`である。
>
> ```sh
> gcloud secrets create fullname
> echo -n "kuroyasu" | gcloud secrets versions add fullname --data-file=-
> ```
>
> - `--data-file=-`は標準入力を意味する。
> - `echo -n`を使うことで改行が末尾に入らないようにする。

## 次は何をするべきか

- [Googleによる認証方法](https://cloud.google.com/docs/authentication)の探求
- [Google Cloudプロダクトのドキュメント](https://cloud.google.com/products)の閲覧
