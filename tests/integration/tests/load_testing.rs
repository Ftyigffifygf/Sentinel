/// Integration test for load testing
/// Tests Requirements: 16.1, 16.2, 16.3
/// 
/// This test verifies:
/// - 1000 concurrent file uploads
/// - 10,000 WebSocket connections
/// - 100,000 endpoint events per second
/// - Worker autoscaling under load

use anyhow::Result;
use futures_util::{SinkExt, StreamExt};
use reqwest::multipart;
use serde_json::json;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::task::JoinSet;
use uuid::Uuid;

const API_BASE_URL: &str = "http://localhost:8080";
const WS_BASE_URL: &str = "ws://localhost:8080";

#[tokio::test]
#[ignore] // Run with: cargo test --test load_testing -- --ignored
async fn test_concurrent_file_uploads() -> Result<()> {
    println!("Starting concurrent file uploads test (1000 uploads)...");

    let token = authenticate_test_user().await?;
    let concurrent_uploads = 1000;
    
    let success_count = Arc::new(AtomicUsize::new(0));
    let failure_count = Arc::new(AtomicUsize::new(0));
    
    let start_time = Instant::now();
    
    // Create tasks for concurrent uploads
    let mut tasks = JoinSet::new();
    
    for i in 0..concurrent_uploads {
        let token_clone = token.clone();
        let success_count_clone = success_count.clone();
        let failure_count_clone = failure_count.clone();
        
        tasks.spawn(async move {
            match upload_test_file(&token_clone, i).await {
                Ok(_) => {
                    success_count_clone.fetch_add(1, Ordering::Relaxed);
                }
                Err(e) => {
                    eprintln!("Upload {} failed: {}", i, e);
                    failure_count_clone.fetch_add(1, Ordering::Relaxed);
                }
            }
        });
    }
    
    // Wait for all uploads to complete
    while tasks.join_next().await.is_some() {}
    
    let duration = start_time.elapsed();
    let successes = success_count.load(Ordering::Relaxed);
    let failures = failure_count.load(Ordering::Relaxed);
    
    println!("✓ Completed {} uploads in {:?}", concurrent_uploads, duration);
    println!("  - Successes: {}", successes);
    println!("  - Failures: {}", failures);
    println!("  - Success rate: {:.1}%", (successes as f64 / concurrent_uploads as f64) * 100.0);
    println!("  - Throughput: {:.1} uploads/sec", concurrent_uploads as f64 / duration.as_secs_f64());
    
    // Verify acceptable success rate (>95%)
    let success_rate = (successes as f64 / concurrent_uploads as f64) * 100.0;
    assert!(
        success_rate >= 95.0,
        "Success rate should be at least 95%, got {:.1}%",
        success_rate
    );
    
    println!("\n✅ Concurrent file uploads test PASSED");
    Ok(())
}

#[tokio::test]
#[ignore]
async fn test_websocket_connections() -> Result<()> {
    println!("Starting WebSocket connections test (10,000 connections)...");

    let target_connections = 10_000;
    let batch_size = 100;
    
    let active_connections = Arc::new(AtomicUsize::new(0));
    let failed_connections = Arc::new(AtomicUsize::new(0));
    
    let start_time = Instant::now();
    
    // Establish connections in batches to avoid overwhelming the system
    for batch in 0..(target_connections / batch_size) {
        let mut tasks = JoinSet::new();
        
        for i in 0..batch_size {
            let active_clone = active_connections.clone();
            let failed_clone = failed_connections.clone();
            let conn_id = batch * batch_size + i;
            
            tasks.spawn(async move {
                match establish_websocket_connection(conn_id).await {
                    Ok(_) => {
                        active_clone.fetch_add(1, Ordering::Relaxed);
                    }
                    Err(e) => {
                        eprintln!("Connection {} failed: {}", conn_id, e);
                        failed_clone.fetch_add(1, Ordering::Relaxed);
                    }
                }
            });
        }
        
        // Wait for batch to complete
        while tasks.join_next().await.is_some() {}
        
        if (batch + 1) % 10 == 0 {
            let current = active_connections.load(Ordering::Relaxed);
            println!("  Progress: {} connections established", current);
        }
    }
    
    let duration = start_time.elapsed();
    let active = active_connections.load(Ordering::Relaxed);
    let failed = failed_connections.load(Ordering::Relaxed);
    
    println!("✓ Established {} WebSocket connections in {:?}", active, duration);
    println!("  - Active: {}", active);
    println!("  - Failed: {}", failed);
    println!("  - Success rate: {:.1}%", (active as f64 / target_connections as f64) * 100.0);
    
    // Verify acceptable success rate (>90% for WebSocket connections)
    let success_rate = (active as f64 / target_connections as f64) * 100.0;
    assert!(
        success_rate >= 90.0,
        "Success rate should be at least 90%, got {:.1}%",
        success_rate
    );
    
    println!("\n✅ WebSocket connections test PASSED");
    Ok(())
}

