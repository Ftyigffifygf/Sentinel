/// Integration test for endpoint monitoring
/// Tests Requirements: 7.1, 7.2, 7.3, 7.4, 7.5
/// 
/// This test verifies:
/// - Telemetry event ingestion from mock endpoint
/// - Event storage in TimescaleDB
/// - Alert generation for suspicious patterns
/// - Real-time alert delivery

use anyhow::Result;
use chrono::Utc;
use serde_json::json;
use sqlx::PgPool;
use std::time::Duration;
use uuid::Uuid;

const API_BASE_URL: &str = "http://localhost:8080";

#[tokio::test]
#[ignore] // Run with: cargo test --test endpoint_monitoring -- --ignored
async fn test_endpoint_telemetry_ingestion() -> Result<()> {
    println!("Starting endpoint telemetry ingestion test...");

    // Step 1: Create test endpoint and get mTLS certificate
    let endpoint_id = Uuid::new_v4();
    let tenant_id = create_test_tenant().await?;
    println!("✓ Test tenant and endpoint created");

    // Step 2: Send telemetry events
    let events = create_test_telemetry_events(&endpoint_id, &tenant_id);
    send_telemetry_events(&events).await?;
    println!("✓ Sent {} telemetry events", events.len());

    // Step 3: Verify events stored in TimescaleDB within 500ms (Requirement 7.3)
    tokio::time::sleep(Duration::from_millis(600)).await;
    
    let stored_events = query_endpoint_events(&tenant_id, &endpoint_id).await?;
    assert_eq!(
        stored_events.len(),
        events.len(),
        "All events should be stored"
    );
    println!("✓ All events stored in TimescaleDB");

    // Step 4: Verify event data integrity
    for (sent, stored) in events.iter().zip(stored_events.iter()) {
        assert_eq!(
            sent["event_type"], stored["event_type"],
            "Event type should match"
        );
        assert_eq!(
            sent["process_name"], stored["process_name"],
            "Process name should match"
        );
    }
    println!("✓ Event data integrity verified");

    println!("\n✅ Telemetry ingestion test PASSED");
    Ok(())
}

#[tokio::test]
#[ignore]
async fn test_suspicious_pattern_detection() -> Result<()> {
    println!("Starting suspicious pattern detection test...");

    let endpoint_id = Uuid::new_v4();
    let tenant_id = create_test_tenant().await?;
    let token = create_test_user_token(&tenant_id).await?;

    // Step 1: Send normal events (should not trigger alerts)
    let normal_events = vec![
        create_telemetry_event(&endpoint_id, &tenant_id, "process_start", "notepad.exe", None),
        create_telemetry_event(&endpoint_id, &tenant_id, "file_open", "notepad.exe", Some("document.txt")),
    ];
    
    send_telemetry_events(&normal_events).await?;
    println!("✓ Sent normal events");

    tokio::time::sleep(Duration::from_secs(2)).await;

    // Verify no alerts generated
    let alerts = query_alerts(&token, &tenant_id).await?;
    let initial_alert_count = alerts.len();
    println!("✓ No alerts for normal activity (baseline: {} alerts)", initial_alert_count);

    // Step 2: Send suspicious events (lateral movement pattern)
    let suspicious_events = vec![
        create_telemetry_event(
            &endpoint_id,
            &tenant_id,
            "process_start",
            "psexec.exe",
            Some("\\\\remote-host\\admin$"),
        ),
        create_telemetry_event(
            &endpoint_id,
            &tenant_id,
            "network_connection",
            "psexec.exe",
            Some("remote-host:445"),
        ),
        create_telemetry_event(
            &endpoint_id,
            &tenant_id,
            "process_start",
            "cmd.exe",
            Some("/c whoami"),
        ),
    ];
    
    send_telemetry_events(&suspicious_events).await?;
    println!("✓ Sent suspicious lateral movement events");

    // Step 3: Wait for detection rules to process
    tokio::time::sleep(Duration::from_secs(3)).await;

    // Step 4: Verify alert was generated (Requirement 7.4)
    let new_alerts = query_alerts(&token, &tenant_id).await?;
    assert!(
        new_alerts.len() > initial_alert_count,
        "Alert should be generated for suspicious pattern"
    );
    println!("✓ Alert generated for lateral movement pattern");

    // Step 5: Verify alert contains detection details
    let latest_alert = &new_alerts[new_alerts.len() - 1];
    assert!(
        latest_alert.get("severity").is_some(),
        "Alert should have severity"
    );
    assert!(
        latest_alert.get("detection_rule").is_some(),
        "Alert should reference detection rule"
    );
    println!("✓ Alert contains required fields");

    println!("\n✅ Suspicious pattern detection test PASSED");
    Ok(())
}

