use derive_more::{Display, Error};
use mobc::{Connection, Pool};
use mobc_redis::{
    redis::{self, AsyncCommands},
    RedisConnectionManager,
};
use serde::{de::DeserializeOwned, Serialize};
use std::time::Duration;

use crate::error::PublicError;

pub type RedisPool = Pool<RedisConnectionManager>;
pub type RedisCon = Connection<RedisConnectionManager>;

const REDIS_CON_STRING: &str = "redis://127.0.0.1:6379/";

const CACHE_POOL_MAX_OPEN: u64 = 16;
const CACHE_POOL_MAX_IDLE: u64 = 8;
const CACHE_POOL_TIMEOUT_SECONDS: u64 = 1;
const CACHE_POOL_EXPIRE_SECONDS: u64 = 60;

#[derive(Debug, Display, Error)]
pub enum Error {
    #[display(fmt = "could not get redis connection from pool : {}", _0)]
    RedisPoolError(mobc::Error<mobc_redis::redis::RedisError>),
    #[display(fmt = "error executing redis command: {}", _0)]
    RedisCMDError(mobc_redis::redis::RedisError),
    #[display(fmt = "error creating Redis client: {}", _0)]
    RedisClientError(mobc_redis::redis::RedisError),
}

pub async fn connect() -> Result<RedisPool, Error> {
    let client = redis::Client::open(REDIS_CON_STRING).map_err(Error::RedisClientError)?;

    let manager = RedisConnectionManager::new(client);

    let pool = Pool::builder()
        .get_timeout(Some(Duration::from_secs(CACHE_POOL_TIMEOUT_SECONDS)))
        .max_lifetime(Some(Duration::from_secs(CACHE_POOL_EXPIRE_SECONDS)))
        .max_open(CACHE_POOL_MAX_OPEN)
        .max_idle(CACHE_POOL_MAX_IDLE)
        .build(manager);

    Ok(pool)
}

async fn get_con(pool: &RedisPool) -> Result<RedisCon, Error> {
    pool.get().await.map_err(|e| {
        eprintln!("error connecting to redis: {}", e);
        Error::RedisPoolError(e).into()
    })
}

// Get or create a cached value with a given TTL in seconds.
// n.b. This only excutes the given closure when the value is not value, expired, or reset=true
pub async fn get_or_create<T, C>(
    pool: &RedisPool,
    key: &str,
    ttl: usize,
    reset: bool,
    callback: C,
) -> Result<T, PublicError>
where
    T: Serialize + DeserializeOwned,
    C: Fn() -> Result<T, PublicError>,
{
    // Get a Redis connection from the pool
    let mut con: RedisCon = get_con(&pool).await?;

    // Get value from cache, if -
    // - We are not force resetting the value, and
    // - The key exists in Redis
    if reset == false {
        let exists: bool = con.exists(key).await.map_err(Error::RedisCMDError)?;
        if exists == true {
            // Cache key found and not expired - pull value and deserialize
            // TODO: Do we need better handling for race-conditations here?
            let res: String = con.get(key).await.map_err(Error::RedisCMDError)?;
            let data: T = serde_json::from_str(&res).unwrap();

            return Ok(data);
        }
    }

    // Cache key not found in Redis, or we are force resetting it -
    // 1. Run the callback,
    // 2. Serialize to JSON string,
    // 3. Save/overwrite value in Redis
    let data: T = callback()?;
    let serialized: String = serde_json::to_string(&data)?;

    con.set_ex(key, serialized, ttl)
        .await
        .map_err(Error::RedisCMDError)?;

    Ok(data)
}