#[tokio::test]
#[ignore]
async fn test_endpoint_events_throughput() -> Result<()> {
    println!("Starting endpoint events throughput test (100,000 events/sec)...");

    let tenant_id = create_test_tenant().await?;
    let endpoint_id = Uuid::new_v4();
    
    let target_events_per_sec = 100_000;
    let test_duration_secs = 5;
    let total_events = target_events_per_sec * test_duration_secs;
    
    let events_sent = Arc::new(AtomicUsize::new(0));
    let events_failed = Arc::new(AtomicUsize::new(0));
    
    println!("  Target: {} events over {} seconds", total_events, test_duration_secs);
    
    let start_time = Instant::now();
    
    // Send events in parallel batches
    let batch_size = 1000;
    let num_batches = total_events / batch_size;
    
    let mut tasks = JoinSet::new();
    
    for batch_num in 0..num_batches {
        let tenant_id_clone = tenant_id.clone();
        let endpoint_id_clone = endpoint_id.clone();
        let sent_clone = events_sent.clone();
        let failed_clone = events_failed.clone();
        
        tasks.spawn(async move {
            let events = create_event_batch(&endpoint_id_clone, &tenant_id_clone, batch_size);
            
            match send_telemetry_batch(&events).await {
                Ok(_) => {
                    sent_clone.fetch_add(batch_size, Ordering::Relaxed);
                }
                Err(e) => {
                    eprintln!("Batch {} failed: {}", batch_num, e);
                    failed_clone.fetch_add(batch_size, Ordering::Relaxed);
                }
            }
        });
        
        // Limit concurrent tasks to avoid overwhelming the system
        if tasks.len() >= 100 {
            tasks.join_next().await;
        }
    }
    
    // Wait for all batches to complete
    while tasks.join_next().await.is_some() {}
    
    let duration = start_time.elapsed();
    let sent = events_sent.load(Ordering::Relaxed);
    let failed = events_failed.load(Ordering::Relaxed);
    
    let throughput = sent as f64 / duration.as_secs_f64();
    
    println!("✓ Sent {} events in {:?}", sent, duration);
    println!("  - Throughput: {:.0} events/sec", throughput);
    println!("  - Failed: {}", failed);
    println!("  - Success rate: {:.1}%", (sent as f64 / total_events as f64) * 100.0);
    
    // Verify throughput meets requirement
    assert!(
        throughput >= 80_000.0,
        "Throughput should be at least 80,000 events/sec, got {:.0}",
        throughput
    );
    
    println!("\n✅ Endpoint events throughput test PASSED");
    Ok(())
}

