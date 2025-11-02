use std::error::Error as _;

use gax::error::binding::BindingError;
use google_cloud_gax as gax;
use google_cloud_secretmanager_v1::client::SecretManagerService;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let client = SecretManagerService::builder().build().await?;
    let secret = client
        .get_secret()
        // .set_name("projects/my-project/secrets/my-secret")
        .send()
        .await;
    let e = secret.unwrap_err();
    assert!(e.is_binding(), "{e:?}");
    assert!(e.source().is_some(), "{e:?}");
    let e = e
        .source()
        .and_then(|e| e.downcast_ref::<BindingError>())
        .expect("should be a BindingError");
    println!("{e:#?}");

    Ok(())
}
