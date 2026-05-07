//! Wireframe-AI Tenant Module
//!
//! Provides multi-tenant isolation for Wireframe-AI deployments.
//! Manages tenant creation, configuration, resource quotas, and isolation policies.
//! All state is persisted to SQLite for durability across restarts.
//!
//! Subscribes to: tenant.create, tenant.configure, tenant.quota.check, tenant.isolate
//! Publishes to: tenant.created, tenant.configured, tenant.quota.status

use agentic_sdk::{Envelope, Module};
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::sync::Arc;
use tokio::sync::Mutex;

/// Safely convert a u64 to i64, capping at i64::MAX to avoid silent wrap-around.
fn safe_u64_to_i64(value: u64) -> i64 {
    value.min(i64::MAX as u64) as i64
}

struct TenantModule {
    db: Arc<Mutex<Connection>>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct Tenant {
    id: String,
    name: String,
    created_at: i64,
    config: TenantConfig,
    status: TenantStatus,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct TenantConfig {
    max_tokens_per_minute: u64,
    max_requests_per_minute: u64,
    max_concurrent_jobs: u32,
    allowed_providers: Vec<String>,
    allowed_topics: Vec<String>,
    sandbox_enabled: bool,
    selfdev_enabled: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
enum TenantStatus {
    Active,
    Suspended,
    Pending,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct TenantUsage {
    tenant_id: String,
    tokens_used: u64,
    tokens_remaining: u64,
    requests_made: u64,
    requests_remaining: u64,
    active_jobs: u32,
    last_updated: i64,
}

const TENANT_SCHEMA: &str = r#"
CREATE TABLE IF NOT EXISTS tenants (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    created_at INTEGER NOT NULL,
    max_tokens_per_minute INTEGER NOT NULL DEFAULT 100000,
    max_requests_per_minute INTEGER NOT NULL DEFAULT 100,
    max_concurrent_jobs INTEGER NOT NULL DEFAULT 5,
    allowed_providers TEXT NOT NULL DEFAULT '[]',
    allowed_topics TEXT NOT NULL DEFAULT '[]',
    sandbox_enabled INTEGER NOT NULL DEFAULT 1,
    selfdev_enabled INTEGER NOT NULL DEFAULT 0,
    status TEXT NOT NULL DEFAULT 'active'
);

CREATE TABLE IF NOT EXISTS tenant_usage (
    tenant_id TEXT PRIMARY KEY,
    tokens_used INTEGER NOT NULL DEFAULT 0,
    tokens_remaining INTEGER NOT NULL DEFAULT 0,
    requests_made INTEGER NOT NULL DEFAULT 0,
    requests_remaining INTEGER NOT NULL DEFAULT 0,
    active_jobs INTEGER NOT NULL DEFAULT 0,
    last_updated INTEGER NOT NULL DEFAULT 0,
    FOREIGN KEY (tenant_id) REFERENCES tenants(id)
);
"#;

fn init_db(db_path: &str) -> Result<Connection, Box<dyn std::error::Error>> {
    let conn = Connection::open(db_path)?;
    conn.execute_batch("PRAGMA journal_mode=WAL;")?;
    conn.execute_batch(TENANT_SCHEMA)?;
    tracing::info!(db = %db_path, "tenant database initialized");
    Ok(conn)
}

fn config_from_row(row: &rusqlite::Row) -> rusqlite::Result<TenantConfig> {
    let providers_json: String = row.get(6)?;
    let topics_json: String = row.get(7)?;
    Ok(TenantConfig {
        max_tokens_per_minute: row.get::<_, i64>(3)? as u64,
        max_requests_per_minute: row.get::<_, i64>(4)? as u64,
        max_concurrent_jobs: row.get::<_, i64>(5)? as u32,
        allowed_providers: serde_json::from_str(&providers_json).unwrap_or_default(),
        allowed_topics: serde_json::from_str(&topics_json).unwrap_or_default(),
        sandbox_enabled: row.get::<_, i64>(8)? != 0,
        selfdev_enabled: row.get::<_, i64>(9)? != 0,
    })
}

fn tenant_from_row(row: &rusqlite::Row) -> rusqlite::Result<Tenant> {
    let status_str: String = row.get(10)?;
    let status = match status_str.as_str() {
        "suspended" => TenantStatus::Suspended,
        "pending" => TenantStatus::Pending,
        _ => TenantStatus::Active,
    };
    Ok(Tenant {
        id: row.get(0)?,
        name: row.get(1)?,
        created_at: row.get(2)?,
        config: config_from_row(row)?,
        status,
    })
}

fn usage_from_row(row: &rusqlite::Row) -> rusqlite::Result<TenantUsage> {
    Ok(TenantUsage {
        tenant_id: row.get(0)?,
        tokens_used: row.get::<_, i64>(1)? as u64,
        tokens_remaining: row.get::<_, i64>(2)? as u64,
        requests_made: row.get::<_, i64>(3)? as u64,
        requests_remaining: row.get::<_, i64>(4)? as u64,
        active_jobs: row.get::<_, i64>(5)? as u32,
        last_updated: row.get(6)?,
    })
}

#[agentic_sdk::module(
    subscribes = ["tenant.create", "tenant.configure", "tenant.quota.check", "tenant.isolate", "tenant.usage.report"],
    publishes  = ["tenant.created", "tenant.configured", "tenant.quota.status", "tenant.isolated"],
    queue_group = "tenant_handler"
)]
impl Module for TenantModule {
    async fn handle(&mut self, env: Envelope<Value>) -> Vec<Envelope<Value>> {
        match env.topic.as_str() {
            "tenant.create" => self.handle_create(env).await,
            "tenant.configure" => self.handle_configure(env).await,
            "tenant.quota.check" => self.handle_quota_check(env).await,
            "tenant.isolate" => self.handle_isolate(env).await,
            "tenant.usage.report" => self.handle_usage_report(env).await,
            _ => vec![],
        }
    }
}

impl TenantModule {
    async fn handle_create(&self, env: Envelope<Value>) -> Vec<Envelope<Value>> {
        let payload = &env.payload;
        let id = payload
            .get("id")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        let name = payload
            .get("name")
            .and_then(|v| v.as_str())
            .unwrap_or(&id)
            .to_string();

        if id.is_empty() {
            return vec![env.reply(
                "tenant.created",
                serde_json::json!({
                    "error": "missing_tenant_id"
                }),
            )];
        }

        let db = self.db.lock().await;
        let existing: i64 = db
            .query_row(
                "SELECT COUNT(*) FROM tenants WHERE id = ?1",
                params![&id],
                |row| row.get(0),
            )
            .unwrap_or(0);
        if existing > 0 {
            return vec![env.reply(
                "tenant.created",
                serde_json::json!({
                    "error": "tenant_already_exists",
                    "id": id,
                }),
            )];
        }

        let now = chrono::Utc::now().timestamp();
        let max_tokens = safe_u64_to_i64(
            payload
                .get("max_tokens_per_minute")
                .and_then(|v| v.as_u64())
                .unwrap_or(100000),
        );
        let max_requests = safe_u64_to_i64(
            payload
                .get("max_requests_per_minute")
                .and_then(|v| v.as_u64())
                .unwrap_or(100),
        );
        let max_jobs = safe_u64_to_i64(
            payload
                .get("max_concurrent_jobs")
                .and_then(|v| v.as_u64())
                .unwrap_or(5),
        );
        let providers = payload
            .get("allowed_providers")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str().map(String::from))
                    .collect::<Vec<_>>()
            })
            .unwrap_or_else(|| {
                vec![
                    "openai".to_string(),
                    "anthropic".to_string(),
                    "ollama".to_string(),
                ]
            });
        let topics = payload
            .get("allowed_topics")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str().map(String::from))
                    .collect::<Vec<_>>()
            })
            .unwrap_or_else(|| vec!["task.>".to_string(), "agent.>".to_string()]);
        let sandbox = payload
            .get("sandbox_enabled")
            .and_then(|v| v.as_bool())
            .unwrap_or(true);
        let selfdev = payload
            .get("selfdev_enabled")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        if let Err(e) = db.execute(
            "INSERT INTO tenants (id, name, created_at, max_tokens_per_minute, max_requests_per_minute, max_concurrent_jobs, allowed_providers, allowed_topics, sandbox_enabled, selfdev_enabled, status)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
            params![
                &id, &name, now, max_tokens, max_requests, max_jobs,
                serde_json::to_string(&providers).unwrap_or_default(),
                serde_json::to_string(&topics).unwrap_or_default(),
                sandbox as i64, selfdev as i64, "active"
            ],
        ) {
            tracing::error!(error = %e, "failed to insert tenant");
            return vec![env.reply("tenant.created", serde_json::json!({
                "error": "database_error",
                "message": e.to_string(),
            }))];
        }

        if let Err(e) = db.execute(
            "INSERT INTO tenant_usage (tenant_id, tokens_used, tokens_remaining, requests_made, requests_remaining, active_jobs, last_updated)
             VALUES (?1, 0, ?2, 0, ?3, 0, ?4)",
            params![&id, max_tokens, max_requests, now],
        ) {
            tracing::error!(error = %e, "failed to insert tenant usage");
        }
        drop(db);

        tracing::info!(tenant_id = %id, "tenant created");

        let tenant = Tenant {
            id: id.clone(),
            name,
            created_at: now,
            config: TenantConfig {
                max_tokens_per_minute: max_tokens as u64,
                max_requests_per_minute: max_requests as u64,
                max_concurrent_jobs: max_jobs as u32,
                allowed_providers: providers,
                allowed_topics: topics,
                sandbox_enabled: sandbox,
                selfdev_enabled: selfdev,
            },
            status: TenantStatus::Active,
        };

        vec![env.reply(
            "tenant.created",
            serde_json::to_value(&tenant).unwrap_or_default(),
        )]
    }

    async fn handle_configure(&self, env: Envelope<Value>) -> Vec<Envelope<Value>> {
        let payload = &env.payload;
        let id = payload.get("id").and_then(|v| v.as_str()).unwrap_or("");

        if id.is_empty() {
            return vec![env.reply(
                "tenant.configured",
                serde_json::json!({
                    "error": "missing_tenant_id"
                }),
            )];
        }

        let db = self.db.lock().await;
        let existing: i64 = db
            .query_row(
                "SELECT COUNT(*) FROM tenants WHERE id = ?1",
                params![id],
                |row| row.get(0),
            )
            .unwrap_or(0);
        if existing == 0 {
            return vec![env.reply(
                "tenant.configured",
                serde_json::json!({
                    "error": "tenant_not_found",
                    "id": id,
                }),
            )];
        }

        if let Some(max_tokens) = payload
            .get("max_tokens_per_minute")
            .and_then(|v| v.as_u64())
        {
            let _ = db.execute(
                "UPDATE tenants SET max_tokens_per_minute = ?1 WHERE id = ?2",
                params![safe_u64_to_i64(max_tokens), id],
            );
        }
        if let Some(max_requests) = payload
            .get("max_requests_per_minute")
            .and_then(|v| v.as_u64())
        {
            let _ = db.execute(
                "UPDATE tenants SET max_requests_per_minute = ?1 WHERE id = ?2",
                params![safe_u64_to_i64(max_requests), id],
            );
        }
        if let Some(max_jobs) = payload.get("max_concurrent_jobs").and_then(|v| v.as_u64()) {
            let _ = db.execute(
                "UPDATE tenants SET max_concurrent_jobs = ?1 WHERE id = ?2",
                params![safe_u64_to_i64(max_jobs), id],
            );
        }
        if let Some(providers) = payload.get("allowed_providers").and_then(|v| v.as_array()) {
            let list: Vec<String> = providers
                .iter()
                .filter_map(|v| v.as_str().map(String::from))
                .collect();
            let _ = db.execute(
                "UPDATE tenants SET allowed_providers = ?1 WHERE id = ?2",
                params![serde_json::to_string(&list).unwrap_or_default(), id],
            );
        }
        if let Some(sandbox) = payload.get("sandbox_enabled").and_then(|v| v.as_bool()) {
            let _ = db.execute(
                "UPDATE tenants SET sandbox_enabled = ?1 WHERE id = ?2",
                params![sandbox as i64, id],
            );
        }
        drop(db);

        tracing::info!(tenant_id = %id, "tenant configured");

        vec![env.reply(
            "tenant.configured",
            serde_json::json!({
                "status": "configured",
                "id": id,
            }),
        )]
    }

    async fn handle_quota_check(&self, env: Envelope<Value>) -> Vec<Envelope<Value>> {
        let payload = &env.payload;
        let id = payload
            .get("tenant_id")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        let reserve = payload
            .get("reserve")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        let requested_tokens = payload
            .get("requested_tokens")
            .and_then(|v| v.as_u64())
            .unwrap_or(1);
        let requested_requests = payload
            .get("requested_requests")
            .and_then(|v| v.as_u64())
            .unwrap_or(1);

        if id.is_empty() {
            return vec![env.reply(
                "tenant.quota.status",
                serde_json::json!({
                    "error": "missing_tenant_id"
                }),
            )];
        }

        let db = self.db.lock().await;
        let tenant = match db.query_row(
            "SELECT * FROM tenants WHERE id = ?1",
            params![id],
            tenant_from_row,
        ) {
            Ok(t) => t,
            Err(_) => {
                return vec![env.reply(
                    "tenant.quota.status",
                    serde_json::json!({
                        "error": "tenant_not_found",
                        "id": id,
                    }),
                )];
            }
        };

        let now = chrono::Utc::now().timestamp();
        let mut usage = match db.query_row(
            "SELECT * FROM tenant_usage WHERE tenant_id = ?1",
            params![id],
            usage_from_row,
        ) {
            Ok(u) => u,
            Err(_) => TenantUsage {
                tenant_id: id.to_string(),
                tokens_used: 0,
                tokens_remaining: tenant.config.max_tokens_per_minute,
                requests_made: 0,
                requests_remaining: tenant.config.max_requests_per_minute,
                active_jobs: 0,
                last_updated: now,
            },
        };

        // Reset time-windowed quotas every 60 seconds
        if now - usage.last_updated >= 60 {
            usage.tokens_remaining = tenant.config.max_tokens_per_minute;
            usage.requests_remaining = tenant.config.max_requests_per_minute;
            usage.last_updated = now;
            let _ = db.execute(
                "UPDATE tenant_usage SET tokens_remaining = ?1, requests_remaining = ?2, last_updated = ?3 WHERE tenant_id = ?4",
                params![
                    safe_u64_to_i64(usage.tokens_remaining),
                    safe_u64_to_i64(usage.requests_remaining),
                    now,
                    id
                ],
            );
        }

        let allowed = usage.tokens_remaining >= requested_tokens
            && usage.requests_remaining >= requested_requests
            && usage.active_jobs < tenant.config.max_concurrent_jobs;

        if reserve && allowed {
            // Atomically reserve quota inside the same DB lock.
            let new_tokens = usage.tokens_remaining - requested_tokens;
            let new_requests = usage.requests_remaining - requested_requests;
            let new_jobs = usage.active_jobs + 1;
            let _ = db.execute(
                "UPDATE tenant_usage SET tokens_remaining = ?1, requests_remaining = ?2, active_jobs = ?3, last_updated = ?4 WHERE tenant_id = ?5",
                params![
                    safe_u64_to_i64(new_tokens),
                    safe_u64_to_i64(new_requests),
                    safe_u64_to_i64(new_jobs as u64),
                    now,
                    id
                ],
            );
            usage.tokens_remaining = new_tokens;
            usage.requests_remaining = new_requests;
            usage.active_jobs = new_jobs;
        }
        drop(db);

        vec![env.reply(
            "tenant.quota.status",
            serde_json::json!({
                "tenant_id": id,
                "allowed": allowed,
                "reserved": reserve && allowed,
                "usage": usage,
                "limits": {
                    "max_tokens_per_minute": tenant.config.max_tokens_per_minute,
                    "max_requests_per_minute": tenant.config.max_requests_per_minute,
                    "max_concurrent_jobs": tenant.config.max_concurrent_jobs,
                },
            }),
        )]
    }

    async fn handle_isolate(&self, env: Envelope<Value>) -> Vec<Envelope<Value>> {
        let payload = &env.payload;
        let id = payload.get("id").and_then(|v| v.as_str()).unwrap_or("");

        if id.is_empty() {
            return vec![env.reply(
                "tenant.isolated",
                serde_json::json!({
                    "error": "missing_tenant_id"
                }),
            )];
        }

        let db = self.db.lock().await;
        let existing: i64 = db
            .query_row(
                "SELECT COUNT(*) FROM tenants WHERE id = ?1",
                params![id],
                |row| row.get(0),
            )
            .unwrap_or(0);
        if existing == 0 {
            return vec![env.reply(
                "tenant.isolated",
                serde_json::json!({
                    "error": "tenant_not_found",
                    "id": id,
                }),
            )];
        }

        let _ = db.execute(
            "UPDATE tenants SET status = 'suspended' WHERE id = ?1",
            params![id],
        );
        drop(db);

        tracing::warn!(tenant_id = %id, "tenant isolated");

        vec![env.reply(
            "tenant.isolated",
            serde_json::json!({
                "status": "isolated",
                "id": id,
            }),
        )]
    }

    async fn handle_usage_report(&self, env: Envelope<Value>) -> Vec<Envelope<Value>> {
        let payload = &env.payload;
        let id = payload
            .get("tenant_id")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        let tokens = payload.get("tokens").and_then(|v| v.as_u64()).unwrap_or(0);
        let requests = payload
            .get("requests")
            .and_then(|v| v.as_u64())
            .unwrap_or(0);
        let active_jobs = safe_u64_to_i64(
            payload
                .get("active_jobs")
                .and_then(|v| v.as_u64())
                .unwrap_or(0),
        );

        if id.is_empty() {
            return vec![];
        }

        let db = self.db.lock().await;
        let existing: i64 = db
            .query_row(
                "SELECT COUNT(*) FROM tenant_usage WHERE tenant_id = ?1",
                params![id],
                |row| row.get(0),
            )
            .unwrap_or(0);
        if existing == 0 {
            drop(db);
            return vec![];
        }

        let _ = db.execute(
            "UPDATE tenant_usage
             SET tokens_used = tokens_used + ?1,
                 tokens_remaining = max(0, tokens_remaining - ?1),
                 requests_made = requests_made + ?2,
                 requests_remaining = max(0, requests_remaining - ?2),
                 active_jobs = ?3
             WHERE tenant_id = ?4",
            params![
                safe_u64_to_i64(tokens),
                safe_u64_to_i64(requests),
                active_jobs,
                id
            ],
        );
        drop(db);

        vec![]
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();

    let db_path = std::env::var("WIREFRAME_AI_TENANT_DB")
        .unwrap_or_else(|_| "wireframe_ai_tenant.db".to_string());
    let conn = init_db(&db_path)?;

    let module = TenantModule {
        db: Arc::new(Mutex::new(conn)),
    };

    module.run("nats://localhost:4222").await
}
