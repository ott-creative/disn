use disn::CONFIG;
use std::time::Duration;
use tokio::time::sleep;

mod helpers;

#[tokio::test]
async fn vc_api_works() {
    let endpoint = helpers::spawn_app().await;

    sleep(Duration::from_millis(1000)).await;
    let client = reqwest::Client::new();
    // Act
    // create did first
    let response = client
        .post(format!("{}/did/create", endpoint))
        .header("x-api-key", &CONFIG.did.api_key)
        .send()
        .await
        .expect("Failed to execute request.");
    // Assert
    assert!(response.status().is_success());
    let did = response.text().await.unwrap();
    println!("created did:{}", did);

    // create vc issuer
    let response = client
        .post(format!("{}/vc/issuer/{}/?create", endpoint, did))
        .header("x-api-key", &CONFIG.did.api_key)
        .send()
        .await
        .expect("Failed to execute request.");
    assert!(response.status().is_success());
    println!("vc issuer created");
}
