use google_cloud_compute_v1::client::Instances;
use google_cloud_gax::paginator::ItemPaginator as _;

const PROJECT_ID: &str = "gcp-for-rust";
const ZONE: &str = "us-central1-a";

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let client = Instances::builder().build().await?;
    println!("Listing instances for project: {PROJECT_ID}");
    let mut instances = client
        .list()
        .set_project(PROJECT_ID)
        .set_zone(ZONE)
        .by_item();
    while let Some(item) = instances.next().await.transpose()? {
        println!("  {:?}", item.name);
    }
    println!("DONE");

    Ok(())
}