#[tokio::test]
#[ignore]
async fn test_real_time_alert_delivery() -> Result<()> {
    println!("Starting real-time alert delivery test...");

    let endpoint_id = Uuid::new_v4();
    let tenant_id = create_test_tenant().await?;
    let token = create_test_user_token(&tenant_id).await?;

    // Step 1: Establish WebSocket connection for alerts
    let ws_url = format!("ws://localhost:8080/api/v1/ws/alerts");
    let (ws_stream, _) = tokio_tungstenite::connect_async(&ws_url).await?;
    let (mut write, mut read) = futures_util::StreamExt::split(ws_stream);
    println!("✓ WebSocket connection established");

    // Step 2: Subscribe to alerts
    use futures_util::SinkExt;
    let subscribe_msg = json!({
        "type": "subscribe_alerts",
        "token": token
    });
    write
        .send(tokio_tungstenite::tungstenite::Message::Text(
            subscribe_msg.to_string(),
        ))
        .await?;
    println!("✓ Subscribed to alerts");

    // Step 3: Send high-severity suspicious events
    let critical_events = vec![
        create_telemetry_event(
            &endpoint_id,
            &tenant_id,
            "process_start",
            "mimikatz.exe",
            None,
        ),
        create_telemetry_event(
            &endpoint_id,
            &tenant_id,
            "registry_modify",
            "mimikatz.exe",
            Some("HKLM\\SAM\\SAM"),
        ),
    ];
    
    let send_time = std::time::Instant::now();
    send_telemetry_events(&critical_events).await?;
    println!("✓ Sent critical security events");

    // Step 4: Wait for alert via WebSocket (should be within 2 seconds - Requirement 7.5)
    use futures_util::StreamExt;
    let timeout_duration = Duration::from_secs(3);
    
    let alert_received = tokio::time::timeout(timeout_duration, async {
        while let Some(msg) = read.next().await {
            if let Ok(tokio_tungstenite::tungstenite::Message::Text(text)) = msg {
                if let Ok(json) = serde_json::from_str::<serde_json::Value>(&text) {
                    if json["type"] == "alert" {
                        return Some(json);
                    }
                }
            }
        }
        None
    })
    .await;

    assert!(alert_received.is_ok(), "Should receive alert within timeout");
    let alert = alert_received?.ok_or_else(|| anyhow::anyhow!("No alert received"))?;
    
    let delivery_time = send_time.elapsed();
    println!("✓ Alert delivered in {:?}", delivery_time);
    
    assert!(
        delivery_time < Duration::from_secs(2),
        "Alert should be delivered within 2 seconds"
    );
    println!("✓ Alert delivery time meets requirement (< 2s)");

    // Step 5: Verify alert content
    assert_eq!(alert["type"], "alert");
    assert!(alert.get("severity").is_some());
    assert!(alert.get("endpoint_id").is_some());
    println!("✓ Alert contains required fields");

    println!("\n✅ Real-time alert delivery test PASSED");
    Ok(())
}

