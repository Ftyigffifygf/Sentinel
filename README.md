# 🛡️ Sentinel - Cloud-Native Malware Analysis Platform
## Comprehensive Documentation

> **Real-time threat detection and behavioral analysis at scale**

A cloud-native, multi-tenant Security-as-a-Service platform built in Rust for real-time detection and analysis of suspicious binaries and endpoint behaviors.

## Table of Contents

1. [Overview](#overview)
2. [Architecture](#architecture)
3. [Core Services](#core-services)
4. [Shared Libraries](#shared-libraries)
5. [Infrastructure Components](#infrastructure-components)
6. [Admin Console](#admin-console)
7. [Getting Started](#getting-started)
8. [Deployment](#deployment)
9. [Database Setup](#database-setup)
10. [API Documentation](#api-documentation)
11. [Security Features](#security-features)
12. [Monitoring & Observability](#monitoring--observability)
13. [Testing](#testing)
14. [Configuration](#configuration)
15. [Troubleshooting](#troubleshooting)
16. [Contributing](#contributing)
17. [License](#license)

---

## Overview

The Security SaaS Platform is an enterprise-grade malware analysis and threat detection system that provides:

- **Static Binary Analysis**: PE/ELF/Mach-O parsing, YARA scanning, entropy analysis, threat intelligence lookups
- **Dynamic Sandbox Analysis**: Isolated execution environments with behavioral monitoring
- **Endpoint Behavioral Monitoring**: Real-time telemetry ingestion and threat detection
- **Multi-Tenant Architecture**: Complete data isolation with tenant-specific encryption
- **Real-Time Streaming**: WebSocket-based progress updates and verdict delivery
- **SIEM/SOAR Integration**: CEF/LEEF export, webhook delivery, external query API
- **Compliance Features**: Audit logging, data retention, GDPR-compliant export/deletion
- **Horizontal Autoscaling**: Queue-based worker scaling with Kubernetes HPA

### Key Features

✅ File upload up to 500MB with instant tracking ID  
✅ Complete analysis pipeline (static + dynamic) in < 5 minutes  
✅ Real-time WebSocket streaming of analysis progress  
✅ Comprehensive verdict with risk scoring (0-100)  
✅ Case management for security investigations  
✅ Alert rules with multiple notification channels  
✅ Allow/deny list management with audit trail  
✅ Threat intelligence integration with auto-refresh  
✅ Horizontal pod autoscaling based on queue depth  
✅ Full observability with Prometheus, Grafana, Jaeger  


---

## Architecture

### System Architecture

```
┌─────────────────────────────────────────────────────────────────────┐
│                          Admin Console (React)                       │
│                    OAuth/OIDC, WebSocket, REST API                   │
└────────────────────────────┬────────────────────────────────────────┘
                             │
                             ▼
┌─────────────────────────────────────────────────────────────────────┐
│                         API Layer (Rust)                             │
│  • File Upload & Ingestion      • WebSocket Streaming               │
│  • Authentication & Authorization • REST Endpoints                   │
│  • SIEM Integration             • Telemetry Ingestion               │
│  • Alerting & Notifications     • Audit Logging                     │
└──────┬──────────────────┬──────────────────┬────────────────────────┘
       │                  │                  │
       ▼                  ▼                  ▼
┌─────────────┐   ┌─────────────┐   ┌─────────────┐
│   Static    │   │   Dynamic   │   │ Behavioral  │
│   Worker    │   │   Worker    │   │   Worker    │
│             │   │             │   │             │
│ • PE/ELF    │   │ • Sandbox   │   │ • Telemetry │
│ • YARA      │   │ • Tracing   │   │ • Detection │
│ • Entropy   │   │ • Heuristics│   │ • Alerting  │
│ • Threat    │   │ • Scoring   │   │ • Storage   │
│   Intel     │   │             │   │             │
└──────┬──────┘   └──────┬──────┘   └──────┬──────┘
       │                  │                  │
       └──────────────────┴──────────────────┘
                          │
       ┌──────────────────┴──────────────────┐
       │                                      │
       ▼                                      ▼
┌─────────────────┐                  ┌─────────────────┐
│   PostgreSQL    │                  │   NATS          │
│   + TimescaleDB │                  │   JetStream     │
│                 │                  │                 │
│ • Artifacts     │                  │ • Job Queue     │
│ • Verdicts      │                  │ • Pub/Sub       │
│ • Cases         │                  │ • Streaming     │
│ • Audit Logs    │                  │                 │
└─────────────────┘                  └─────────────────┘
       │                                      │
       ▼                                      ▼
┌─────────────────┐                  ┌─────────────────┐
│   MinIO (S3)    │                  │   Redis         │
│                 │                  │                 │
│ • Artifact      │                  │ • WebSocket     │
│   Storage       │                  │   Buffering     │
│ • Snapshots     │                  │ • Rate Limiting │
└─────────────────┘                  └─────────────────┘
```

### Workspace Structure

```
security-saas-platform/
├── crates/
│   ├── api-layer/              # REST & WebSocket API service
│   ├── static-worker/          # Static binary analysis
│   ├── dynamic-worker/         # Sandbox execution
│   ├── behavioral-worker/      # Endpoint monitoring
│   ├── shared-domain/          # Common models & types
│   ├── shared-crypto/          # Cryptographic utilities
│   └── shared-db/              # Database utilities
├── admin-console/              # React frontend
├── migrations/                 # Database migrations
├── k8s/                        # Kubernetes manifests
├── docker/                     # Dockerfiles
├── scripts/                    # Deployment scripts
└── tests/                      # Integration tests
```


---

## Core Services

### 1. API Layer

**Purpose**: Central API gateway for all client interactions

**Key Features**:
- **Authentication**: OAuth 2.0/OIDC with JWT validation, RBAC (Admin/Analyst/Viewer)
- **File Upload**: Multipart upload up to 500MB, SHA-256/MD5/ssdeep hashing
- **WebSocket Streaming**: Real-time progress updates and verdict delivery
- **REST Endpoints**: Artifact queries, verdict management, case management
- **SIEM Integration**: CEF/LEEF export, webhook delivery with retry logic
- **Telemetry Ingestion**: Endpoint behavioral data collection
- **Alerting**: Rule engine with email/webhook/Slack notifications
- **Compliance**: Audit logging, data retention, GDPR export/deletion

**Technologies**: Axum, SQLx, async-nats, Redis, AWS S3 SDK

**Endpoints**:
- `POST /api/v1/artifacts/upload` - Upload files for analysis
- `GET /api/v1/ws` - WebSocket connection for real-time updates
- `GET /api/v1/verdicts/:id` - Retrieve verdict details
- `PATCH /api/v1/verdicts/:id/override` - Override verdict with justification
- `POST /api/v1/cases` - Create investigation case
- `POST /api/v1/alerts/rules` - Configure alert rules
- `POST /api/v1/telemetry/events` - Ingest endpoint telemetry
- `GET /api/v1/audit/logs` - Query audit logs
- `GET /metrics` - Prometheus metrics

### 2. Static Worker

**Purpose**: Automated static binary analysis

**Analysis Capabilities**:
- **Binary Parsing**: PE (Windows), ELF (Linux), Mach-O (macOS) format parsing
- **YARA Scanning**: Pattern matching with 30-second timeout
- **String Extraction**: ASCII and UTF-16LE strings with suspicious pattern detection
- **Entropy Analysis**: Shannon entropy calculation, packed region detection
- **Threat Intelligence**: Hash/domain/IP lookup with auto-refresh (15 min)
- **Risk Scoring**: Weighted scoring algorithm (0-100 scale)

**Scoring Weights**:
- YARA matches: 30 points each (max 40)
- Threat intel hits: 40 points each (max 50)
- Suspicious strings: 5 points each (max 20)
- High entropy sections: 10 points each (max 15)
- Suspicious flags: 15 points each (max 20)

**Performance**: Completes analysis within 30 seconds

**Technologies**: goblin, yara-rust, async-nats, SQLx

### 3. Dynamic Worker

**Purpose**: Isolated sandbox execution and behavioral monitoring

**Sandbox Features**:
- **Container Isolation**: Filesystem, network, and process isolation
- **Resource Limits**: CPU (1 core), Memory (2GB), Disk (10GB), Time (5 min)
- **Security Policies**: Seccomp filters, AppArmor/SELinux profiles
- **Network Blocking**: DNS sinkhole, HTTP interception
- **Snapshot Capability**: State capture for forensic analysis

**Behavioral Monitoring**:
- **File Operations**: Open, read, write, rename, delete tracking
- **Registry Operations**: Windows registry modification detection
- **Process Events**: Creation, termination, injection detection
- **Network Activity**: DNS queries, HTTP requests, connection tracking
- **System Calls**: ptrace-based syscall tracing

**Heuristics Detection**:
- Ransomware indicators (high file op rate, shadow copy deletion, encryption patterns)
- Persistence mechanisms (registry run keys, startup folders, scheduled tasks)
- Privilege escalation (UAC bypass, token manipulation)
- Process injection (DLL injection, process hollowing)
- Data exfiltration (large network transfers)

**Risk Scoring**:
- Ransomware: 30 points
- Data Exfiltration: 25 points
- Privilege Escalation: 20 points
- Lateral Movement: 20 points
- Persistence: 15 points
- Defense Evasion: 15 points

**Technologies**: nix, libc, tokio, SQLx

### 4. Behavioral Worker

**Purpose**: Endpoint telemetry ingestion and real-time threat detection

**Features**:
- **Telemetry Ingestion**: High-volume event processing (100k+ events/sec)
- **TimescaleDB Storage**: Time-series data with < 500ms latency
- **Detection Rules**: Pattern matching for suspicious behaviors
- **Alert Generation**: Real-time alerts within 2 seconds
- **Event Correlation**: Cross-endpoint attack chain detection

**Detection Patterns**:
- Lateral movement (SMB, RDP, WinRM connections)
- Suspicious process execution (mimikatz, psexec, procdump)
- Privilege escalation attempts (runas, elevate, sudo)
- Suspicious file operations (system files, shadow copies)

**Technologies**: SQLx, TimescaleDB, async-nats


---

## Shared Libraries

### shared-domain

**Purpose**: Common domain models, enums, and error types

**Key Components**:
- **Models**: Artifact, Verdict, Tenant, User, Case, Alert
- **Enums**: VerdictType, FileType, Role, Permission, Severity
- **Errors**: Domain-specific error types with context
- **Metrics**: Prometheus metrics for all services
- **Observability**: Structured logging, OpenTelemetry tracing

### shared-crypto

**Purpose**: Cryptographic utilities for security operations

**Features**:
- **Hashing**: SHA-256, MD5, ssdeep (fuzzy hashing)
- **Encryption**: AES-256-GCM with random nonce generation
- **Key Management**: Secure key generation and base64 encoding
- **ID Generation**: Cryptographically secure UUID generation

**Usage Example**:
```rust
use shared_crypto::hash::compute_all_hashes;
use shared_crypto::encryption::{generate_key, encrypt, decrypt};

// Compute hashes
let hashes = compute_all_hashes(&file_data)?;
println!("SHA-256: {}", hashes.sha256);

// Encrypt data
let key = generate_key();
let ciphertext = encrypt(&data, &key)?;
let plaintext = decrypt(&ciphertext, &key)?;
```

### shared-db

**Purpose**: Database connection pooling and migration management

**Features**:
- **Connection Pooling**: SQLx-based PostgreSQL pool with retry logic
- **Migrations**: Automated schema migration with version tracking
- **Health Checks**: Connection health monitoring with timeout support
- **Configuration**: Environment-based configuration with defaults

**Configuration**:
```bash
DATABASE_URL=postgres://user:password@localhost:5432/security_saas
DB_MAX_CONNECTIONS=20
DB_MIN_CONNECTIONS=2
DB_CONNECT_TIMEOUT=30
DB_IDLE_TIMEOUT=600
DB_MAX_LIFETIME=1800
```

**Usage Example**:
```rust
use shared_db::{DbConfig, create_pool_with_retry, run_migrations};

let config = DbConfig::from_env()?;
let pool = create_pool_with_retry(&config, 5, 2).await?;
run_migrations(&pool).await?;
```

---

## Infrastructure Components

### PostgreSQL + TimescaleDB

**Purpose**: Primary data store with time-series capabilities

**Tables**:
- `tenants` - Multi-tenant organization data
- `users` - User accounts with roles
- `artifacts` - Uploaded files metadata
- `verdicts` - Analysis results and risk scores
- `cases` - Investigation cases
- `audit_logs` - Immutable audit trail (UPDATE/DELETE blocked)
- `hash_lists` - Allow/deny lists
- `alert_rules` - Alert configurations
- `alerts` - Generated alerts
- `endpoint_events` - Time-series telemetry (TimescaleDB hypertable)
- `webhook_integrations` - SIEM/SOAR webhooks
- `retention_policies` - Data retention configuration
- `data_exports` - Export request tracking
- `data_deletions` - Deletion request tracking

**Row-Level Security**: Enabled on all tenant-specific tables

### NATS JetStream

**Purpose**: Message queue for job distribution and pub/sub

**Streams**:
- `artifacts.uploaded` - New artifact analysis jobs
- `analysis.dynamic.requested` - Dynamic analysis requests
- `analysis.dynamic.complete` - Analysis completion events
- `verdicts.generated` - Verdict generation events
- `alerts.critical` - Critical alert notifications
- `alerts.general` - General alert notifications
- `telemetry.endpoints` - Endpoint behavioral events

**Features**:
- Durable consumers with explicit acknowledgment
- Consumer groups for load balancing
- Automatic retry with exponential backoff
- Dead letter queue for failed messages

### MinIO (S3-Compatible Storage)

**Purpose**: Object storage for artifacts and snapshots

**Storage Structure**:
```
{tenant_id}/artifacts/{year}/{month}/{day}/{artifact_id}.bin
{tenant_id}/snapshots/{sandbox_id}/{timestamp}.tar.gz
```

**Features**:
- Tenant-based path isolation
- Retry logic with exponential backoff
- Lifecycle policies for automatic cleanup

### Redis

**Purpose**: Caching and WebSocket buffering

**Use Cases**:
- WebSocket connection state tracking
- Message buffering for disconnected clients (5-minute TTL)
- Rate limiting state
- Session management


---

## Admin Console

**Technology Stack**: React 19, TypeScript, Vite, React Router v7, Zustand, TanStack Query, Axios

### Features

#### Authentication
- OAuth 2.0 / OIDC integration with PKCE flow
- JWT token management with automatic refresh
- Persistent authentication state
- Protected route guards

#### File Upload
- Drag-and-drop interface
- Multiple file upload support
- Real-time progress tracking via WebSocket
- Instant tracking ID display
- Navigation to verdict on completion

#### Dashboard
- Real-time metrics and statistics
- Verdict distribution visualization
- Top threats list
- Recent verdicts feed
- Critical alert banner
- Auto-refresh every 30 seconds

#### Verdict Display
- Detailed risk scoring breakdown
- Evidence presentation (YARA, behavioral, threat intel)
- Artifact metadata
- Override capability with justification
- Color-coded risk indicators

#### Case Management
- Investigation workflow
- Create cases with severity levels
- Link multiple artifacts to cases
- Case timeline tracking
- Status management (open → investigating → closed)
- Close case with incident summary

#### Hash Lists
- Allow/deny list management
- SHA-256 and MD5 hash support
- Justification and classification tracking
- Entry removal with audit trail

#### Alert Configuration
- Alert rule creation with conditions
- Multiple notification channels (email, webhook, Slack)
- Alert history view
- Rule management (create, delete)
- Severity-based filtering

### Configuration

Create `.env` file:
```bash
VITE_API_BASE_URL=http://localhost:8080
VITE_WS_BASE_URL=ws://localhost:8080/ws
VITE_OIDC_PROVIDER_URL=http://localhost:8080/auth
VITE_OIDC_CLIENT_ID=admin-console
VITE_OIDC_REDIRECT_URI=http://localhost:5173/callback
```

### Development

```bash
cd admin-console
npm install
npm run dev
```

### Production Build

```bash
npm run build
npm run preview
```

### Docker Deployment

```dockerfile
FROM node:18-alpine as build
WORKDIR /app
COPY package*.json ./
RUN npm ci
COPY . .
RUN npm run build

FROM nginx:alpine
COPY --from=build /app/dist /usr/share/nginx/html
COPY nginx.conf /etc/nginx/conf.d/default.conf
EXPOSE 80
CMD ["nginx", "-g", "daemon off;"]
```


---

## Getting Started

### Prerequisites

- **Rust**: 1.75 or higher
- **Node.js**: 20 or higher
- **Docker**: For infrastructure services
- **PostgreSQL**: 14+ with TimescaleDB extension
- **sqlx-cli**: `cargo install sqlx-cli --no-default-features --features postgres`

### Quick Start (Local Development)

#### 1. Start Infrastructure Services

```bash
chmod +x scripts/deploy-local.sh
./scripts/deploy-local.sh
```

This starts:
- PostgreSQL with TimescaleDB
- Redis
- NATS JetStream
- MinIO
- Prometheus, Grafana, Jaeger

#### 2. Set Environment Variables

```bash
export DATABASE_URL=postgresql://postgres:postgres@localhost:5432/security_saas
export REDIS_URL=redis://localhost:6379
export NATS_URL=nats://localhost:4222
export S3_ENDPOINT=http://localhost:9000
export S3_ACCESS_KEY=minioadmin
export S3_SECRET_KEY=minioadmin
export S3_BUCKET=security-saas-artifacts
```

#### 3. Run Database Migrations

```bash
sqlx migrate run
```

#### 4. Start Application Services

```bash
# Terminal 1: API Layer
cargo run -p api-layer

# Terminal 2: Static Worker
cargo run -p static-worker

# Terminal 3: Dynamic Worker
cargo run -p dynamic-worker

# Terminal 4: Behavioral Worker
cargo run -p behavioral-worker
```

#### 5. Start Frontend

```bash
cd admin-console
npm install
npm run dev
```

### Access Services

- **Admin Console**: http://localhost:5173
- **API**: http://localhost:8080
- **MinIO Console**: http://localhost:9001 (minioadmin/minioadmin)
- **Prometheus**: http://localhost:9090
- **Grafana**: http://localhost:3000 (admin/admin)
- **Jaeger**: http://localhost:16686

### Stop Services

```bash
docker-compose down
```

---

## Deployment

### Kubernetes Deployment

#### Prerequisites

- Kubernetes cluster (v1.24+)
- kubectl configured
- Helm 3.x
- cert-manager for TLS
- NGINX Ingress Controller
- Minimum: 8 CPU cores, 32GB RAM

#### Deployment Steps

1. **Create Namespaces**:
```bash
kubectl apply -f k8s/namespace.yaml
```

2. **Create Secrets**:
```bash
cp k8s/secrets-template.yaml k8s/secrets.yaml
# Edit secrets.yaml with actual base64-encoded credentials
kubectl apply -f k8s/secrets.yaml
```

3. **Create ConfigMaps**:
```bash
kubectl apply -f k8s/configmaps.yaml
```

4. **Deploy Infrastructure**:
```bash
kubectl apply -f k8s/postgresql-deployment.yaml
kubectl apply -f k8s/redis-deployment.yaml
kubectl apply -f k8s/nats-deployment.yaml
kubectl apply -f k8s/minio-deployment.yaml
```

5. **Run Migrations**:
```bash
kubectl apply -f k8s/migration-job.yaml
kubectl wait --for=condition=complete job/database-migration -n security-saas --timeout=300s
```

6. **Deploy Workers**:
```bash
kubectl apply -f k8s/static-worker-deployment.yaml
kubectl apply -f k8s/dynamic-worker-deployment.yaml
kubectl apply -f k8s/behavioral-worker-deployment.yaml
```

7. **Deploy API Layer**:
```bash
kubectl apply -f k8s/api-layer-deployment.yaml
```

8. **Deploy HPAs**:
```bash
kubectl apply -f k8s/static-worker-hpa.yaml
kubectl apply -f k8s/dynamic-worker-hpa.yaml
kubectl apply -f k8s/behavioral-worker-hpa.yaml
```

9. **Deploy Observability**:
```bash
kubectl apply -f k8s/observability/
kubectl apply -f k8s/worker-servicemonitor.yaml
kubectl apply -f k8s/prometheus-adapter-config.yaml
```

10. **Deploy Ingress**:
```bash
kubectl apply -f k8s/ingress.yaml
```

#### Verify Deployment

```bash
chmod +x scripts/test-deployment.sh
./scripts/test-deployment.sh
```

### Autoscaling Configuration

#### Static Worker HPA
- Min Replicas: 2
- Max Replicas: 20
- Target Queue Depth: 100 messages per pod
- Target CPU: 70%

#### Dynamic Worker HPA
- Min Replicas: 2
- Max Replicas: 15
- Target Queue Depth: 50 messages per pod
- Target CPU: 80%
- Target Memory: 75%

#### Behavioral Worker HPA
- Min Replicas: 2
- Max Replicas: 25
- Target Queue Depth: 200 messages per pod
- Target CPU: 65%


---

## Database Setup

### PostgreSQL Installation

#### Using Docker (Recommended for Development)

```bash
docker run -d \
  --name security-saas-db \
  -p 5432:5432 \
  -e POSTGRES_PASSWORD=your_password \
  -e POSTGRES_DB=security_saas \
  timescale/timescaledb:latest-pg14
```

#### Using Package Manager

**Ubuntu/Debian**:
```bash
sudo apt update
sudo apt install postgresql-14 postgresql-contrib-14
```

**macOS (Homebrew)**:
```bash
brew install postgresql@14
brew services start postgresql@14
```

### TimescaleDB Extension

```bash
psql -U postgres -d security_saas

CREATE EXTENSION IF NOT EXISTS timescaledb CASCADE;
\dx
\q
```

### Running Migrations

```bash
export DATABASE_URL=postgres://postgres:password@localhost:5432/security_saas

# Check migration status
sqlx migrate info

# Run all pending migrations
sqlx migrate run

# Revert last migration (if needed)
sqlx migrate revert
```

### Database Schema

The platform uses 20+ tables organized into categories:

**Core Tables**:
- `tenants` - Multi-tenant organization data
- `users` - User accounts with roles
- `artifacts` - Uploaded files for analysis
- `verdicts` - Analysis results and risk scores

**Analysis Tables**:
- `static_analysis_reports` - Static analysis results
- `behavioral_analysis_reports` - Dynamic analysis results

**Security Tables**:
- `hash_lists` - Allow/deny lists for hashes
- `audit_logs` - Immutable audit trail

**Case Management**:
- `cases` - Investigation cases
- `case_artifacts` - Case-artifact relationships

**Alerting**:
- `alert_rules` - Alert configurations
- `alerts` - Generated alerts

**Integrations**:
- `webhook_integrations` - SIEM/SOAR webhooks
- `webhook_deliveries` - Delivery tracking
- `threat_intel_feeds` - Threat intelligence sources
- `threat_intel_indicators` - Threat indicators cache

**Telemetry**:
- `endpoint_events` - Time-series endpoint data (TimescaleDB hypertable)

**Compliance**:
- `retention_policies` - Data retention configuration
- `data_exports` - Export request tracking
- `data_deletions` - Deletion request tracking

### Row-Level Security

Most tables have RLS enabled for tenant isolation. Applications must set:

```sql
SET app.current_tenant = 'tenant-uuid-here';
```

### Backup and Restore

**Backup**:
```bash
pg_dump -U postgres -d security_saas -F c -f backup.dump
```

**Restore**:
```bash
pg_restore -U postgres -d security_saas -c backup.dump
```


---

## API Documentation

### Authentication

All API requests require JWT authentication via Bearer token:

```bash
Authorization: Bearer <jwt_token>
```

### File Upload

**Endpoint**: `POST /api/v1/artifacts/upload`

**Request**:
```bash
curl -X POST http://localhost:8080/api/v1/artifacts/upload \
  -H "Authorization: Bearer <token>" \
  -F "file=@/path/to/file.exe"
```

**Response**:
```json
{
  "tracking_id": "abc123",
  "artifact_id": "uuid",
  "message": "File uploaded successfully"
}
```

### WebSocket Connection

**Endpoint**: `ws://localhost:8080/api/v1/ws`

**Subscribe to Updates**:
```json
{
  "type": "subscribe",
  "artifact_id": "uuid"
}
```

**Progress Update**:
```json
{
  "type": "progress",
  "artifact_id": "uuid",
  "stage": "static_analysis",
  "percent": 50
}
```

**Verdict Ready**:
```json
{
  "type": "verdict",
  "artifact_id": "uuid",
  "verdict_id": "uuid",
  "verdict": "malicious",
  "risk_score": 85
}
```

### Verdict Retrieval

**Endpoint**: `GET /api/v1/verdicts/:id`

**Response**:
```json
{
  "id": "uuid",
  "artifact_id": "uuid",
  "verdict": "malicious",
  "risk_score": 85,
  "static_score": 80,
  "behavioral_score": 90,
  "evidence": {
    "yara_matches": ["ransomware.rule"],
    "behavioral_indicators": ["file_encryption", "shadow_copy_deletion"],
    "threat_intel_hits": ["known_bad_hash"],
    "suspicious_strings": ["ransom_note.txt"],
    "network_indicators": ["c2_server.com"]
  },
  "created_at": "2025-11-15T10:30:45Z"
}
```

### Case Management

**Create Case**: `POST /api/v1/cases`
```json
{
  "title": "Ransomware Investigation",
  "description": "Suspected ransomware activity",
  "severity": "critical",
  "assigned_to": "analyst-uuid"
}
```

**Link Artifact**: `POST /api/v1/cases/:id/artifacts`
```json
{
  "artifact_id": "uuid"
}
```

**Close Case**: `POST /api/v1/cases/:id/close`
```json
{
  "incident_summary": "Confirmed ransomware, systems isolated and restored from backup"
}
```

### Alert Configuration

**Create Alert Rule**: `POST /api/v1/alerts/rules`
```json
{
  "name": "High Risk Malware Alert",
  "description": "Alert when malware with high risk score is detected",
  "conditions": [
    {
      "field": "risk_score",
      "operator": "greater_than",
      "value": 80
    },
    {
      "field": "verdict",
      "operator": "equals",
      "value": "\"Malicious\""
    }
  ],
  "severity": "critical",
  "enabled": true,
  "notification_channels": [
    {
      "type": "email",
      "recipients": ["security-team@example.com"]
    },
    {
      "type": "slack",
      "webhook_url": "https://hooks.slack.com/services/YOUR/WEBHOOK/URL"
    }
  ]
}
```

### SIEM Integration

**Create Webhook**: `POST /api/v1/integrations/webhook`
```json
{
  "name": "Splunk Integration",
  "url": "https://splunk.example.com/services/collector/event",
  "format": "cef",
  "auth_type": "bearer",
  "auth_credentials": {
    "token": "your-splunk-hec-token"
  }
}
```

**Query Verdicts**: `GET /api/v1/integrations/verdicts`
```bash
curl -X GET "http://localhost:8080/api/v1/integrations/verdicts?verdict=malicious&min_risk_score=70&limit=50" \
  -H "Authorization: Bearer <token>"
```

### Telemetry Ingestion

**Endpoint**: `POST /api/v1/telemetry/events`

**Request**:
```json
{
  "events": [
    {
      "endpoint_id": "uuid",
      "event_type": "process_start",
      "process_name": "suspicious.exe",
      "process_pid": 1234,
      "severity": 75,
      "timestamp": "2025-11-15T10:30:45Z"
    }
  ]
}
```

### Audit Logs

**Query Logs**: `GET /api/v1/audit/logs`
```bash
curl -X GET "http://localhost:8080/api/v1/audit/logs?action=verdict.overridden&start_date=2025-11-01&format=json" \
  -H "Authorization: Bearer <token>"
```


---

## Security Features

### Multi-Tenant Isolation

**Database Level**:
- Row-Level Security (RLS) on all tenant-specific tables
- Automatic tenant_id filtering on all queries
- Session variable: `SET app.current_tenant = 'uuid'`

**Application Level**:
- JWT token contains tenant_id claim
- All API requests validated against tenant context
- Cross-tenant access attempts logged and rejected

**Storage Level**:
- Tenant-based path isolation in S3: `{tenant_id}/artifacts/...`
- Separate encryption keys per tenant

### Authentication & Authorization

**OAuth 2.0 / OIDC**:
- PKCE flow for secure authorization
- JWT tokens with RS256 signature
- Automatic token refresh on expiration
- Token revocation support

**Role-Based Access Control (RBAC)**:
- **Admin**: Full system access
- **Analyst**: Upload, analyze, manage cases, override verdicts
- **Viewer**: Read-only access to artifacts, verdicts, cases

**Permissions**:
- UploadArtifacts, ViewArtifacts, DeleteArtifacts
- ViewVerdicts, OverrideVerdicts
- ManageCases, ViewCases
- ManageAlerts, ViewAlerts
- ManageHashLists, ViewHashLists
- ViewAuditLogs, ExportData

### Encryption

**Data at Rest**:
- AES-256-GCM encryption for sensitive data
- Tenant-specific encryption keys
- Encrypted database columns for PII

**Data in Transit**:
- TLS 1.3 for all external communication
- mTLS for service-to-service communication
- Certificate validation and expiration monitoring

### Audit Logging

**Immutable Audit Trail**:
- Database rules prevent UPDATE and DELETE on audit_logs
- All security-relevant actions logged
- Includes: user_id, tenant_id, action, resource, timestamp, IP address

**Logged Actions**:
- Authentication events (login, logout, failed attempts)
- Artifact operations (upload, view, delete)
- Verdict operations (generate, override, view)
- List modifications (allow/deny list changes)
- Case management (create, update, close)
- Alert operations (rule create/update/delete, triggered)
- Data lifecycle (export, deletion requests)

### Compliance Features

**GDPR Compliance**:
- Right to Access: Complete data export in JSON/ZIP format
- Right to Erasure: 30-day grace period deletion
- Data Minimization: Configurable retention policies
- Audit Trail: Immutable logs of all data access

**Data Retention**:
- Per-tenant retention policies
- Automatic cleanup of expired data
- Default periods: Artifacts (90 days), Verdicts (365 days), Audit Logs (730 days)

**Data Export**:
- Complete tenant data export
- Includes: artifacts, verdicts, cases, audit logs, hash lists, alerts
- ZIP archive with JSON files
- Export request tracking

**Data Deletion**:
- 30-day grace period before deletion
- Cancellation support for pending deletions
- Complete purging of all tenant data
- Deletion confirmation with counts

### Sandbox Security

**Isolation**:
- Container-based filesystem isolation
- Network blocking by default
- Seccomp syscall filtering
- AppArmor/SELinux profiles

**Resource Limits**:
- CPU: 1 core maximum
- Memory: 2GB maximum
- Disk: 10GB maximum
- Execution time: 5 minutes maximum

**Monitoring**:
- Real-time resource usage tracking
- Automatic termination on violations
- Snapshot capability for forensics


---

## Monitoring & Observability

### Structured Logging

**Configuration**:
```bash
LOG_LEVEL=info              # trace, debug, info, warn, error
LOG_FORMAT=json             # json or text
LOG_INCLUDE_LOCATION=false  # Include file/line numbers
```

**Log Format (JSON)**:
```json
{
  "timestamp": "2025-11-15T10:30:45.123Z",
  "level": "INFO",
  "service": "api-layer",
  "trace_id": "a1b2c3d4-e5f6-7890",
  "span_id": "1234567890abcdef",
  "tenant_id": "tenant-uuid",
  "user_id": "user-uuid",
  "message": "Processing request",
  "target": "api_layer::upload"
}
```

### Prometheus Metrics

**API Layer Metrics**:
- `http_requests_total` - Total HTTP requests by method, endpoint, status
- `http_request_duration_seconds` - Request latency histogram
- `websocket_connections_active` - Active WebSocket connections
- `artifacts_uploaded_total` - Artifacts uploaded by tenant
- `verdicts_generated_total` - Verdicts generated by type

**Worker Metrics**:
- `analysis_queue_depth` - Pending tasks in message queue
- `analysis_jobs_processed_total` - Jobs processed by worker type
- `analysis_duration_seconds` - Analysis duration histogram
- `analysis_active_tasks` - Currently processing tasks
- `worker_scaling_events_total` - Scaling events
- `worker_replica_count` - Current replica count

**Database Metrics**:
- `db_query_duration_seconds` - Database query duration
- `db_connections_active` - Active database connections

**Metrics Endpoint**: `GET /metrics`

### OpenTelemetry Tracing

**Configuration**:
```bash
ENABLE_TRACING=true
JAEGER_ENDPOINT=localhost:6831
```

**Trace Visualization**:
- Complete analysis pipeline from upload to verdict
- Service-to-service communication
- Database queries and external API calls
- Error propagation and timing

### Grafana Dashboards

**Recommended Dashboards**:

1. **API Performance**:
   - Request rate by endpoint
   - Latency percentiles (p50, p95, p99)
   - Error rate by status code
   - Request/response size distribution

2. **Worker Performance**:
   - Queue depth by worker type
   - Analysis duration histogram
   - Jobs processed rate
   - Active tasks gauge

3. **System Health**:
   - Database connection pool usage
   - Cache hit/miss ratio
   - WebSocket connections
   - Service uptime

4. **Business Metrics**:
   - Artifacts uploaded per tenant
   - Verdicts by type (clean/suspicious/malicious)
   - Alert generation rate
   - Case creation rate

### Prometheus Alerts

**Example Alert Rules**:

```yaml
groups:
  - name: api_alerts
    rules:
      - alert: HighErrorRate
        expr: rate(http_requests_total{status=~"5.."}[5m]) > 0.05
        for: 5m
        annotations:
          summary: "High error rate detected"
          
      - alert: HighLatency
        expr: histogram_quantile(0.95, http_request_duration_seconds) > 2
        for: 5m
        annotations:
          summary: "High API latency detected"
          
      - alert: QueueBacklog
        expr: analysis_queue_depth > 500
        for: 10m
        annotations:
          summary: "Analysis queue backlog"
          
      - alert: HPAMaxedOut
        expr: worker_replica_count >= worker_max_replicas
        for: 15m
        annotations:
          summary: "Worker HPA at maximum replicas"
```

### Access Dashboards

```bash
# Grafana
kubectl port-forward -n observability svc/grafana 3000:3000
# Open http://localhost:3000 (admin/admin)

# Prometheus
kubectl port-forward -n observability svc/prometheus 9090:9090
# Open http://localhost:9090

# Jaeger
kubectl port-forward -n observability svc/jaeger-query 16686:16686
# Open http://localhost:16686
```

### Log Aggregation

**Loki Integration**:
```bash
kubectl port-forward -n observability svc/loki 3100:3100
```

**Query Example**:
```logql
{app="api-layer"} 
| json 
| level="ERROR" 
| line_format "{{.timestamp}} {{.message}}"
```


---

## Testing

### Unit Tests

Run unit tests for all crates:

```bash
# All tests
cargo test --workspace

# Specific crate
cargo test -p shared-crypto
cargo test -p shared-db
cargo test -p api-layer
cargo test -p static-worker
cargo test -p dynamic-worker

# With output
cargo test -- --nocapture

# Specific test
cargo test -p shared-crypto hash::tests::test_sha256_hello_world
```

### Integration Tests

Integration tests require running infrastructure:

```bash
# Start infrastructure
docker-compose up -d

# Set environment variables
export DATABASE_URL=postgres://postgres:postgres@localhost:5432/security_saas
export REDIS_URL=redis://localhost:6379
export NATS_URL=nats://localhost:4222

# Run integration tests
cargo test --package integration-tests -- --ignored

# Specific test suite
cargo test --test complete_analysis_flow -- --ignored --nocapture
cargo test --test security_testing -- --ignored --nocapture
cargo test --test load_testing -- --ignored --nocapture
```

### Test Suites

**Complete Analysis Flow** (`complete_analysis_flow.rs`):
- File upload through Admin Console
- Static analysis execution
- Dynamic analysis execution
- Verdict generation and streaming
- WebSocket reconnection and buffering

**Multi-Tenant Isolation** (`multi_tenant_isolation.rs`):
- Multiple tenant account creation
- Data isolation verification
- Cross-tenant access rejection
- Tenant-specific encryption
- Database-level isolation

**SIEM Integration** (`siem_integration.rs`):
- Webhook configuration
- CEF format export
- LEEF format export
- Retry logic with exponential backoff
- Delivery timing verification

**Endpoint Monitoring** (`endpoint_monitoring.rs`):
- Telemetry event ingestion
- TimescaleDB storage (< 500ms)
- Suspicious pattern detection
- Real-time alert delivery (< 2s)
- High-volume telemetry handling

**Load Testing** (`load_testing.rs`):
- 1,000 concurrent file uploads
- 10,000 WebSocket connections
- 100,000 endpoint events per second
- Worker autoscaling verification
- Sustained load testing (5 minutes)

**Security Testing** (`security_testing.rs`):
- SQL injection prevention (7 attack vectors)
- Cross-tenant access prevention (5 bypass methods)
- JWT token manipulation detection (6 scenarios)
- Sandbox escape prevention (3 exploit types)
- Rate limit enforcement
- Authorization bypass prevention

### Frontend Testing

See `admin-console/TESTING.md` for frontend testing guide.

**Setup**:
```bash
cd admin-console
npm install --save-dev vitest @testing-library/react @testing-library/jest-dom
```

**Run Tests**:
```bash
npm test
npm run test:watch
```

### Performance Benchmarks

**Expected Performance**:
- File upload tracking ID: < 1s
- Static analysis: < 30s
- Dynamic analysis: < 5min
- Verdict storage: < 2s
- WebSocket delivery: < 1s
- Alert delivery: < 2s
- Telemetry storage: < 500ms
- API throughput: > 1000 req/min
- Event throughput: > 80,000 events/sec


---

## Configuration

### Environment Variables

#### API Layer

```bash
# Database
DATABASE_URL=postgresql://user:password@localhost:5432/security_saas
DB_MAX_CONNECTIONS=20
DB_MIN_CONNECTIONS=2

# Redis
REDIS_URL=redis://localhost:6379

# NATS
NATS_URL=nats://localhost:4222

# S3 Storage
S3_ENDPOINT=http://localhost:9000
S3_ACCESS_KEY=minioadmin
S3_SECRET_KEY=minioadmin
S3_BUCKET=security-saas-artifacts

# OIDC
OIDC_ISSUER_URL=https://auth.example.com
OIDC_CLIENT_ID=security-saas-platform
OIDC_CLIENT_SECRET=secret

# SMTP (for email notifications)
SMTP_HOST=smtp.example.com
SMTP_PORT=587
SMTP_USERNAME=alerts@example.com
SMTP_PASSWORD=secret
SMTP_FROM_ADDRESS=alerts@example.com

# Logging
LOG_LEVEL=info
LOG_FORMAT=json

# Tracing
ENABLE_TRACING=true
JAEGER_ENDPOINT=localhost:6831

# Metrics
METRICS_PORT=9090
```

#### Static Worker

```bash
DATABASE_URL=postgresql://user:password@localhost:5432/security_saas
NATS_URL=nats://localhost:4222
YARA_RULES_DIR=./yara_rules
STORAGE_BASE_PATH=./storage
THREAT_INTEL_FEEDS=/feeds/malware.json:json,/feeds/iocs.csv:csv
METRICS_PORT=9090
LOG_LEVEL=info
```

#### Dynamic Worker

```bash
DATABASE_URL=postgresql://user:password@localhost:5432/security_saas
NATS_URL=nats://localhost:4222
SANDBOX_BASE_DIR=/tmp/sandboxes
METRICS_PORT=9091
LOG_LEVEL=info
```

#### Behavioral Worker

```bash
DATABASE_URL=postgresql://user:password@localhost:5432/security_saas
TIMESCALE_URL=postgresql://user:password@localhost:5432/security_saas
NATS_URL=nats://localhost:4222
METRICS_PORT=9092
LOG_LEVEL=info
```

#### Admin Console

```bash
VITE_API_BASE_URL=http://localhost:8080
VITE_WS_BASE_URL=ws://localhost:8080/ws
VITE_OIDC_PROVIDER_URL=http://localhost:8080/auth
VITE_OIDC_CLIENT_ID=admin-console
VITE_OIDC_REDIRECT_URI=http://localhost:5173/callback
```

### Kubernetes ConfigMaps

See `k8s/configmaps.yaml` for Kubernetes configuration.

### Kubernetes Secrets

See `k8s/secrets-template.yaml` for secret structure. **Never commit actual secrets!**


---

## Troubleshooting

### Common Issues

#### Pods Not Starting

```bash
# Check pod status
kubectl get pods -n security-saas

# Describe pod to see events
kubectl describe pod <pod-name> -n security-saas

# Check logs
kubectl logs <pod-name> -n security-saas

# Check previous container logs (if crashed)
kubectl logs <pod-name> -n security-saas --previous
```

#### Database Connection Issues

```bash
# Test database connectivity
kubectl run -it --rm debug --image=postgres:14 --restart=Never -n security-saas -- \
  psql postgresql://postgres:password@postgresql.security-saas.svc.cluster.local:5432/security_saas

# Check database pod
kubectl logs -n security-saas postgresql-0

# Check connection pool
curl http://localhost:9090/metrics | grep db_connections
```

#### Message Queue Issues

```bash
# Check NATS status
kubectl exec -it -n security-saas nats-0 -- nats server info

# List streams
kubectl exec -it -n security-saas nats-0 -- nats stream ls

# Check stream info
kubectl exec -it -n security-saas nats-0 -- nats stream info ARTIFACTS

# Check consumer lag
kubectl exec -it -n security-saas nats-0 -- nats consumer info ARTIFACTS static-worker
```

#### Storage Issues

```bash
# Check PVC status
kubectl get pvc -n security-saas

# Check MinIO status
kubectl port-forward -n security-saas svc/minio 9001:9001
# Open http://localhost:9001

# Test S3 connectivity
aws s3 --endpoint-url http://localhost:9000 ls s3://security-saas-artifacts
```

#### HPA Not Scaling

```bash
# Check HPA status
kubectl get hpa -n security-saas
kubectl describe hpa static-worker-hpa -n security-saas

# Check if metrics are available
kubectl get --raw /apis/custom.metrics.k8s.io/v1beta1

# Check Prometheus Adapter logs
kubectl logs -n monitoring deployment/prometheus-adapter

# Verify ServiceMonitor is scraping
kubectl get servicemonitor -n security-saas
```

#### Metrics Not Available

```bash
# Check worker metrics endpoint
kubectl port-forward -n security-saas deployment/static-worker 9090:9090
curl http://localhost:9090/metrics

# Check Prometheus targets
kubectl port-forward -n monitoring svc/prometheus-operated 9090:9090
# Visit http://localhost:9090/targets

# Check Prometheus scrape config
kubectl get prometheus -n monitoring -o yaml
```

#### Workers Not Processing

```bash
# Check NATS connectivity
kubectl exec -it -n security-saas deployment/static-worker -- \
  nats stream info --server=nats://nats.security-saas.svc.cluster.local:4222

# Check worker logs
kubectl logs -n security-saas deployment/static-worker -f

# Check database connectivity
kubectl exec -it -n security-saas deployment/static-worker -- \
  psql $DATABASE_URL -c "SELECT 1"

# Check queue depth
kubectl get --raw "/apis/custom.metrics.k8s.io/v1beta1/namespaces/security-saas/pods/*/analysis_queue_depth" | jq .
```

#### WebSocket Connection Issues

```bash
# Check Redis connectivity
kubectl exec -it -n security-saas deployment/api-layer -- \
  redis-cli -h redis.security-saas.svc.cluster.local ping

# Check WebSocket endpoint
wscat -c ws://localhost:8080/api/v1/ws

# Check buffered messages
kubectl exec -it -n security-saas redis-0 -- \
  redis-cli KEYS "ws:buffer:*"
```

#### High Memory Usage

```bash
# Check resource usage
kubectl top pods -n security-saas
kubectl top nodes

# Check for memory leaks
kubectl exec -it -n security-saas <pod-name> -- ps aux

# Restart pod if needed
kubectl rollout restart deployment/<deployment-name> -n security-saas
```

### Performance Issues

```bash
# Check resource usage
kubectl top pods -n security-saas
kubectl top nodes

# Check HPA status
kubectl get hpa -n security-saas

# Check metrics
kubectl port-forward -n security-saas svc/api-layer 9090:9090
curl http://localhost:9090/metrics

# Check slow queries
kubectl logs -n security-saas postgresql-0 | grep "duration:"
```

### Rollback Deployment

```bash
# View rollout history
kubectl rollout history deployment/api-layer -n security-saas

# Rollback to previous version
kubectl rollout undo deployment/api-layer -n security-saas

# Rollback to specific revision
kubectl rollout undo deployment/api-layer -n security-saas --to-revision=2

# Check rollout status
kubectl rollout status deployment/api-layer -n security-saas
```


---

## Contributing

### Development Workflow

1. **Fork the repository**
2. **Create a feature branch**: `git checkout -b feature/my-feature`
3. **Make changes and commit**: `git commit -am 'Add new feature'`
4. **Run tests**: `cargo test --workspace`
5. **Run linting**: `cargo fmt && cargo clippy`
6. **Push to branch**: `git push origin feature/my-feature`
7. **Create Pull Request**

### Code Style

**Rust**:
- Follow Rust standard formatting: `cargo fmt`
- Address all clippy warnings: `cargo clippy`
- Write unit tests for new functionality
- Document public APIs with doc comments

**TypeScript/React**:
- Follow ESLint configuration
- Use TypeScript strict mode
- Write component tests
- Document complex logic

### Commit Messages

Follow conventional commits format:

```
feat: Add new feature
fix: Fix bug in component
docs: Update documentation
test: Add tests for feature
refactor: Refactor code
chore: Update dependencies
```

### Pull Request Guidelines

- Provide clear description of changes
- Reference related issues
- Include tests for new functionality
- Update documentation as needed
- Ensure CI passes

### Testing Requirements

- Unit tests for all new functions
- Integration tests for API endpoints
- Security tests for authentication/authorization
- Performance tests for critical paths

---

## License

MIT License

Copyright (c) 2025 Security SaaS Platform

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.

---

## Additional Resources

### Documentation Files

- **STRUCTURE.md** - Project structure and organization
- **DATABASE_SETUP.md** - Database installation and configuration
- **DEPLOYMENT.md** - Detailed deployment instructions
- **OBSERVABILITY_IMPLEMENTATION.md** - Monitoring and observability setup
- **QUEUE_MONITORING_IMPLEMENTATION.md** - Queue depth monitoring
- **SCALING_EVENT_LOGGING.md** - Autoscaling event logging
- **TEST_IMPLEMENTATION_SUMMARY.md** - Testing strategy and coverage

### Implementation Summaries

- **admin-console/IMPLEMENTATION_SUMMARY.md** - Frontend implementation details
- **admin-console/TESTING.md** - Frontend testing guide
- **crates/api-layer/AUTH_IMPLEMENTATION.md** - Authentication implementation
- **crates/api-layer/UPLOAD_IMPLEMENTATION.md** - File upload implementation
- **crates/api-layer/WEBSOCKET_IMPLEMENTATION.md** - WebSocket implementation
- **crates/api-layer/VERDICT_IMPLEMENTATION.md** - Verdict generation
- **crates/api-layer/REST_API_IMPLEMENTATION.md** - REST API endpoints
- **crates/api-layer/TELEMETRY_IMPLEMENTATION.md** - Telemetry ingestion
- **crates/api-layer/SIEM_INTEGRATION_IMPLEMENTATION.md** - SIEM integration
- **crates/api-layer/ALERTING_IMPLEMENTATION.md** - Alerting system
- **crates/api-layer/COMPLIANCE_IMPLEMENTATION.md** - Compliance features
- **crates/static-worker/IMPLEMENTATION_SUMMARY.md** - Static analysis worker
- **crates/static-worker/THREAT_INTEL_IMPLEMENTATION.md** - Threat intelligence
- **crates/dynamic-worker/IMPLEMENTATION_SUMMARY.md** - Dynamic analysis worker
- **tests/integration/README.md** - Integration testing guide
- **tests/integration/IMPLEMENTATION_COMPLETE.md** - Test implementation status
- **k8s/README.md** - Kubernetes deployment guide
- **k8s/DEPLOYMENT_GUIDE.md** - Detailed Kubernetes instructions
- **k8s/autoscaling-tests/README.md** - Autoscaling test guide

### External Links

- [Rust Documentation](https://doc.rust-lang.org/)
- [Axum Web Framework](https://docs.rs/axum/)
- [SQLx Database Library](https://github.com/launchbadge/sqlx)
- [NATS Messaging](https://docs.nats.io/)
- [React Documentation](https://react.dev/)
- [Kubernetes Documentation](https://kubernetes.io/docs/)
- [Prometheus Documentation](https://prometheus.io/docs/)
- [Grafana Documentation](https://grafana.com/docs/)

---

## Support

For issues, questions, or contributions:

- **GitHub Issues**: Report bugs and request features
- **Documentation**: Check the docs/ directory for detailed guides
- **Logs**: Review service logs for troubleshooting
- **Metrics**: Check Grafana dashboards for system health
- **Traces**: Use Jaeger for distributed tracing

---

**Built with ❤️ using Rust, React, and Kubernetes**

