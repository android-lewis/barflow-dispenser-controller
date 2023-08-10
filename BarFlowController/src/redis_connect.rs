use std::env;
use redis;
use redis::Commands;
use redis::RedisResult;

pub fn connect() -> redis::Connection {
    //format - host:port
    let redis_host_name =
        env::var("REDIS_HOSTNAME").expect("missing environment variable REDIS_HOSTNAME");
    
    let redis_password = env::var("REDIS_PASS").unwrap_or_default();
    //if Redis server needs secure connection
    let uri_scheme = match env::var("IS_TLS") {
        Ok(_) => "rediss",
        Err(_) => "redis",
    };
    let redis_conn_url = format!("{}://:{}@{}", uri_scheme, redis_password, redis_host_name);
    redis::Client::open(redis_conn_url)
        .expect("Invalid connection URL")
        .get_connection()
        .expect("failed to connect to Redis")
}

pub fn get_value(key: &str) -> RedisResult<f32>{
    let mut conn = connect();
    let flow : f32 = conn.get(key)?;

    Ok(flow)
}

pub fn set_value(key: &str, val: f32) -> RedisResult<()> {
    let mut conn = connect();
    let _ : () = conn.set(key, val)?;
    Ok(())
}