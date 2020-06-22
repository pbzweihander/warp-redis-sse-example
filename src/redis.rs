use {
    crate::config::RedisConfig,
    anyhow::Result,
    bb8::{Pool, PooledConnection},
    bb8_redis::RedisConnectionManager,
    futures::{
        prelude::*,
        ready,
        task::{AtomicWaker, Context, Poll},
    },
    std::{pin::Pin, sync::Arc},
    tokio::sync::mpsc,
};

pub use redis::aio::PubSub as RedisPubSub;
pub type RedisPool = Pool<RedisConnectionManager>;
pub type RedisConnection<'a> = PooledConnection<'a, RedisConnectionManager>;

pub async fn make_redis_pool(config: &RedisConfig) -> Result<RedisPool> {
    let manager = RedisConnectionManager::new(config.url.clone())?;
    let pool = Pool::builder().build(manager).await?;

    Ok(pool)
}

struct RedisSub {
    receiver: mpsc::Receiver<redis::Msg>,
    waker: Arc<AtomicWaker>,
}

impl Stream for RedisSub {
    type Item = redis::Msg;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context) -> Poll<Option<Self::Item>> {
        self.receiver.poll_next_unpin(cx)
    }
}

impl Drop for RedisSub {
    fn drop(&mut self) {
        self.waker.wake();
    }
}

pub fn make_redis_pubsub_stream(mut pubsub: RedisPubSub) -> impl Stream<Item = redis::Msg> {
    let (mut sender, receiver) = mpsc::channel(1);
    let waker = Arc::new(AtomicWaker::new());
    let w = waker.clone();

    let stream_fut = async move {
        let mut stream = pubsub.on_message();
        stream::poll_fn(move |mut ctx| {
            w.register(ctx.waker());
            let res = ready!(sender.poll_ready(&mut ctx));
            if res.is_err() {
                Poll::Ready(None)
            } else {
                match stream.poll_next_unpin(&mut ctx) {
                    Poll::Ready(Some(item)) => {
                        let res = sender.try_send(item);
                        if res.is_err() {
                            Poll::Ready(None)
                        } else {
                            Poll::Ready(Some(()))
                        }
                    }
                    Poll::Ready(None) => Poll::Ready(None),
                    Poll::Pending => {
                        sender.disarm();
                        Poll::Pending
                    }
                }
            }
        })
        .collect::<()>()
        .await;
        log::info!("Connection dropped");
    };
    tokio::spawn(stream_fut);

    RedisSub { receiver, waker }
}
