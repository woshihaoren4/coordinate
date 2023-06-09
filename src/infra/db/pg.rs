// use sqlx::PgPool;
// use crate::config;
// #[warn(dead_code)]
// pub async fn init_pgsql(cfg:config::PGSql) ->anyhow::Result<PgPool>{
//     let pool = sqlx::postgres::PgPoolOptions::new()
//         .max_connections(cfg.max_conn_size)
//         .min_connections(cfg.max_idle_conn)
//         .connect(cfg.url.as_str()).await?;Ok(pool)
// }
