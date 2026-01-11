use argon2::{Argon2, PasswordHash, PasswordVerifier};
use axum::{
    debug_handler,
    extract::{Path, Request, State},
    http::StatusCode,
    middleware::{from_fn_with_state, Next},
    response::IntoResponse,
    routing::{delete, get, post},
    Json,
    Router,
};
use axum_extra::{
    headers::{authorization::Basic, Authorization},
    TypedHeader,
};

use super::{
    errors::AppError,
    models::{HttpCreateFactRequestBody, HttpFactResponse},
    state::AppState,
};
use crate::facts::repository::{CreateFactRequest, FactId};

pub struct AppRouter {
    state: AppState,
}

impl AppRouter {
    pub fn new(state: AppState) -> Self {
        Self { state }
    }
}

#[debug_handler]
pub async fn get_fact(
    Path(id): Path<i32>,
    State(state): State<AppState>,
) -> Result<impl IntoResponse, AppError> {
    let id = FactId::new(id)?;
    let result: HttpFactResponse = state.facts.get(id).await?.into();

    Ok((StatusCode::OK, Json(result)))
}

#[debug_handler]
pub async fn get_random_fact(State(state): State<AppState>) -> Result<impl IntoResponse, AppError> {
    let result: HttpFactResponse = state.facts.get_random().await?.into();

    Ok((StatusCode::OK, Json(result)))
}

#[debug_handler]
pub async fn create_fact(
    State(state): State<AppState>,
    Json(body): Json<HttpCreateFactRequestBody>,
) -> Result<impl IntoResponse, AppError> {
    let request: CreateFactRequest = body.try_into()?;
    let result: HttpFactResponse = state.facts.create(&request).await?.into();

    Ok((StatusCode::CREATED, Json(result)))
}

#[debug_handler]
pub async fn delete_fact(
    State(state): State<AppState>,
    Path(id): Path<i32>,
) -> Result<impl IntoResponse, AppError> {
    let id = FactId::new(id)?;
    state.facts.delete(id).await?;

    Ok(StatusCode::NO_CONTENT)
}

#[debug_handler]
pub async fn health(State(state): State<AppState>) -> Result<impl IntoResponse, AppError> {
    if state.facts.get_random().await.is_ok() {
        Ok((StatusCode::OK, Json("Healthy")))
    } else {
        Ok((StatusCode::SERVICE_UNAVAILABLE, Json("Unhealthy")))
    }
}

pub async fn auth_middleware(
    State(state): State<AppState>,
    TypedHeader(auth): TypedHeader<Authorization<Basic>>,
    request: Request,
    next: Next,
) -> Result<impl IntoResponse, AppError> {
    let hashed = PasswordHash::new(&state.auth_key).map_err(|err| AppError {
        status_code: StatusCode::INTERNAL_SERVER_ERROR,
        details: format!("Auth failed: Can't hash the stored API key: {err}").to_owned(),
    })?;
    let input = auth.password().as_bytes();

    Argon2::default()
        .verify_password(input, &hashed)
        .map_err(|_| AppError {
            status_code: StatusCode::FORBIDDEN,
            details: "Auth failed: Hashes mismatch".to_owned(),
        })?;

    Ok(next.run(request).await)
}

