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
    Path(id): Path<i64>,
    State(state): State<AppState>,
) -> Result<impl IntoResponse, AppError> {
    let result: HttpEntity = state.dao.get(id).await?.into();

    Ok((StatusCode::OK, Json(result)))
}

#[debug_handler]
pub async fn get_random_fact(State(state): State<AppState>) -> Result<impl IntoResponse, AppError> {
    let result: HttpEntity = state.dao.get_random().await?.into();

    Ok((StatusCode::OK, Json(result)))
}

#[debug_handler]
pub async fn health(State(state): State<AppState>) -> Result<impl IntoResponse, AppError> {
    if state.dao.get_random().await.is_ok() {
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
    use sqlx::{
        any::{install_default_drivers, AnyPoolOptions},
        migrate::Migrator,
        query,
        AnyPool,
    };
    use tower::ServiceExt;

    use super::*;
    use crate::facts::{dao::Entity, SqlxDao};

    static MIGRATOR: Migrator = sqlx::migrate!("./src/facts/migrations");

    async fn setup() -> AnyPool {
        install_default_drivers();
        let pool = AnyPoolOptions::new()
            .max_connections(1)
            .connect("sqlite::memory:")
            .await
            .unwrap();

        MIGRATOR.run(&pool).await.unwrap();

        pool
    }

    #[tokio::test]
    async fn get_ok() {
        let router: Router<AppState> = AppRouter::default().into();
        let entity = Faker.fake::<Entity>();
        let pool = setup().await;

        query("INSERT INTO facts (id, title, body) VALUES ($1, $2, $3)")
            .bind(entity.id())
            .bind(entity.title())
            .bind(entity.body())
            .execute(&pool)
            .await
            .unwrap();

        let state = AppState {
            dao: Arc::new(SqlxDao::new(pool)),
        };

        let raw_response = router
            .with_state(state)
            .oneshot(
                Request::builder()
                    .method(Method::GET)
                    .uri(format!("/{}", entity.id()))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(raw_response.status(), StatusCode::OK);

        let response =
            from_slice::<HttpEntity>(&raw_response.into_body().collect().await.unwrap().to_bytes())
                .unwrap();

        assert_eq!(entity.id(), response.id());
        assert_eq!(entity.title(), response.title());
        assert_eq!(entity.body(), response.body());
    }

    #[tokio::test]
    async fn get_non_existent() {
        let router: Router<AppState> = AppRouter::default().into();
        let id: i64 = Faker.fake();
        let pool = setup().await;

        let state = AppState {
            dao: Arc::new(SqlxDao::new(pool)),
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

    #[tokio::test]
    async fn get_random() {
        let router: Router<AppState> = AppRouter::default().into();
        let pool = setup().await;

        for _ in 0..10 {
            let entity = Faker.fake::<Entity>();

            query("INSERT INTO facts (id, title, body) VALUES ($1, $2, $3)")
                .bind(entity.id())
                .bind(entity.title())
                .bind(entity.body())
                .execute(&pool)
                .await
                .unwrap();
        }

        let state = AppState {
            dao: Arc::new(SqlxDao::new(pool)),
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

    #[tokio::test]
    async fn get_random_from_empty() {
        let router: Router<AppState> = AppRouter::default().into();
        let pool = setup().await;

        let state = AppState {
            dao: Arc::new(SqlxDao::new(pool)),
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

    #[tokio::test]
    async fn healthcheck() {
        let router: Router<AppState> = AppRouter::default().into();
        let pool = setup().await;
        let entity = Faker.fake::<Entity>();

        query("INSERT INTO facts (id, title, body) VALUES ($1, $2, $3);")
            .bind(entity.id())
            .bind(entity.title())
            .bind(entity.body())
            .execute(&pool)
            .await
            .unwrap();

        let state = AppState {
            dao: Arc::new(SqlxDao::new(pool)),
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