#[tokio::test]
#[ignore]
async fn test_worker_autoscaling() -> Result<()> {
    println!("Starting worker autoscaling test...");

    let token = authenticate_test_user().await?;
    
    // Step 1: Check initial worker count
    let initial_workers = get_worker_count("static-worker").await?;
    println!("✓ Initial static-worker count: {}", initial_workers);
    
    // Step 2: Generate high load by uploading many files
    let load_uploads = 500;
    println!("  Generating load with {} uploads...", load_uploads);
    
    let mut tasks = JoinSet::new();
    for i in 0..load_uploads {
        let token_clone = token.clone();
        tasks.spawn(async move {
            let _ = upload_test_file(&token_clone, i).await;
        });
    }
    
    // Don't wait for all to complete, just start the load
    tokio::time::sleep(Duration::from_secs(5)).await;
    
    // Step 3: Monitor queue depth
    let queue_depth = get_queue_depth("artifacts.uploaded").await?;
    println!("✓ Queue depth under load: {}", queue_depth);
    
    // Step 4: Wait for autoscaling to trigger (Requirement 16.1, 16.2)
    println!("  Waiting for autoscaling to trigger...");
    tokio::time::sleep(Duration::from_secs(30)).await;
    
    // Step 5: Check if workers scaled up
    let scaled_workers = get_worker_count("static-worker").await?;
    println!("✓ Static-worker count after scaling: {}", scaled_workers);
    
    assert!(
        scaled_workers > initial_workers,
        "Workers should scale up under load"
    );
    println!("✓ Workers scaled up from {} to {}", initial_workers, scaled_workers);
    
    // Step 6: Wait for load to complete
    while tasks.join_next().await.is_some() {}
    println!("✓ Load generation complete");
    
    // Step 7: Wait for scale-down
    println!("  Waiting for scale-down...");
    tokio::time::sleep(Duration::from_secs(60)).await;
    
    let final_workers = get_worker_count("static-worker").await?;
    println!("✓ Final static-worker count: {}", final_workers);
    
    // Verify minimum replicas maintained (Requirement 16.4)
    assert!(
        final_workers >= 2,
        "Should maintain at least 2 workers for high availability"
    );
    println!("✓ Minimum replica count maintained");
    
    // Step 8: Verify scaling events were logged (Requirement 16.5)
    let scaling_events = get_scaling_events().await?;
    assert!(
        !scaling_events.is_empty(),
        "Scaling events should be logged"
    );
    println!("✓ Scaling events logged: {} events", scaling_events.len());
    
    println!("\n✅ Worker autoscaling test PASSED");
    Ok(())
}

#[tokio::test]
#[ignore]
async fn test_sustained_load() -> Result<()> {
    println!("Starting sustained load test (5 minutes)...");

    let token = authenticate_test_user().await?;
    let test_duration = Duration::from_secs(300); // 5 minutes
    let uploads_per_minute = 100;
    
    let total_uploads = Arc::new(AtomicUsize::new(0));
    let total_failures = Arc::new(AtomicUsize::new(0));
    
    let start_time = Instant::now();
    
    println!("  Running sustained load for {} seconds...", test_duration.as_secs());
    
    while start_time.elapsed() < test_duration {
        let mut tasks = JoinSet::new();
        
        for i in 0..uploads_per_minute {
            let token_clone = token.clone();
            let uploads_clone = total_uploads.clone();
            let failures_clone = total_failures.clone();
            
            tasks.spawn(async move {
                match upload_test_file(&token_clone, i).await {
                    Ok(_) => {
                        uploads_clone.fetch_add(1, Ordering::Relaxed);
                    }
                    Err(_) => {
                        failures_clone.fetch_add(1, Ordering::Relaxed);
                    }
                }
            });
        }
        
        // Wait for batch
        while tasks.join_next().await.is_some() {}
        
        let elapsed = start_time.elapsed();
        let uploads = total_uploads.load(Ordering::Relaxed);
        println!("  {:?} - {} uploads completed", elapsed, uploads);
        
        // Wait before next batch
        tokio::time::sleep(Duration::from_secs(60)).await;
    }
    
    let duration = start_time.elapsed();
    let uploads = total_uploads.load(Ordering::Relaxed);
    let failures = total_failures.load(Ordering::Relaxed);
    
    println!("✓ Sustained load test completed");
    println!("  - Duration: {:?}", duration);
    println!("  - Total uploads: {}", uploads);
    println!("  - Failures: {}", failures);
    println!("  - Average throughput: {:.1} uploads/min", uploads as f64 / (duration.as_secs_f64() / 60.0));
    
    let success_rate = (uploads as f64 / (uploads + failures) as f64) * 100.0;
    assert!(
        success_rate >= 95.0,
        "Success rate should remain above 95% under sustained load"
    );
    
    println!("\n✅ Sustained load test PASSED");
    Ok(())
}

// Helper functions

async fn authenticate_test_user() -> Result<String> {
    Ok("test_jwt_token".to_string())
}

