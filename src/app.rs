use {
    crate::{
        api::apis,
        config::AppConfig,
        error::unpack_problem,
        redis::{make_redis_pool, RedisConnection, RedisPool, RedisPubSub},
    },
    anyhow::Result,
    warp::Filter,
};

#[derive(Debug)]
pub struct App {
    pub config: AppConfig,
    pub redis_pool: RedisPool,
}

impl App {
    pub async fn from_config(config: AppConfig) -> Result<Self> {
        let redis_pool = make_redis_pool(&config.redis).await?;
        Ok(Self { config, redis_pool })
    }

    pub fn into_filter(
        self,
    ) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        apis(self)
            .or(warp::path!("ping").map(|| "ok"))
            .recover(unpack_problem)
            .with(warp::log(env!("CARGO_PKG_NAME")))
    }

    pub async fn get_redis_connection(&self) -> Result<RedisConnection<'_>> {
        Ok(self.redis_pool.get().await?)
    }

    pub async fn get_redis_pubsub(&self) -> Result<RedisPubSub> {
        Ok(self.redis_pool.dedicated_connection().await?.into_pubsub())
    }
}
