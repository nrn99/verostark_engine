use sqlx::postgres::PgPoolOptions;
use sqlx::{Pool, Postgres};
use std::collections::HashMap;
use std::time::Duration;
use uuid::Uuid;
use log::{info, error};

#[derive(Clone)]
pub struct Db {
    pool: Pool<Postgres>,
}

impl Db {
    pub async fn new(database_url: &str) -> Result<Self, sqlx::Error> {
        let pool = PgPoolOptions::new()
            .max_connections(20)
            .acquire_timeout(Duration::from_secs(3))
            .connect(database_url)
            .await?;

        // Run migrations
        sqlx::migrate!("./migrations")
            .run(&pool)
            .await?;

        info!("✅ [Persistence] Connected to PostgreSQL & Migrations Applied.");
        Ok(Db { pool })
    }

    pub async fn save_request_log(
        &self,
        request_id: &str,
        client_ip: &str,
        upstream_status: u16,
        latency_ms: u128,
        risk_verdict: &str,
        scrub_count: usize,
        scrub_details: &HashMap<String, usize>,
    ) {
        let uuid = match Uuid::parse_str(request_id) {
            Ok(u) => u,
            Err(e) => {
                error!("Invalid Request ID UUID: {}. Error: {}", request_id, e);
                return;
            }
        };

        let scrub_details_json = serde_json::to_value(scrub_details).unwrap_or(serde_json::json!({}));

        let query = sqlx::query(
            r#"
            INSERT INTO request_audits (
                request_id, timestamp, client_ip, upstream_status, latency_ms, 
                risk_verdict, scrub_count, scrub_details
            )
            VALUES ($1, NOW(), $2, $3, $4, $5, $6, $7)
            "#)
            .bind(uuid)
            .bind(client_ip)
            .bind(upstream_status as i16) // Postgres SMALLINT is i16
            .bind(latency_ms as i32)      // Postgres INT is i32
            .bind(risk_verdict)
            .bind(scrub_count as i32)
            .bind(scrub_details_json);

        if let Err(e) = query.execute(&self.pool).await {
            error!("❌ [Persistence] Failed to save audit log: {}", e);
        }
    }
}
