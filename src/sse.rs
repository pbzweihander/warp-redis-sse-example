use {
    crate::error::RedisOrSerdeJsonError,
    anyhow::Result,
    serde::{Deserialize, Serialize},
    std::time::Duration,
    warp::sse::ServerSentEvent,
};

#[derive(Debug, Deserialize, Serialize)]
pub struct SseMessage {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    data: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    event: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    retry: Option<Duration>,
}

impl SseMessage {
    fn into_sse(self) -> Option<impl ServerSentEvent> {
        macro_rules! append_sse {
            ($sse:expr, $ty:path, $other:expr) => {
                if let Some(other) = $other {
                    $sse = Some(if let Some(inner_sse) = $sse {
                        ($ty(other), inner_sse).boxed()
                    } else {
                        $ty(other).boxed()
                    })
                }
            };
        }
        let mut sse = None;
        if let Some(id) = self.id {
            sse = Some(warp::sse::id(id).boxed());
        }
        append_sse!(sse, warp::sse::data, self.data);
        append_sse!(sse, warp::sse::event, self.event);
        append_sse!(sse, warp::sse::retry, self.retry);
        sse
    }
}

pub fn redis_msg_to_sse(
    msg: redis::Msg,
) -> Result<Option<impl ServerSentEvent>, RedisOrSerdeJsonError> {
    let string_msg: String = msg.get_payload()?;
    let sse_msg: SseMessage = serde_json::from_str(&string_msg)?;
    Ok(sse_msg.into_sse())
}
