/// Integration test for complete analysis flow
/// Tests Requirements: 1.1, 2.6, 3.6, 5.6, 6.3
/// 
/// This test verifies:
/// - File upload through API
/// - Static analysis execution
/// - Dynamic analysis execution
/// - Verdict generation
/// - Result streaming via WebSocket

use anyhow::Result;
use futures_util::{SinkExt, StreamExt};
use reqwest::multipart;
use serde_json::json;
use std::time::Duration;
use tokio::time::timeout;
use tokio_tungstenite::{connect_async, tungstenite::Message};
use uuid::Uuid;

const API_BASE_URL: &str = "http://localhost:8080";
const WS_BASE_URL: &str = "ws://localhost:8080";

#[tokio::test]
#[ignore] // Run with: cargo test --test complete_analysis_flow -- --ignored
async fn test_complete_analysis_flow() -> Result<()> {
    println!("Starting complete analysis flow test...");

    // Step 1: Authenticate and get JWT token
    let token = authenticate_test_user().await?;
    println!("✓ Authentication successful");

    // Step 2: Create test binary file
    let test_file = create_test_binary();
    println!("✓ Test binary created");

    // Step 3: Upload file through API
    let artifact_id = upload_file(&token, test_file).await?;
    println!("✓ File uploaded, artifact_id: {}", artifact_id);

    // Step 4: Establish WebSocket connection for result streaming
    let ws_url = format!("{}/api/v1/ws", WS_BASE_URL);
    let (ws_stream, _) = connect_async(&ws_url).await?;
    let (mut write, mut read) = ws_stream.split();
    println!("✓ WebSocket connection established");

    // Step 5: Subscribe to artifact updates
    let subscribe_msg = json!({
        "type": "subscribe",
        "artifact_id": artifact_id.to_string()
    });
    write
        .send(Message::Text(subscribe_msg.to_string()))
        .await?;
    println!("✓ Subscribed to artifact updates");

    // Step 6: Wait for static analysis progress update
    let static_progress = wait_for_progress_update(&mut read, "static_analysis").await?;
    println!("✓ Static analysis progress: {:?}", static_progress);

    // Step 7: Verify static analysis completion
    verify_static_analysis_complete(&token, &artifact_id).await?;
    println!("✓ Static analysis completed");

    // Step 8: Wait for dynamic analysis progress update
    let dynamic_progress = wait_for_progress_update(&mut read, "dynamic_analysis").await?;
    println!("✓ Dynamic analysis progress: {:?}", dynamic_progress);

    // Step 9: Verify dynamic analysis completion
    verify_dynamic_analysis_complete(&token, &artifact_id).await?;
    println!("✓ Dynamic analysis completed");

    // Step 10: Wait for verdict via WebSocket
    let verdict = wait_for_verdict(&mut read).await?;
    println!("✓ Verdict received: {:?}", verdict);

    // Step 11: Verify verdict was stored in database
    verify_verdict_stored(&token, &artifact_id).await?;
    println!("✓ Verdict stored in database");

    // Step 12: Verify verdict contains required fields
    assert!(verdict.get("verdict").is_some(), "Verdict missing 'verdict' field");
    assert!(verdict.get("risk_score").is_some(), "Verdict missing 'risk_score' field");
    assert!(verdict.get("evidence").is_some(), "Verdict missing 'evidence' field");
    println!("✓ Verdict contains all required fields");

    println!("\n✅ Complete analysis flow test PASSED");
    Ok(())
}

async fn authenticate_test_user() -> Result<String> {
    // In a real test, this would authenticate with the OIDC provider
    // For integration testing, we'll use a test token or mock authentication
    Ok("test_jwt_token".to_string())
}