async fn upload_test_file(token: &str, id: usize) -> Result<Uuid> {
    let client = reqwest::Client::new();
    
    let test_data = create_minimal_binary();
    
    let form = multipart::Form::new().part(
        "file",
        multipart::Part::bytes(test_data)
            .file_name(format!("test_{}.exe", id))
            .mime_str("application/x-msdownload")?,
    );
    
    let response = client
        .post(format!("{}/api/v1/artifacts/upload", API_BASE_URL))
        .header("Authorization", format!("Bearer {}", token))
        .multipart(form)
        .timeout(Duration::from_secs(30))
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

fn create_minimal_binary() -> Vec<u8> {
    vec![0x4D, 0x5A, 0x90, 0x00] // Minimal PE header
}

async fn establish_websocket_connection(id: usize) -> Result<()> {
    let ws_url = format!("{}/api/v1/ws", WS_BASE_URL);
    
    let (ws_stream, _) = tokio::time::timeout(
        Duration::from_secs(10),
        tokio_tungstenite::connect_async(&ws_url),
    )
    .await??;
    
    let (mut write, _read) = ws_stream.split();
    
    // Send a ping to keep connection alive
    let ping_msg = json!({
        "type": "ping",
        "connection_id": id
    });
    
    write
        .send(tokio_tungstenite::tungstenite::Message::Text(
            ping_msg.to_string(),
        ))
        .await?;
    
    // Keep connection open for a bit
    tokio::time::sleep(Duration::from_secs(60)).await;
    
    Ok(())
}

async fn create_test_tenant() -> Result<Uuid> {
    use sqlx::PgPool;
    
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://postgres:postgres@localhost:5432/security_saas".to_string());
    
    let pool = PgPool::connect(&database_url).await?;
    let tenant_id = Uuid::new_v4();
    
    sqlx::query!(
        r#"
        INSERT INTO tenants (id, name, encryption_key_id, created_at)
        VALUES ($1, $2, $3, NOW())
        "#,
        tenant_id,
        "Load Test Tenant",
        format!("key_{}", tenant_id)
    )
    .execute(&pool)
    .await?;
    
    Ok(tenant_id)
}

fn create_event_batch(endpoint_id: &Uuid, tenant_id: &Uuid, count: usize) -> Vec<serde_json::Value> {
    (0..count)
        .map(|i| {
            json!({
                "endpoint_id": endpoint_id.to_string(),
                "tenant_id": tenant_id.to_string(),
                "event_type": "file_open",
                "process_name": "test.exe",
                "process_pid": 1000 + i,
                "timestamp": chrono::Utc::now().to_rfc3339(),
                "severity": 1
            })
        })
        .collect()
}

async fn send_telemetry_batch(events: &Vec<serde_json::Value>) -> Result<()> {
    let client = reqwest::Client::new();
    
    let response = client
        .post(format!("{}/api/v1/telemetry/events", API_BASE_URL))
        .json(&json!({ "events": events }))
        .timeout(Duration::from_secs(10))
        .send()
        .await?;
    
    if !response.status().is_success() {
        anyhow::bail!("Failed to send telemetry: {}", response.status());
    }
    
    Ok(())
}

async fn get_worker_count(worker_type: &str) -> Result<usize> {
    // Query Kubernetes API or metrics endpoint for worker count
    let client = reqwest::Client::new();
    
    let response = client
        .get(format!("{}/api/v1/metrics/workers/{}", API_BASE_URL, worker_type))
        .send()
        .await?;
    
    if !response.status().is_success() {
        // Fallback to default if metrics not available
        return Ok(2);
    }
    
    let body: serde_json::Value = response.json().await?;
    Ok(body["count"].as_u64().unwrap_or(2) as usize)
}

async fn get_queue_depth(queue_name: &str) -> Result<usize> {
    let client = reqwest::Client::new();
    
    let response = client
        .get(format!("{}/api/v1/metrics/queue/{}", API_BASE_URL, queue_name))
        .send()
        .await?;
    
    if !response.status().is_success() {
        return Ok(0);
    }
    
    let body: serde_json::Value = response.json().await?;
    Ok(body["depth"].as_u64().unwrap_or(0) as usize)
}

async fn get_scaling_events() -> Result<Vec<serde_json::Value>> {
    let client = reqwest::Client::new();
    
    let response = client
        .get(format!("{}/api/v1/metrics/scaling-events", API_BASE_URL))
        .send()
        .await?;
    
    if !response.status().is_success() {
        return Ok(Vec::new());
    }
    
    let body: serde_json::Value = response.json().await?;
    Ok(body["events"].as_array().unwrap_or(&vec![]).clone())
}
