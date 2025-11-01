# 7. Compute Engine APIを使用する

<https://googleapis.github.io/google-cloud-rust/compute.html>

[Compute Engine](https://cloud.google.com/compute) APIは、Google Cloudで仮想マシン（VM）を作成または実行できるようにします。

このガイドは、Rust用のCompute Engineクライアントライブラリを初期化して、ライブラリを使用していくつかの基本的な操作を実行する方法を紹介します。

## 事前条件

このガイドは、[支払いが有効](https://cloud.google.com/billing/docs/how-to/verify-billing-enabled#confirm_billing_is_enabled_on_a_project)な[Google Cloudプロジェクト](https://cloud.google.com/resource-manager/docs/creating-managing-projects)があることを想定しています。

## 依存関係にクライアントライブラリを追加する

`cargo`を使用して、必要な依存関係を追加します。

```sh
cargo add google-cloud-compute-v1
```

## すべての仮想マシンをリストする

仮想マシンを作成そして操作するクライアントは**インスタンス**と呼ばれます。
`list()`関数を使用して、プロジェクト内のすべての仮想マシンをリストできます。

```rust
pub async fn quickstart(project_id: &str) -> anyhow::Result<()> {
    use google_cloud_compute_v1::client::Instances;
    use google_cloud_gax::paginator::ItemPaginator;

    const ZONE: &str = "us-central1-a";

    let client = Instances::builder().build().await?;
    println!("Listing instances for project {project_id}");
    let mut instances = client
        .list()
        .set_project(project_id)
        .set_zone(ZONE)
        .by_item();
    while let Some(item) = instances.next().await.transpose()? {
        println!("  {:?}", item.name);
    }
    println!("DONE");
    Ok(())
}
```

## 次のステップ

- [Compute Engineクライアントライブラリ](https://cloud.google.com/compute/docs/api/libraries)
- [長い時間がかかる操作を実行する](https://googleapis.github.io/working_with_long_running_operations.html)
- [問い合わせポリシーの設定](https://googleapis.github.io/configuring_polling_policies.html)