fn create_test_binary() -> Vec<u8> {
    // Create a minimal PE executable for testing
    // This is a simplified PE header that can be parsed
    let mut binary = Vec::new();
    
    // DOS header
    binary.extend_from_slice(b"MZ"); // DOS signature
    binary.extend_from_slice(&[0u8; 58]); // DOS stub
    binary.extend_from_slice(&[0x80, 0x00, 0x00, 0x00]); // PE header offset
    
    // DOS stub program
    binary.extend_from_slice(&[0u8; 64]);
    
    // PE signature
    binary.extend_from_slice(b"PE\0\0");
    
    // COFF header
    binary.extend_from_slice(&[0x4c, 0x01]); // Machine (i386)
    binary.extend_from_slice(&[0x01, 0x00]); // Number of sections
    binary.extend_from_slice(&[0u8; 12]); // Timestamp and symbol table
    binary.extend_from_slice(&[0xE0, 0x00]); // Size of optional header
    binary.extend_from_slice(&[0x02, 0x01]); // Characteristics
    
    // Optional header
    binary.extend_from_slice(&[0x0B, 0x01]); // Magic (PE32)
    binary.extend_from_slice(&[0u8; 222]); // Rest of optional header
    
    // Section header
    binary.extend_from_slice(b".text\0\0\0"); // Section name
    binary.extend_from_slice(&[0u8; 32]); // Section data
    
    // Add some suspicious strings for detection
    binary.extend_from_slice(b"CreateRemoteThread\0");
    binary.extend_from_slice(b"VirtualAllocEx\0");
    
    binary
}

async fn upload_file(token: &str, file_data: Vec<u8>) -> Result<Uuid> {
    let client = reqwest::Client::new();
    
    let form = multipart::Form::new()
        .part(
            "file",
            multipart::Part::bytes(file_data)
                .file_name("test_binary.exe")
                .mime_str("application/x-msdownload")?,
        );
    
    let response = client
        .post(format!("{}/api/v1/artifacts/upload", API_BASE_URL))
        .header("Authorization", format!("Bearer {}", token))
        .multipart(form)
        .send()
        .await?;
    
    if !response.status().is_success() {
        anyhow::bail!("Upload failed: {}", response.status());
    }
    
    let body: serde_json::Value = response.json().await?;
    let artifact_id = body["artifact_id"]
        .as_str()
        .ok_or_else(|| anyhow::anyhow!("Missing artifact_id"))?;
    
    Ok(Uuid::parse_str(artifact_id)?)
}

async fn wait_for_progress_update(
    read: &mut futures_util::stream::SplitStream<
        tokio_tungstenite::WebSocketStream<
            tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>,
        >,
    >,
    expected_stage: &str,
) -> Result<serde_json::Value> {
    let timeout_duration = Duration::from_secs(60);
    
    let result = timeout(timeout_duration, async {
        while let Some(msg) = read.next().await {
            if let Ok(Message::Text(text)) = msg {
                if let Ok(json) = serde_json::from_str::<serde_json::Value>(&text) {
                    if json["type"] == "progress" && json["stage"] == expected_stage {
                        return Ok(json);
                    }
                }
            }
        }
        Err(anyhow::anyhow!("WebSocket stream ended"))
    })
    .await??;
    
    Ok(result)
}

async fn wait_for_verdict(
    read: &mut futures_util::stream::SplitStream<
        tokio_tungstenite::WebSocketStream<
            tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>,
        >,
    >,
) -> Result<serde_json::Value> {
    let timeout_duration = Duration::from_secs(120);
    
    let result = timeout(timeout_duration, async {
        while let Some(msg) = read.next().await {
            if let Ok(Message::Text(text)) = msg {
                if let Ok(json) = serde_json::from_str::<serde_json::Value>(&text) {
                    if json["type"] == "verdict" {
                        return Ok(json);
                    }
                }
            }
        }
        Err(anyhow::anyhow!("WebSocket stream ended"))
    })
    .await??;
    
    Ok(result)
}

