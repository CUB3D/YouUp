#[macro_export]
macro_rules! get_pool {
    ($pool: ident) => {{
        let pool = match $pool.get() {
            Ok(pool) => pool,
            Err(e) => {
                tracing::warn!("Failed to get pool: {:?}", e);
                return HttpResponse::InternalServerError().finish();
            }
        };
        pool
    }};
}
