use {
    crate::{
        app::App,
        error::from_anyhow,
        redis::make_redis_pubsub_stream,
        sse::{redis_msg_to_sse, SseMessage},
    },
    anyhow::Result,
    futures::prelude::*,
    redis::AsyncCommands,
    std::sync::Arc,
    warp::Filter,
};

pub fn apis(app: App) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    let app = Arc::new(app);
    let get = warp::path!(String / "messages")
        .and(warp::get())
        .and(with_app(app.clone()))
        .and_then(handle_get_messages);
    let post = warp::path!(String / "messages")
        .and(warp::post())
        .and(warp::body::json())
        .and(with_app(app))
        .and_then(handle_post_messages);
    get.or(post)
}

fn with_app(
    app: Arc<App>,
) -> impl Filter<Extract = (Arc<App>,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || app.clone())
}

async fn handle_get_messages(
    channel: String,
    app: Arc<App>,
) -> Result<impl warp::Reply, warp::Rejection> {
    let mut pubsub = app.get_redis_pubsub().await.map_err(from_anyhow)?;
    pubsub.subscribe(&channel).await.map_err(from_anyhow)?;
    let stream = make_redis_pubsub_stream(pubsub);
    let stream = stream
        .inspect(move |msg| {
            if let Ok(msg) = msg.get_payload::<String>() {
                log::info!("Subscribe from {}: {}", channel, msg);
            }
        })
        .map(redis_msg_to_sse)
        .try_filter_map(future::ok);
    Ok(warp::sse::reply(warp::sse::keep_alive().stream(stream)))
}

async fn handle_post_messages(
    channel: String,
    msg: SseMessage,
    app: Arc<App>,
) -> Result<impl warp::Reply, warp::Rejection> {
    let mut conn = app.get_redis_connection().await.map_err(from_anyhow)?;
    let msg = serde_json::to_string(&msg).map_err(from_anyhow)?;
    conn.publish(&channel, &msg).await.map_err(from_anyhow)?;
    log::info!("Published to {}: {}", channel, msg);
    Ok(warp::reply::json(&serde_json::json!({"result": "success"})))
}
