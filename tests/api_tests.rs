use dm_rust::run_app;
use tokio::time::{sleep, Duration};
use reqwest;

const ROOT_URL: &str = "http://localhost:18081";
const API_URL: &str = "http://localhost:18081/lspcapi";

#[tokio::test]
async fn test_app_lifecycle_and_api() {
    // 1. Start Server in Background
    let config_path = "config.integration.toml";
    
    // Validate config exists
    assert!(std::path::Path::new(config_path).exists(), "Integration config missing");

    let server_handle = tokio::spawn(async move {
        // Run with "error" log level to keep test output clean
        if let Err(e) = run_app(config_path, "error").await {
            eprintln!("Runtime Error: {:?}", e);
        }
    });

    // Give it time to start
    sleep(Duration::from_secs(2)).await;

    // 2. Test: Health Check (Root)
    let client = reqwest::Client::new();
    let resp = client.get(ROOT_URL)
        .send()
        .await;

    
    match resp {
        Ok(response) => {
            assert!(response.status().is_success(), "Root endpoint should return 200 OK");
        }
        Err(e) => {
            panic!("Failed to connect to server: {}. Is it running?", e);
        }
    }

    // 3. Test: Get All Node States
    let resp = client.post(format!("{}/device/getAllNodeStates", API_URL))
        .send()
        .await
        .expect("Failed to call getAllNodeStates");
    
    assert!(resp.status().is_success());
    let body = resp.text().await.expect("Failed to read body");
    // Verify JSON structure (simple check)
    assert!(body.contains("code"), "Response should contain 'code'");

    // 4. Test: Mock Protocol interaction (Node 1)
    // Read Node 1
    let resp = client.post(format!("{}/device/read", API_URL))
        .json(&serde_json::json!({ "id": 1 }))
        .send()
        .await
        .expect("Failed to read device");
    assert!(resp.status().is_success());

    // 5. Shutdown (Optional clean up)
    // tokio test runtime will kill the spawn when main future finishes
    server_handle.abort();
}
