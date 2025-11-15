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
    Path(id): Path<u64>,
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
    use std::{collections::HashMap, sync::Arc};

    use axum::{body::Body, http::Request};
    use fake::{Fake, Faker};
    use http_body_util::BodyExt;
    use reqwest::Method;
    use serde_json::from_slice;
    use tower::ServiceExt;

    use super::*;
    use crate::facts::dao::{Entity, HashMapDao};

    impl Default for AppState {
        fn default() -> Self {
            Self {
                dao: Arc::new(HashMapDao::new(HashMap::new())),
            }
        }
    }

    #[tokio::test]
    async fn get_ok() {
        let router: Router<AppState> = AppRouter::default().into();
        let entity = Faker.fake::<Entity>();

        let predefined_dao = HashMapDao::new(HashMap::from_iter([(entity.id(), entity.clone())]));

        let state = AppState {
            dao: Arc::new(predefined_dao),
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
        println!("{response:#?}");

        assert_eq!(entity.id(), response.id());
        assert_eq!(entity.title(), response.title());
        assert_eq!(entity.body(), response.body());
    }

    #[tokio::test]
    async fn get_non_existent() {
        let router: Router<AppState> = AppRouter::default().into();
        let id: u64 = Faker.fake();

        let raw_response = router
            .with_state(AppState::default())
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
        let mut map = HashMap::new();

        for _ in 0..10 {
            let entity = Faker.fake::<Entity>();
            map.insert(entity.id(), entity);
        }

        let predefined_dao = HashMapDao::new(map);

        let state = AppState {
            dao: Arc::new(predefined_dao),
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

        let raw_response = router
            .with_state(AppState::default())
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
        let mut map = HashMap::new();

        for _ in 0..10 {
            let entity = Faker.fake::<Entity>();
            map.insert(entity.id(), entity);
        }

        let predefined_dao = HashMapDao::new(map);

        let state = AppState {
            dao: Arc::new(predefined_dao),
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
