use disn::configuration::get_configuration;
use disn::handlers::passbase::HookData;
use std::time::Duration;
use tokio::time::sleep;

mod helpers;

#[tokio::test]
async fn passbase_hook_works() {
    //let configuration = get_configuration().unwrap();
    let endpoint = helpers::spawn_app().await;

    sleep(Duration::from_millis(1000)).await;
    let client = reqwest::Client::new();
    // Act
    // create did first
    let response = client
        .post(format!("{}/passbase", endpoint))
        .json(&HookData {
            event: "VERIFICATION_COMPLETED".to_string(),
            key: "ddd2a2b6-db6a-4967-8a46-ebe33d738ff7".to_string(),
            status: "approved".to_string(),
            created: 1648712399,
            updated: 1648712399,
            processed: 1648712399,
        })
        .send()
        .await
        .expect("Failed to execute request.");
    // Assert
    assert!(response.status().is_success());
}
