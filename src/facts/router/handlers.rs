use axum::{
    debug_handler,
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::get,
    Json,
    Router,
};

use super::{dtos::HttpEntity, errors::AppError, state::AppState};

#[derive(Default)]
pub struct AppRouter {}

#[debug_handler]
pub async fn get_fact(
    Path(id): Path<i32>,
    State(state): State<AppState>,
) -> Result<impl IntoResponse, AppError> {
    let result: HttpEntity = state.facts.get(id).await?.into();

    Ok((StatusCode::OK, Json(result)))
}

#[debug_handler]
pub async fn get_random_fact(State(state): State<AppState>) -> Result<impl IntoResponse, AppError> {
    let result: HttpEntity = state.facts.get_random().await?.into();

    Ok((StatusCode::OK, Json(result)))
}

#[debug_handler]
pub async fn health(State(state): State<AppState>) -> Result<impl IntoResponse, AppError> {
    if state.facts.get_random().await.is_ok() {
        Ok((StatusCode::OK, Json("Healthy")))
    } else {
        Ok((StatusCode::SERVICE_UNAVAILABLE, Json("Unhealthy")))
    }
}

impl From<AppRouter> for Router<AppState> {
    fn from(_: AppRouter) -> Self {
        Router::new()
            .route("/{id}", get(get_fact))
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
    use reqwest::Method;
    use serde_json::from_slice;
    use sqlx::{query, query_scalar, PgPool};
    use tower::ServiceExt;

    use super::*;
    use crate::facts::{repository::Fact, SqlxFactsRepository};

    #[sqlx::test(
        migrations = "./src/facts/migrations",
        fixtures(path = "fixtures", scripts("truncate_facts_table"))
    )]
    async fn get_ok(pool: PgPool) {
        let router: Router<AppState> = AppRouter::default().into();
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
        };

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

        let response =
            from_slice::<HttpEntity>(&raw_response.into_body().collect().await.unwrap().to_bytes())
                .unwrap();
        let result = Fact::new(response.id(), response.title(), response.body()).unwrap();

        assert_eq!(entity.body(), result.body());
        assert_eq!(entity.title(), result.title());
    }

    #[sqlx::test(
        migrations = "./src/facts/migrations",
        fixtures(path = "fixtures", scripts("truncate_facts_table"))
    )]
    async fn get_non_existent(pool: PgPool) {
        let router: Router<AppState> = AppRouter::default().into();
        let id: i32 = Faker.fake();

        let state = AppState {
            facts: Arc::new(SqlxFactsRepository::new(pool)),
        };

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

        assert_eq!(raw_response.status(), StatusCode::NOT_FOUND);
    }

    #[sqlx::test(
        migrations = "./src/facts/migrations",
        fixtures(path = "fixtures", scripts("truncate_facts_table"))
    )]
    async fn get_random(pool: PgPool) {
        let router: Router<AppState> = AppRouter::default().into();

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
        };

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

        from_slice::<HttpEntity>(&raw_response.into_body().collect().await.unwrap().to_bytes())
            .unwrap();
    }

    #[sqlx::test(
        migrations = "./src/facts/migrations",
        fixtures(path = "fixtures", scripts("truncate_facts_table"))
    )]
    async fn get_random_from_empty(pool: PgPool) {
        let router: Router<AppState> = AppRouter::default().into();

        let state = AppState {
            facts: Arc::new(SqlxFactsRepository::new(pool)),
        };

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
    async fn healthcheck(pool: PgPool) {
        let router: Router<AppState> = AppRouter::default().into();
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
        };

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
