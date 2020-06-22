use {http_api_problem::HttpApiProblem, warp::Rejection};

#[derive(Debug)]
pub enum RedisOrSerdeJsonError {
    Redis(redis::RedisError),
    SerdeJson(serde_json::Error),
}

impl From<redis::RedisError> for RedisOrSerdeJsonError {
    fn from(err: redis::RedisError) -> Self {
        Self::Redis(err)
    }
}

impl From<serde_json::Error> for RedisOrSerdeJsonError {
    fn from(err: serde_json::Error) -> Self {
        Self::SerdeJson(err)
    }
}

impl std::error::Error for RedisOrSerdeJsonError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Redis(err) => err.source(),
            Self::SerdeJson(err) => err.source(),
        }
    }
}

impl std::fmt::Display for RedisOrSerdeJsonError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::Redis(err) => err.fmt(f),
            Self::SerdeJson(err) => err.fmt(f),
        }
    }
}

pub fn from_anyhow(e: impl Into<anyhow::Error>) -> Rejection {
    let e: anyhow::Error = e.into();
    let problem = match e.downcast::<HttpApiProblem>() {
        Ok(problem) => problem,
        Err(e) => HttpApiProblem::new(format!("Internal Server Error\n{:?}", e))
            .set_status(warp::http::StatusCode::INTERNAL_SERVER_ERROR),
    };
    warp::reject::custom(problem)
}

pub async fn unpack_problem(rejection: Rejection) -> Result<impl warp::Reply, Rejection> {
    if let Some(problem) = rejection.find::<HttpApiProblem>() {
        let code = problem
            .status
            .unwrap_or(warp::http::StatusCode::INTERNAL_SERVER_ERROR);

        let reply = warp::reply::json(problem);
        let reply = warp::reply::with_status(reply, code);
        let reply = warp::reply::with_header(
            reply,
            warp::http::header::CONTENT_TYPE,
            http_api_problem::PROBLEM_JSON_MEDIA_TYPE,
        );

        return Ok(reply);
    }

    Err(rejection)
}