async fn verify_static_analysis_complete(token: &str, artifact_id: &Uuid) -> Result<()> {
    let client = reqwest::Client::new();
    
    // Poll for static analysis report
    for _ in 0..30 {
        let response = client
            .get(format!(
                "{}/api/v1/artifacts/{}/static-report",
                API_BASE_URL, artifact_id
            ))
            .header("Authorization", format!("Bearer {}", token))
            .send()
            .await?;
        
        if response.status().is_success() {
            let report: serde_json::Value = response.json().await?;
            if report.get("static_score").is_some() {
                return Ok(());
            }
        }
        
        tokio::time::sleep(Duration::from_secs(1)).await;
    }
    
    anyhow::bail!("Static analysis did not complete within timeout")
}

async fn verify_dynamic_analysis_complete(token: &str, artifact_id: &Uuid) -> Result<()> {
    let client = reqwest::Client::new();
    
    // Poll for behavioral report
    for _ in 0..60 {
        let response = client
            .get(format!(
                "{}/api/v1/artifacts/{}/behavioral-report",
                API_BASE_URL, artifact_id
            ))
            .header("Authorization", format!("Bearer {}", token))
            .send()
            .await?;
        
        if response.status().is_success() {
            let report: serde_json::Value = response.json().await?;
            if report.get("behavioral_score").is_some() {
                return Ok(());
            }
        }
        
        tokio::time::sleep(Duration::from_secs(1)).await;
    }
    
    anyhow::bail!("Dynamic analysis did not complete within timeout")
}

async fn verify_verdict_stored(token: &str, artifact_id: &Uuid) -> Result<()> {
    let client = reqwest::Client::new();
    
    let response = client
        .get(format!("{}/api/v1/verdicts/{}", API_BASE_URL, artifact_id))
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await?;
    
    if !response.status().is_success() {
        anyhow::bail!("Failed to retrieve verdict: {}", response.status());
    }
    
    let verdict: serde_json::Value = response.json().await?;
    
    if verdict.get("verdict").is_none() {
        anyhow::bail!("Verdict not properly stored");
    }
    
    Ok(())
}

#[tokio::test]
#[ignore]
async fn test_analysis_flow_with_reconnection() -> Result<()> {
    println!("Testing analysis flow with WebSocket reconnection...");
    
    let token = authenticate_test_user().await?;
    let test_file = create_test_binary();
    let artifact_id = upload_file(&token, test_file).await?;
    
    // Connect, disconnect, and reconnect to test buffering
    let ws_url = format!("{}/api/v1/ws", WS_BASE_URL);
    let (ws_stream, _) = connect_async(&ws_url).await?;
    let (mut write, mut read) = ws_stream.split();
    
    let subscribe_msg = json!({
        "type": "subscribe",
        "artifact_id": artifact_id.to_string()
    });
    write.send(Message::Text(subscribe_msg.to_string())).await?;
    
    // Wait for first progress update
    wait_for_progress_update(&mut read, "static_analysis").await?;
    
    // Simulate disconnection
    drop(write);
    drop(read);
    println!("✓ Simulated disconnection");
    
    // Wait a bit for buffering
    tokio::time::sleep(Duration::from_secs(5)).await;
    
    // Reconnect
    let (ws_stream, _) = connect_async(&ws_url).await?;
    let (mut write, mut read) = ws_stream.split();
    
    write.send(Message::Text(subscribe_msg.to_string())).await?;
    println!("✓ Reconnected");
    
    // Should receive buffered updates
    let mut received_updates = 0;
    let timeout_duration = Duration::from_secs(10);
    
    let _ = timeout(timeout_duration, async {
        while let Some(msg) = read.next().await {
            if let Ok(Message::Text(text)) = msg {
                if let Ok(json) = serde_json::from_str::<serde_json::Value>(&text) {
                    if json["type"] == "progress" || json["type"] == "verdict" {
                        received_updates += 1;
                        println!("✓ Received buffered update: {:?}", json["type"]);
                    }
                }
            }
        }
    })
    .await;
    
    assert!(
        received_updates > 0,
        "Should have received buffered updates after reconnection"
    );
    
    println!("\n✅ Reconnection test PASSED");
    Ok(())
}
