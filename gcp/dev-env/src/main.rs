use google_cloud_gax::paginator::ItemPaginator as _;
use google_cloud_secretmanager_v1::client::SecretManagerService;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
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
