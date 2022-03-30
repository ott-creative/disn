use disn::configuration::get_configuration;
use std::time::Duration;
use tokio::time::sleep;

mod helpers;

#[tokio::test]
async fn did_api_works() {
    let configuration = get_configuration().unwrap();
    let endpoint = helpers::spawn_app().await;

    sleep(Duration::from_millis(1000)).await;
    let client = reqwest::Client::new();
    // Act
    let response = client
        .post(format!("{}/did/create", endpoint))
        .header("x-api-key", &configuration.did.api_key)
        .send()
        .await
        .expect("Failed to execute request.");
    // Assert
    assert!(response.status().is_success());

    let did = response.text().await.unwrap();
    println!("created did:{}", did);

    let response = client
        .get(format!("{}/did/resolve/{}", endpoint, did))
        .header("x-api-key", &configuration.did.api_key)
        .send()
        .await
        .expect("Failed to execute request.");
    assert!(response.status().is_success());
    println!(
        "did resolve content length:{:?}",
        response.content_length().unwrap()
    );

    println!("did document:{}", response.text().await.unwrap());

    // incorrect api key should fail
    let response = client
        .post(format!("{}/did/create", endpoint))
        .header("x-api-key", " ".to_string())
        .send()
        .await
        .expect("Failed to execute request.");
    // Assert
    assert!(!response.status().is_success());
}