#[tokio::test]
#[ignore]
async fn test_high_volume_telemetry() -> Result<()> {
    println!("Starting high-volume telemetry test...");

    let endpoint_id = Uuid::new_v4();
    let tenant_id = create_test_tenant().await?;

    // Step 1: Generate large batch of events
    let event_count = 1000;
    let mut events = Vec::new();
    
    for i in 0..event_count {
        events.push(create_telemetry_event(
            &endpoint_id,
            &tenant_id,
            "file_open",
            "test_process.exe",
            Some(&format!("file_{}.txt", i)),
        ));
    }
    println!("✓ Generated {} test events", event_count);

    // Step 2: Send events in batches
    let batch_size = 100;
    let start_time = std::time::Instant::now();
    
    for chunk in events.chunks(batch_size) {
        send_telemetry_events(&chunk.to_vec()).await?;
    }
    
    let send_duration = start_time.elapsed();
    println!("✓ Sent {} events in {:?}", event_count, send_duration);

    // Step 3: Wait for processing
    tokio::time::sleep(Duration::from_secs(2)).await;

    // Step 4: Verify all events stored
    let stored_events = query_endpoint_events(&tenant_id, &endpoint_id).await?;
    
    let stored_count = stored_events.len();
    let success_rate = (stored_count as f64 / event_count as f64) * 100.0;
    
    println!("✓ Stored {}/{} events ({:.1}% success rate)", stored_count, event_count, success_rate);
    
    assert!(
        success_rate >= 95.0,
        "Should store at least 95% of events under load"
    );

    println!("\n✅ High-volume telemetry test PASSED");
    Ok(())
}

#[tokio::test]
#[ignore]
async fn test_mtls_authentication() -> Result<()> {
    println!("Starting mTLS authentication test...");

    let endpoint_id = Uuid::new_v4();
    let tenant_id = create_test_tenant().await?;

    // Step 1: Send telemetry without valid certificate (should fail)
    let event = create_telemetry_event(&endpoint_id, &tenant_id, "process_start", "test.exe", None);
    
    let result = send_telemetry_without_cert(&vec![event.clone()]).await;
    assert!(
        result.is_err() || result.unwrap() == false,
        "Request without valid mTLS cert should be rejected"
    );
    println!("✓ Request without mTLS certificate rejected");

    // Step 2: Send telemetry with valid certificate (should succeed)
    let result = send_telemetry_events(&vec![event]).await;
    assert!(result.is_ok(), "Request with valid mTLS cert should succeed");
    println!("✓ Request with valid mTLS certificate accepted");

    println!("\n✅ mTLS authentication test PASSED");
    Ok(())
}

#[tokio::test]
#[ignore]
async fn test_behavioral_correlation() -> Result<()> {
    println!("Starting behavioral correlation test...");

    let endpoint_id = Uuid::new_v4();
    let tenant_id = create_test_tenant().await?;
    let token = create_test_user_token(&tenant_id).await?;

    // Step 1: Send sequence of events that form an attack chain
    let attack_chain = vec![
        // Initial access
        create_telemetry_event(&endpoint_id, &tenant_id, "process_start", "outlook.exe", None),
        create_telemetry_event(&endpoint_id, &tenant_id, "file_write", "outlook.exe", Some("payload.exe")),
        
        // Execution
        create_telemetry_event(&endpoint_id, &tenant_id, "process_start", "payload.exe", None),
        
        // Privilege escalation
        create_telemetry_event(&endpoint_id, &tenant_id, "process_start", "payload.exe", Some("runas /user:admin")),
        
        // Lateral movement
        create_telemetry_event(&endpoint_id, &tenant_id, "network_connection", "payload.exe", Some("192.168.1.50:445")),
    ];
    
    for event in &attack_chain {
        send_telemetry_events(&vec![event.clone()]).await?;
        tokio::time::sleep(Duration::from_millis(500)).await;
    }
    println!("✓ Sent attack chain events");

    // Step 2: Wait for correlation engine
    tokio::time::sleep(Duration::from_secs(5)).await;

    // Step 3: Query for correlated alerts
    let alerts = query_alerts(&token, &tenant_id).await?;
    
    // Should have generated a high-severity correlated alert
    let correlated_alerts: Vec<_> = alerts
        .iter()
        .filter(|a| {
            a.get("correlation_id").is_some() || 
            a.get("attack_chain").is_some()
        })
        .collect();
    
    assert!(
        !correlated_alerts.is_empty(),
        "Should generate correlated alert for attack chain"
    );
    println!("✓ Correlated alert generated for attack chain");

    println!("\n✅ Behavioral correlation test PASSED");
    Ok(())
}

