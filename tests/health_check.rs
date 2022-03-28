use std::time::Duration;
use tokio::time::sleep;

mod helpers;

#[tokio::test]
async fn health_check_works() {
    let endpoint = helpers::spawn_app().await;

    sleep(Duration::from_millis(1000)).await;
    println!("Delay expired");
    let client = reqwest::Client::new();
    // Act
    let response = client
        .get(format!("{}/health_check", endpoint))
        .send()
        .await
        .expect("Failed to execute request.");
    // Assert
    assert!(response.status().is_success());
    assert_eq!(Some(0), response.content_length());
}