impl From<AppRouter> for Router<AppState> {
    fn from(app_router: AppRouter) -> Self {
        Router::new()
            .route(
                "/",
                post(create_fact).route_layer(from_fn_with_state(
                    app_router.state.clone(),
                    auth_middleware,
                )),
            )
            .route("/{id}", get(get_fact))
            .route(
                "/{id}",
                delete(delete_fact)
                    .route_layer(from_fn_with_state(app_router.state, auth_middleware)),
            )
            .route("/random", get(get_random_fact))
            .route("/health", get(health))
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use axum::{body::Body, http::Request};
    use fake::{Fake, Faker};
    use http_body_util::BodyExt;
    use reqwest::{
        header::{AUTHORIZATION, CONTENT_TYPE},
        Method,
    };
    use serde_json::from_slice;
    use sqlx::{query, query_scalar, PgPool};
    use tower::ServiceExt;

    use super::*;
    use crate::facts::{
        repository::{Fact, FactBody, FactTitle},
        SqlxFactsRepository,
    };

    #[sqlx::test(
        migrations = "./src/facts/migrations",
        fixtures(path = "fixtures", scripts("truncate_facts_table"))
    )]
    async fn get_ok(pool: PgPool) {
        let entity = Faker.fake::<Fact>();

        let id = query_scalar!(
            "INSERT INTO facts (title, body) VALUES ($1, $2) RETURNING id",
            Into::<String>::into(entity.title().to_owned()),
            Into::<String>::into(entity.body().to_owned())
        )
        .fetch_one(&pool)
        .await
        .unwrap();

        let state = AppState {
            facts: Arc::new(SqlxFactsRepository::new(pool)),
            ..Default::default()
        };

        let router: Router<AppState> = AppRouter::new(state.clone()).into();
        let raw_response = router
            .with_state(state)
            .oneshot(
                Request::builder()
                    .method(Method::GET)
                    .uri(format!("/{id}"))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(raw_response.status(), StatusCode::OK);

        let response = from_slice::<HttpFactResponse>(
            &raw_response.into_body().collect().await.unwrap().to_bytes(),
        )
        .unwrap();
        let result = Fact::new(
            FactId::new(response.id()).unwrap(),
            &FactTitle::new(response.title()).unwrap(),
            &FactBody::new(response.body()).unwrap(),
        );

        assert_eq!(entity.body(), result.body());
        assert_eq!(entity.title(), result.title());
    }

    #[sqlx::test(
        migrations = "./src/facts/migrations",
        fixtures(path = "fixtures", scripts("truncate_facts_table"))
    )]
    async fn get_non_existent(pool: PgPool) {
        let state = AppState {
            facts: Arc::new(SqlxFactsRepository::new(pool)),
            ..Default::default()
        };
        let router: Router<AppState> = AppRouter::new(state.clone()).into();

        let raw_response = router
            .with_state(state)
            .oneshot(
                Request::builder()
                    .method(Method::GET)
                    .uri("/1")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(raw_response.status(), StatusCode::NOT_FOUND);
    }

    #[sqlx::test(
        migrations = "./src/facts/migrations",
        fixtures(path = "fixtures", scripts("truncate_facts_table"))
    )]
    async fn get_random(pool: PgPool) {
        for _ in 0..10 {
            let entity = Faker.fake::<Fact>();

            query!(
                "INSERT INTO facts (title, body) VALUES ($1, $2)",
                Into::<String>::into(entity.title().to_owned()),
                Into::<String>::into(entity.body().to_owned())
            )
            .execute(&pool)
            .await
            .unwrap();
        }

        let state = AppState {
            facts: Arc::new(SqlxFactsRepository::new(pool)),
            ..Default::default()
        };

        let router: Router<AppState> = AppRouter::new(state.clone()).into();

        let raw_response = router
            .with_state(state)
            .oneshot(
                Request::builder()
                    .method(Method::GET)
                    .uri("/random")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(raw_response.status(), StatusCode::OK);

        from_slice::<HttpFactResponse>(
            &raw_response.into_body().collect().await.unwrap().to_bytes(),
        )
        .unwrap();
    }

    #[sqlx::test(
        migrations = "./src/facts/migrations",
        fixtures(path = "fixtures", scripts("truncate_facts_table"))
    )]
    async fn get_random_from_empty(pool: PgPool) {
        let state = AppState {
            facts: Arc::new(SqlxFactsRepository::new(pool)),
            ..Default::default()
        };
        let router: Router<AppState> = AppRouter::new(state.clone()).into();

        let raw_response = router
            .with_state(state)
            .oneshot(
                Request::builder()
                    .method(Method::GET)
                    .uri("/random")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(raw_response.status(), StatusCode::NOT_FOUND);
    }

    #[sqlx::test(
        migrations = "./src/facts/migrations",
        fixtures(path = "fixtures", scripts("truncate_facts_table"))
    )]
    async fn create_ok(pool: PgPool) {
        let state = AppState {
            facts: Arc::new(SqlxFactsRepository::new(pool)),
            ..Default::default()
        };

        let router: Router<AppState> = AppRouter::new(state.clone()).into();
        let raw_response = router
            .with_state(state)
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri("/")
                    .header(CONTENT_TYPE.as_str(), "application/json")
                    .header(AUTHORIZATION, "Basic Og==")
                    .body(Body::from(r#"{"title": "foo", "body": "bar"}"#))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(raw_response.status(), StatusCode::CREATED);

        let response = from_slice::<HttpFactResponse>(
            &raw_response.into_body().collect().await.unwrap().to_bytes(),
        )
        .unwrap();

        assert_eq!(response.body(), "bar");
        assert_eq!(response.title(), "foo");
    }

    #[sqlx::test(
        migrations = "./src/facts/migrations",
        fixtures(path = "fixtures", scripts("truncate_facts_table"))
    )]
    async fn delete_non_existent(pool: PgPool) {
        let state = AppState {
            facts: Arc::new(SqlxFactsRepository::new(pool)),
            ..Default::default()
        };

        let router: Router<AppState> = AppRouter::new(state.clone()).into();
        let raw_response = router
            .with_state(state)
            .oneshot(
                Request::builder()
                    .method(Method::DELETE)
                    .uri("/1")
                    .header(AUTHORIZATION, "Basic Og==")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(raw_response.status(), StatusCode::NOT_FOUND);
    }

    #[sqlx::test(
        migrations = "./src/facts/migrations",
        fixtures(path = "fixtures", scripts("truncate_facts_table"))
    )]
    async fn delete_ok(pool: PgPool) {
        let entity = Faker.fake::<Fact>();

        let id = query_scalar!(
            "INSERT INTO facts (title, body) VALUES ($1, $2) RETURNING id",
            Into::<String>::into(entity.title().to_owned()),
            Into::<String>::into(entity.body().to_owned())
        )
        .fetch_one(&pool)
        .await
        .unwrap();

        let state = AppState {
            facts: Arc::new(SqlxFactsRepository::new(pool)),
            ..Default::default()
        };

        let router: Router<AppState> = AppRouter::new(state.clone()).into();
        let raw_response = router
            .with_state(state)
            .oneshot(
                Request::builder()
                    .method(Method::DELETE)
                    .uri(format!("/{id}"))
                    .header(AUTHORIZATION, "Basic Og==")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(raw_response.status(), StatusCode::NO_CONTENT);
    }

    #[sqlx::test(
        migrations = "./src/facts/migrations",
        fixtures(path = "fixtures", scripts("truncate_facts_table"))
    )]
    async fn healthcheck(pool: PgPool) {
        let entity = Faker.fake::<Fact>();

        query!(
            "INSERT INTO facts (title, body) VALUES ($1, $2)",
            Into::<String>::into(entity.title().to_owned()),
            Into::<String>::into(entity.body().to_owned())
        )
        .execute(&pool)
        .await
        .unwrap();

        let state = AppState {
            facts: Arc::new(SqlxFactsRepository::new(pool)),
            ..Default::default()
        };
        let router: Router<AppState> = AppRouter::new(state.clone()).into();

        let raw_response = router
            .with_state(state)
            .oneshot(
                Request::builder()
                    .method(Method::GET)
                    .uri("/health")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(raw_response.status(), StatusCode::OK);
    }
}