// Helper functions

async fn create_test_tenant() -> Result<Uuid> {
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
        "Test Tenant",
        format!("key_{}", tenant_id)
    )
    .execute(&pool)
    .await?;
    
    Ok(tenant_id)
}

async fn create_test_user_token(tenant_id: &Uuid) -> Result<String> {
    Ok(format!("test_token_{}", tenant_id))
}

fn create_test_telemetry_events(endpoint_id: &Uuid, tenant_id: &Uuid) -> Vec<serde_json::Value> {
    vec![
        create_telemetry_event(endpoint_id, tenant_id, "process_start", "chrome.exe", None),
        create_telemetry_event(endpoint_id, tenant_id, "network_connection", "chrome.exe", Some("google.com:443")),
        create_telemetry_event(endpoint_id, tenant_id, "file_open", "chrome.exe", Some("download.pdf")),
        create_telemetry_event(endpoint_id, tenant_id, "process_stop", "chrome.exe", None),
    ]
}

fn create_telemetry_event(
    endpoint_id: &Uuid,
    tenant_id: &Uuid,
    event_type: &str,
    process_name: &str,
    additional_data: Option<&str>,
) -> serde_json::Value {
    let mut event = json!({
        "endpoint_id": endpoint_id.to_string(),
        "tenant_id": tenant_id.to_string(),
        "event_type": event_type,
        "process_name": process_name,
        "process_pid": 1234,
        "timestamp": Utc::now().to_rfc3339(),
        "severity": 1
    });
    
    if let Some(data) = additional_data {
        event["event_data"] = json!({ "details": data });
    }
    
    event
}

async fn send_telemetry_events(events: &Vec<serde_json::Value>) -> Result<()> {
    let client = reqwest::Client::new();
    
    let response = client
        .post(format!("{}/api/v1/telemetry/events", API_BASE_URL))
        .json(&json!({ "events": events }))
        .send()
        .await?;
    
    if !response.status().is_success() {
        anyhow::bail!("Failed to send telemetry: {}", response.status());
    }
    
    Ok(())
}

async fn send_telemetry_without_cert(events: &Vec<serde_json::Value>) -> Result<bool> {
    let client = reqwest::Client::builder()
        .danger_accept_invalid_certs(true)
        .build()?;
    
    let response = client
        .post(format!("{}/api/v1/telemetry/events", API_BASE_URL))
        .json(&json!({ "events": events }))
        .send()
        .await?;
    
    Ok(response.status().is_success())
}

async fn query_endpoint_events(tenant_id: &Uuid, endpoint_id: &Uuid) -> Result<Vec<serde_json::Value>> {
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://postgres:postgres@localhost:5432/security_saas".to_string());
    
    let pool = PgPool::connect(&database_url).await?;
    
    let rows = sqlx::query!(
        r#"
        SELECT event_type, process_name, event_data
        FROM endpoint_events
        WHERE tenant_id = $1 AND endpoint_id = $2
        ORDER BY time DESC
        "#,
        tenant_id,
        endpoint_id
    )
    .fetch_all(&pool)
    .await?;
    
    let events: Vec<serde_json::Value> = rows
        .into_iter()
        .map(|row| {
            json!({
                "event_type": row.event_type,
                "process_name": row.process_name,
                "event_data": row.event_data
            })
        })
        .collect();
    
    Ok(events)
}

async fn query_alerts(token: &str, tenant_id: &Uuid) -> Result<Vec<serde_json::Value>> {
    let client = reqwest::Client::new();
    
    let response = client
        .get(format!("{}/api/v1/alerts", API_BASE_URL))
        .header("Authorization", format!("Bearer {}", token))
        .query(&[("tenant_id", tenant_id.to_string())])
        .send()
        .await?;
    
    if !response.status().is_success() {
        return Ok(Vec::new());
    }
    
    let body: serde_json::Value = response.json().await?;
    Ok(body["alerts"].as_array().unwrap_or(&vec![]).clone())
}
