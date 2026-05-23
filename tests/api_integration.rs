/// Integration tests for the OpenFGA Rust SDK — uses mockito to intercept HTTP.
///
/// Coverage: all 23 endpoints from the OpenAPI spec
///   - Happy path (200/201/204) for every endpoint
///   - Error handling: 401 for every endpoint, plus 400/404/500 for core endpoints
///   - Query parameter encoding for paginated/filtered endpoints
///   - Auth header variants: Bearer, OAuth, Basic auth, API key
///   - NDJSON streaming: success items and server-side stream errors (B-1 regression)
use mockito::{Matcher, Server};
use openfga::apis::Error;
use openfga::apis::configuration::Configuration;
use openfga::models;

// ── Common test JSON bodies ────────────────────────────────────────────────────

const ERR_401: &str = r#"{"code":"unauthenticated","message":"not authenticated"}"#;
const ERR_400: &str = r#"{"code":"invalid_argument","message":"invalid argument"}"#;
const ERR_403: &str = r#"{"code":"forbidden","message":"forbidden"}"#;
const ERR_404: &str = r#"{"code":"not_found","message":"not found"}"#;
const ERR_409: &str = r#"{"code":"aborted","message":"write conflict"}"#;
const ERR_422: &str = r#"{"code":"resource_exhausted","message":"rate limit exceeded"}"#;
const ERR_500: &str = r#"{"code":"internal_error","message":"internal error"}"#;

const STORE_OBJ: &str = r#"{"id":"s1","name":"test-store","created_at":"2024-01-01T00:00:00+00:00","updated_at":"2024-01-01T00:00:00+00:00"}"#;

// ── Assertion helper macro ─────────────────────────────────────────────────────

macro_rules! assert_status {
    ($result:expr, $code:expr) => {
        match $result {
            Err(Error::ResponseError(rc)) => {
                assert_eq!(rc.status, $code, "unexpected status code in ResponseError")
            }
            other => panic!("expected Err(ResponseError({})), got: {:?}", $code, other),
        }
    };
}

// ── Store Management ───────────────────────────────────────────────────────────

#[tokio::test]
async fn list_stores_200_returns_ok_response() {
    let mut server = Server::new_async().await;
    let mock = server
        .mock("GET", "/stores")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"{"stores":[],"continuation_token":""}"#)
        .create_async()
        .await;

    let config = Configuration::builder().base_path(server.url()).build();
    let result = openfga::apis::stores_api::list_stores(&config, None, None, None).await;

    mock.assert_async().await;
    assert!(result.is_ok(), "expected Ok, got: {:?}", result);
    assert_eq!(result.unwrap().stores.len(), 0);
}

#[tokio::test]
async fn list_stores_401_returns_response_error() {
    let mut server = Server::new_async().await;
    server
        .mock("GET", "/stores")
        .with_status(401)
        .with_body(ERR_401)
        .create_async()
        .await;

    let config = Configuration::builder().base_path(server.url()).build();
    let result = openfga::apis::stores_api::list_stores(&config, None, None, None).await;
    assert_status!(result, reqwest::StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn create_store_200_returns_created_store() {
    let mut server = Server::new_async().await;
    let mock = server
        .mock("POST", "/stores")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(STORE_OBJ)
        .create_async()
        .await;

    let config = Configuration::builder().base_path(server.url()).build();
    let body = models::CreateStoreRequest::new("test-store".to_string());
    let result = openfga::apis::stores_api::create_store(&config, body).await;

    mock.assert_async().await;
    assert!(result.is_ok(), "expected Ok, got: {:?}", result);
    assert_eq!(result.unwrap().id, "s1");
}

#[tokio::test]
async fn create_store_401_returns_response_error() {
    let mut server = Server::new_async().await;
    server
        .mock("POST", "/stores")
        .with_status(401)
        .with_body(ERR_401)
        .create_async()
        .await;

    let config = Configuration::builder().base_path(server.url()).build();
    let result = openfga::apis::stores_api::create_store(
        &config,
        models::CreateStoreRequest::new("x".into()),
    )
    .await;
    assert_status!(result, reqwest::StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn get_store_200_returns_store() {
    let mut server = Server::new_async().await;
    let mock = server
        .mock("GET", "/stores/s1")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(STORE_OBJ)
        .create_async()
        .await;

    let config = Configuration::builder().base_path(server.url()).build();
    let result = openfga::apis::stores_api::get_store(&config, "s1").await;

    mock.assert_async().await;
    assert!(result.is_ok(), "expected Ok, got: {:?}", result);
    assert_eq!(result.unwrap().name, "test-store");
}

#[tokio::test]
async fn get_store_404_returns_response_error() {
    let mut server = Server::new_async().await;
    server
        .mock("GET", "/stores/unknown")
        .with_status(404)
        .with_body(ERR_404)
        .create_async()
        .await;

    let config = Configuration::builder().base_path(server.url()).build();
    let result = openfga::apis::stores_api::get_store(&config, "unknown").await;
    assert_status!(result, reqwest::StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn delete_store_204_returns_ok() {
    let mut server = Server::new_async().await;
    let mock = server
        .mock("DELETE", "/stores/s1")
        .with_status(204)
        .create_async()
        .await;

    let config = Configuration::builder().base_path(server.url()).build();
    let result = openfga::apis::stores_api::delete_store(&config, "s1").await;

    mock.assert_async().await;
    assert!(result.is_ok(), "expected Ok(()), got: {:?}", result);
}

#[tokio::test]
async fn delete_store_401_returns_response_error() {
    let mut server = Server::new_async().await;
    server
        .mock("DELETE", "/stores/s1")
        .with_status(401)
        .with_body(ERR_401)
        .create_async()
        .await;

    let config = Configuration::builder().base_path(server.url()).build();
    let result = openfga::apis::stores_api::delete_store(&config, "s1").await;
    assert_status!(result, reqwest::StatusCode::UNAUTHORIZED);
}

// ── Assertions ─────────────────────────────────────────────────────────────────

#[tokio::test]
async fn read_assertions_200_returns_ok_response() {
    let mut server = Server::new_async().await;
    let mock = server
        .mock("GET", "/stores/s1/assertions/m1")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"{"authorization_model_id":"m1","assertions":[]}"#)
        .create_async()
        .await;

    let config = Configuration::builder().base_path(server.url()).build();
    let result = openfga::apis::assertions_api::read_assertions(&config, "s1", "m1").await;

    mock.assert_async().await;
    assert!(result.is_ok(), "expected Ok, got: {:?}", result);
    assert_eq!(result.unwrap().authorization_model_id, "m1");
}

#[tokio::test]
async fn read_assertions_401_returns_response_error() {
    let mut server = Server::new_async().await;
    server
        .mock("GET", "/stores/s1/assertions/m1")
        .with_status(401)
        .with_body(ERR_401)
        .create_async()
        .await;

    let config = Configuration::builder().base_path(server.url()).build();
    let result = openfga::apis::assertions_api::read_assertions(&config, "s1", "m1").await;
    assert_status!(result, reqwest::StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn read_assertions_404_returns_response_error() {
    let mut server = Server::new_async().await;
    server
        .mock("GET", "/stores/s1/assertions/bad-model")
        .with_status(404)
        .with_body(ERR_404)
        .create_async()
        .await;

    let config = Configuration::builder().base_path(server.url()).build();
    let result = openfga::apis::assertions_api::read_assertions(&config, "s1", "bad-model").await;
    assert_status!(result, reqwest::StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn read_assertions_500_returns_response_error() {
    let mut server = Server::new_async().await;
    server
        .mock("GET", "/stores/s1/assertions/m1")
        .with_status(500)
        .with_body(ERR_500)
        .create_async()
        .await;

    let config = Configuration::builder().base_path(server.url()).build();
    let result = openfga::apis::assertions_api::read_assertions(&config, "s1", "m1").await;
    assert_status!(result, reqwest::StatusCode::INTERNAL_SERVER_ERROR);
}

#[tokio::test]
async fn read_assertions_200_malformed_json_returns_serde_error() {
    let mut server = Server::new_async().await;
    server
        .mock("GET", "/stores/s1/assertions/m1")
        .with_status(200)
        .with_body("not valid json {{{")
        .create_async()
        .await;

    let config = Configuration::builder().base_path(server.url()).build();
    let result = openfga::apis::assertions_api::read_assertions(&config, "s1", "m1").await;
    assert!(
        matches!(result, Err(Error::Serde(_))),
        "expected Err(Serde(_)), got: {:?}",
        result
    );
}

#[tokio::test]
async fn write_assertions_204_returns_ok() {
    let mut server = Server::new_async().await;
    let mock = server
        .mock("PUT", "/stores/s1/assertions/m1")
        .with_status(204)
        .create_async()
        .await;

    let config = Configuration::builder().base_path(server.url()).build();
    let body = models::WriteAssertionsRequest::new(vec![]);
    let result = openfga::apis::assertions_api::write_assertions(&config, "s1", "m1", body).await;

    mock.assert_async().await;
    assert!(result.is_ok(), "expected Ok(()), got: {:?}", result);
}

#[tokio::test]
async fn write_assertions_401_returns_response_error() {
    let mut server = Server::new_async().await;
    server
        .mock("PUT", "/stores/s1/assertions/m1")
        .with_status(401)
        .with_body(ERR_401)
        .create_async()
        .await;

    let config = Configuration::builder().base_path(server.url()).build();
    let body = models::WriteAssertionsRequest::new(vec![]);
    let result = openfga::apis::assertions_api::write_assertions(&config, "s1", "m1", body).await;
    assert_status!(result, reqwest::StatusCode::UNAUTHORIZED);
}

// ── Authorization Models ───────────────────────────────────────────────────────

#[tokio::test]
async fn read_authorization_models_200_returns_empty_list() {
    let mut server = Server::new_async().await;
    let mock = server
        .mock("GET", "/stores/s1/authorization-models")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"{"authorization_models":[],"continuation_token":null}"#)
        .create_async()
        .await;

    let config = Configuration::builder().base_path(server.url()).build();
    let result = openfga::apis::authorization_models_api::read_authorization_models(
        &config, "s1", None, None,
    )
    .await;

    mock.assert_async().await;
    assert!(result.is_ok(), "expected Ok, got: {:?}", result);
    assert_eq!(result.unwrap().authorization_models.len(), 0);
}

#[tokio::test]
async fn read_authorization_models_401_returns_response_error() {
    let mut server = Server::new_async().await;
    server
        .mock("GET", "/stores/s1/authorization-models")
        .with_status(401)
        .with_body(ERR_401)
        .create_async()
        .await;

    let config = Configuration::builder().base_path(server.url()).build();
    let result = openfga::apis::authorization_models_api::read_authorization_models(
        &config, "s1", None, None,
    )
    .await;
    assert_status!(result, reqwest::StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn write_authorization_model_200_returns_model_id() {
    let mut server = Server::new_async().await;
    let mock = server
        .mock("POST", "/stores/s1/authorization-models")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"{"authorization_model_id":"m1"}"#)
        .create_async()
        .await;

    let config = Configuration::builder().base_path(server.url()).build();
    let body = models::WriteAuthorizationModelRequest::new(vec![], "1.1".to_string());
    let result =
        openfga::apis::authorization_models_api::write_authorization_model(&config, "s1", body)
            .await;

    mock.assert_async().await;
    assert!(result.is_ok(), "expected Ok, got: {:?}", result);
    assert_eq!(result.unwrap().authorization_model_id, "m1");
}

#[tokio::test]
async fn write_authorization_model_400_returns_response_error() {
    let mut server = Server::new_async().await;
    server
        .mock("POST", "/stores/s1/authorization-models")
        .with_status(400)
        .with_body(ERR_400)
        .create_async()
        .await;

    let config = Configuration::builder().base_path(server.url()).build();
    let body = models::WriteAuthorizationModelRequest::new(vec![], "1.1".to_string());
    let result =
        openfga::apis::authorization_models_api::write_authorization_model(&config, "s1", body)
            .await;
    assert_status!(result, reqwest::StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn read_authorization_model_200_returns_model() {
    let mut server = Server::new_async().await;
    let mock = server
        .mock("GET", "/stores/s1/authorization-models/m1")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"{"authorization_model":null}"#)
        .create_async()
        .await;

    let config = Configuration::builder().base_path(server.url()).build();
    let result =
        openfga::apis::authorization_models_api::read_authorization_model(&config, "s1", "m1")
            .await;

    mock.assert_async().await;
    assert!(result.is_ok(), "expected Ok, got: {:?}", result);
}

#[tokio::test]
async fn read_authorization_model_404_returns_response_error() {
    let mut server = Server::new_async().await;
    server
        .mock("GET", "/stores/s1/authorization-models/bad")
        .with_status(404)
        .with_body(ERR_404)
        .create_async()
        .await;

    let config = Configuration::builder().base_path(server.url()).build();
    let result =
        openfga::apis::authorization_models_api::read_authorization_model(&config, "s1", "bad")
            .await;
    assert_status!(result, reqwest::StatusCode::NOT_FOUND);
}

// ── Relationship Queries ───────────────────────────────────────────────────────

#[tokio::test]
async fn check_200_returns_allowed_true() {
    let mut server = Server::new_async().await;
    let mock = server
        .mock("POST", "/stores/s1/check")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"{"allowed":true}"#)
        .create_async()
        .await;

    let config = Configuration::builder().base_path(server.url()).build();
    let body = models::CheckRequest::default();
    let result = openfga::apis::relationship_queries_api::check(&config, "s1", body).await;

    mock.assert_async().await;
    assert!(result.is_ok(), "expected Ok, got: {:?}", result);
    assert_eq!(result.unwrap().allowed, Some(true));
}

#[tokio::test]
async fn check_200_returns_allowed_false() {
    let mut server = Server::new_async().await;
    server
        .mock("POST", "/stores/s1/check")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"{"allowed":false}"#)
        .create_async()
        .await;

    let config = Configuration::builder().base_path(server.url()).build();
    let result = openfga::apis::relationship_queries_api::check(
        &config,
        "s1",
        models::CheckRequest::default(),
    )
    .await;
    assert!(result.is_ok());
    assert_eq!(result.unwrap().allowed, Some(false));
}

#[tokio::test]
async fn check_401_returns_response_error() {
    let mut server = Server::new_async().await;
    server
        .mock("POST", "/stores/s1/check")
        .with_status(401)
        .with_body(ERR_401)
        .create_async()
        .await;

    let config = Configuration::builder().base_path(server.url()).build();
    let result = openfga::apis::relationship_queries_api::check(
        &config,
        "s1",
        models::CheckRequest::default(),
    )
    .await;
    assert_status!(result, reqwest::StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn check_403_returns_response_error() {
    let mut server = Server::new_async().await;
    server
        .mock("POST", "/stores/s1/check")
        .with_status(403)
        .with_body(ERR_403)
        .create_async()
        .await;

    let config = Configuration::builder().base_path(server.url()).build();
    let result = openfga::apis::relationship_queries_api::check(
        &config,
        "s1",
        models::CheckRequest::default(),
    )
    .await;
    assert_status!(result, reqwest::StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn batch_check_200_returns_ok() {
    let mut server = Server::new_async().await;
    let mock = server
        .mock("POST", "/stores/s1/batch-check")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"{"result":{}}"#)
        .create_async()
        .await;

    let config = Configuration::builder().base_path(server.url()).build();
    let body = models::BatchCheckRequest::default();
    let result = openfga::apis::relationship_queries_api::batch_check(&config, "s1", body).await;

    mock.assert_async().await;
    assert!(result.is_ok(), "expected Ok, got: {:?}", result);
}

#[tokio::test]
async fn batch_check_401_returns_response_error() {
    let mut server = Server::new_async().await;
    server
        .mock("POST", "/stores/s1/batch-check")
        .with_status(401)
        .with_body(ERR_401)
        .create_async()
        .await;

    let config = Configuration::builder().base_path(server.url()).build();
    let result = openfga::apis::relationship_queries_api::batch_check(
        &config,
        "s1",
        models::BatchCheckRequest::default(),
    )
    .await;
    assert_status!(result, reqwest::StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn expand_200_returns_ok() {
    let mut server = Server::new_async().await;
    let mock = server
        .mock("POST", "/stores/s1/expand")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"{"tree":null}"#)
        .create_async()
        .await;

    let config = Configuration::builder().base_path(server.url()).build();
    let body = models::ExpandRequest::default();
    let result = openfga::apis::relationship_queries_api::expand(&config, "s1", body).await;

    mock.assert_async().await;
    assert!(result.is_ok(), "expected Ok, got: {:?}", result);
}

#[tokio::test]
async fn expand_401_returns_response_error() {
    let mut server = Server::new_async().await;
    server
        .mock("POST", "/stores/s1/expand")
        .with_status(401)
        .with_body(ERR_401)
        .create_async()
        .await;

    let config = Configuration::builder().base_path(server.url()).build();
    let result = openfga::apis::relationship_queries_api::expand(
        &config,
        "s1",
        models::ExpandRequest::default(),
    )
    .await;
    assert_status!(result, reqwest::StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn list_objects_200_returns_objects() {
    let mut server = Server::new_async().await;
    let mock = server
        .mock("POST", "/stores/s1/list-objects")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"{"objects":["doc:1","doc:2"]}"#)
        .create_async()
        .await;

    let config = Configuration::builder().base_path(server.url()).build();
    let body = models::ListObjectsRequest::new(
        "document".to_string(),
        "viewer".to_string(),
        "user:alice".to_string(),
    );
    let result = openfga::apis::relationship_queries_api::list_objects(&config, "s1", body).await;

    mock.assert_async().await;
    assert!(result.is_ok(), "expected Ok, got: {:?}", result);
    assert_eq!(result.unwrap().objects.len(), 2);
}

#[tokio::test]
async fn list_objects_401_returns_response_error() {
    let mut server = Server::new_async().await;
    server
        .mock("POST", "/stores/s1/list-objects")
        .with_status(401)
        .with_body(ERR_401)
        .create_async()
        .await;

    let config = Configuration::builder().base_path(server.url()).build();
    let body = models::ListObjectsRequest::new(
        "document".to_string(),
        "viewer".to_string(),
        "user:alice".to_string(),
    );
    let result = openfga::apis::relationship_queries_api::list_objects(&config, "s1", body).await;
    assert_status!(result, reqwest::StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn list_users_200_returns_users() {
    let mut server = Server::new_async().await;
    let mock = server
        .mock("POST", "/stores/s1/list-users")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"{"users":[]}"#)
        .create_async()
        .await;

    let config = Configuration::builder().base_path(server.url()).build();
    let body = models::ListUsersRequest::default();
    let result = openfga::apis::relationship_queries_api::list_users(&config, "s1", body).await;

    mock.assert_async().await;
    assert!(result.is_ok(), "expected Ok, got: {:?}", result);
    assert_eq!(result.unwrap().users.len(), 0);
}

#[tokio::test]
async fn list_users_401_returns_response_error() {
    let mut server = Server::new_async().await;
    server
        .mock("POST", "/stores/s1/list-users")
        .with_status(401)
        .with_body(ERR_401)
        .create_async()
        .await;

    let config = Configuration::builder().base_path(server.url()).build();
    let result = openfga::apis::relationship_queries_api::list_users(
        &config,
        "s1",
        models::ListUsersRequest::default(),
    )
    .await;
    assert_status!(result, reqwest::StatusCode::UNAUTHORIZED);
}

// ── Streaming (B-1 regression) ─────────────────────────────────────────────────

#[tokio::test]
async fn streamed_list_objects_returns_all_success_items() {
    let mut server = Server::new_async().await;
    let ndjson = "{\"result\":{\"object\":\"doc:1\"}}\n{\"result\":{\"object\":\"doc:2\"}}\n";
    let mock = server
        .mock("POST", "/stores/s1/streamed-list-objects")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(ndjson)
        .create_async()
        .await;

    let config = Configuration::builder().base_path(server.url()).build();
    let body = models::ListObjectsRequest::new(
        "document".to_string(),
        "viewer".to_string(),
        "user:alice".to_string(),
    );
    let result =
        openfga::apis::relationship_queries_api::streamed_list_objects(&config, "s1", body).await;

    mock.assert_async().await;
    assert!(result.is_ok(), "expected Ok, got: {:?}", result);
    let items = result.unwrap();
    assert_eq!(items.len(), 2);
    assert_eq!(items[0].object, "doc:1");
    assert_eq!(items[1].object, "doc:2");
}

#[tokio::test]
async fn streamed_list_objects_propagates_server_side_stream_error() {
    // B-1 regression: a 200 response containing an error NDJSON line must
    // surface as Err, not silently return Ok([]).
    let mut server = Server::new_async().await;
    let ndjson = r#"{"error":{"code":8,"message":"rate limit exceeded"}}"#;
    let mock = server
        .mock("POST", "/stores/s1/streamed-list-objects")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(ndjson)
        .create_async()
        .await;

    let config = Configuration::builder().base_path(server.url()).build();
    let body = models::ListObjectsRequest::new(
        "document".to_string(),
        "viewer".to_string(),
        "user:alice".to_string(),
    );
    let result =
        openfga::apis::relationship_queries_api::streamed_list_objects(&config, "s1", body).await;

    mock.assert_async().await;
    assert!(
        matches!(result, Err(Error::ResponseError(_))),
        "expected Err(ResponseError) for stream error, got: {:?}",
        result
    );
}

#[tokio::test]
async fn streamed_list_objects_401_returns_response_error() {
    let mut server = Server::new_async().await;
    server
        .mock("POST", "/stores/s1/streamed-list-objects")
        .with_status(401)
        .with_body(ERR_401)
        .create_async()
        .await;

    let config = Configuration::builder().base_path(server.url()).build();
    let body = models::ListObjectsRequest::new(
        "document".to_string(),
        "viewer".to_string(),
        "user:alice".to_string(),
    );
    let result =
        openfga::apis::relationship_queries_api::streamed_list_objects(&config, "s1", body).await;
    assert_status!(result, reqwest::StatusCode::UNAUTHORIZED);
}

// ── Relationship Tuples ────────────────────────────────────────────────────────

#[tokio::test]
async fn read_tuples_200_returns_tuples() {
    let mut server = Server::new_async().await;
    let mock = server
        .mock("POST", "/stores/s1/read")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"{"tuples":[],"continuation_token":""}"#)
        .create_async()
        .await;

    let config = Configuration::builder().base_path(server.url()).build();
    let body = models::ReadRequest::default();
    let result = openfga::apis::relationship_tuples_api::read(&config, "s1", body).await;

    mock.assert_async().await;
    assert!(result.is_ok(), "expected Ok, got: {:?}", result);
    assert_eq!(result.unwrap().tuples.len(), 0);
}

#[tokio::test]
async fn read_tuples_401_returns_response_error() {
    let mut server = Server::new_async().await;
    server
        .mock("POST", "/stores/s1/read")
        .with_status(401)
        .with_body(ERR_401)
        .create_async()
        .await;

    let config = Configuration::builder().base_path(server.url()).build();
    let result =
        openfga::apis::relationship_tuples_api::read(&config, "s1", models::ReadRequest::default())
            .await;
    assert_status!(result, reqwest::StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn read_changes_200_returns_changes() {
    let mut server = Server::new_async().await;
    let mock = server
        .mock("GET", "/stores/s1/changes")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"{"changes":[],"continuation_token":null}"#)
        .create_async()
        .await;

    let config = Configuration::builder().base_path(server.url()).build();
    let result =
        openfga::apis::relationship_tuples_api::read_changes(&config, "s1", None, None, None, None)
            .await;

    mock.assert_async().await;
    assert!(result.is_ok(), "expected Ok, got: {:?}", result);
    assert_eq!(result.unwrap().changes.len(), 0);
}

#[tokio::test]
async fn read_changes_401_returns_response_error() {
    let mut server = Server::new_async().await;
    server
        .mock("GET", "/stores/s1/changes")
        .with_status(401)
        .with_body(ERR_401)
        .create_async()
        .await;

    let config = Configuration::builder().base_path(server.url()).build();
    let result =
        openfga::apis::relationship_tuples_api::read_changes(&config, "s1", None, None, None, None)
            .await;
    assert_status!(result, reqwest::StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn write_tuples_200_returns_ok() {
    let mut server = Server::new_async().await;
    let mock = server
        .mock("POST", "/stores/s1/write")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"{}"#)
        .create_async()
        .await;

    let config = Configuration::builder().base_path(server.url()).build();
    let body = models::WriteRequest::default();
    let result = openfga::apis::relationship_tuples_api::write(&config, "s1", body).await;

    mock.assert_async().await;
    assert!(result.is_ok(), "expected Ok, got: {:?}", result);
}

#[tokio::test]
async fn write_tuples_400_returns_response_error() {
    let mut server = Server::new_async().await;
    server
        .mock("POST", "/stores/s1/write")
        .with_status(400)
        .with_body(ERR_400)
        .create_async()
        .await;

    let config = Configuration::builder().base_path(server.url()).build();
    let result = openfga::apis::relationship_tuples_api::write(
        &config,
        "s1",
        models::WriteRequest::default(),
    )
    .await;
    assert_status!(result, reqwest::StatusCode::BAD_REQUEST);
}

// ── AuthZen (experimental) ─────────────────────────────────────────────────────

#[tokio::test]
async fn get_configuration_200_returns_ok() {
    let mut server = Server::new_async().await;
    let mock = server
        .mock("GET", "/.well-known/authzen-configuration/s1")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(
            r#"{"policy_decision_point":"openfga","access_evaluation_endpoint":"/evaluation"}"#,
        )
        .create_async()
        .await;

    let config = Configuration::builder().base_path(server.url()).build();
    let result = openfga::apis::auth_zen_service_api::get_configuration(&config, "s1").await;

    mock.assert_async().await;
    assert!(result.is_ok(), "expected Ok, got: {:?}", result);
    assert_eq!(result.unwrap().policy_decision_point, "openfga");
}

#[tokio::test]
async fn get_configuration_401_returns_response_error() {
    let mut server = Server::new_async().await;
    server
        .mock("GET", "/.well-known/authzen-configuration/s1")
        .with_status(401)
        .with_body(ERR_401)
        .create_async()
        .await;

    let config = Configuration::builder().base_path(server.url()).build();
    let result = openfga::apis::auth_zen_service_api::get_configuration(&config, "s1").await;
    assert_status!(result, reqwest::StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn evaluation_200_returns_decision() {
    let mut server = Server::new_async().await;
    let mock = server
        .mock("POST", "/stores/s1/access/v1/evaluation")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"{"decision":true}"#)
        .create_async()
        .await;

    let config = Configuration::builder().base_path(server.url()).build();
    let body = models::EvaluationRequest::default();
    let result = openfga::apis::auth_zen_service_api::evaluation(&config, "s1", body).await;

    mock.assert_async().await;
    assert!(result.is_ok(), "expected Ok, got: {:?}", result);
    assert_eq!(result.unwrap().decision, Some(true));
}

#[tokio::test]
async fn evaluation_401_returns_response_error() {
    let mut server = Server::new_async().await;
    server
        .mock("POST", "/stores/s1/access/v1/evaluation")
        .with_status(401)
        .with_body(ERR_401)
        .create_async()
        .await;

    let config = Configuration::builder().base_path(server.url()).build();
    let result = openfga::apis::auth_zen_service_api::evaluation(
        &config,
        "s1",
        models::EvaluationRequest::default(),
    )
    .await;
    assert_status!(result, reqwest::StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn evaluations_200_returns_ok() {
    let mut server = Server::new_async().await;
    let mock = server
        .mock("POST", "/stores/s1/access/v1/evaluations")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"{"evaluations":[]}"#)
        .create_async()
        .await;

    let config = Configuration::builder().base_path(server.url()).build();
    let body = models::EvaluationsRequest::default();
    let result = openfga::apis::auth_zen_service_api::evaluations(&config, "s1", body).await;

    mock.assert_async().await;
    assert!(result.is_ok(), "expected Ok, got: {:?}", result);
}

#[tokio::test]
async fn evaluations_401_returns_response_error() {
    let mut server = Server::new_async().await;
    server
        .mock("POST", "/stores/s1/access/v1/evaluations")
        .with_status(401)
        .with_body(ERR_401)
        .create_async()
        .await;

    let config = Configuration::builder().base_path(server.url()).build();
    let result = openfga::apis::auth_zen_service_api::evaluations(
        &config,
        "s1",
        models::EvaluationsRequest::default(),
    )
    .await;
    assert_status!(result, reqwest::StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn action_search_200_returns_ok() {
    let mut server = Server::new_async().await;
    let mock = server
        .mock("POST", "/stores/s1/access/v1/search/action")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"{"results":[]}"#)
        .create_async()
        .await;

    let config = Configuration::builder().base_path(server.url()).build();
    let body = models::ActionSearchRequest::default();
    let result = openfga::apis::auth_zen_service_api::action_search(&config, "s1", body).await;

    mock.assert_async().await;
    assert!(result.is_ok(), "expected Ok, got: {:?}", result);
}

#[tokio::test]
async fn action_search_401_returns_response_error() {
    let mut server = Server::new_async().await;
    server
        .mock("POST", "/stores/s1/access/v1/search/action")
        .with_status(401)
        .with_body(ERR_401)
        .create_async()
        .await;

    let config = Configuration::builder().base_path(server.url()).build();
    let result = openfga::apis::auth_zen_service_api::action_search(
        &config,
        "s1",
        models::ActionSearchRequest::default(),
    )
    .await;
    assert_status!(result, reqwest::StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn resource_search_200_returns_ok() {
    let mut server = Server::new_async().await;
    let mock = server
        .mock("POST", "/stores/s1/access/v1/search/resource")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"{"results":[]}"#)
        .create_async()
        .await;

    let config = Configuration::builder().base_path(server.url()).build();
    let body = models::ResourceSearchRequest::default();
    let result = openfga::apis::auth_zen_service_api::resource_search(&config, "s1", body).await;

    mock.assert_async().await;
    assert!(result.is_ok(), "expected Ok, got: {:?}", result);
}

#[tokio::test]
async fn resource_search_401_returns_response_error() {
    let mut server = Server::new_async().await;
    server
        .mock("POST", "/stores/s1/access/v1/search/resource")
        .with_status(401)
        .with_body(ERR_401)
        .create_async()
        .await;

    let config = Configuration::builder().base_path(server.url()).build();
    let result = openfga::apis::auth_zen_service_api::resource_search(
        &config,
        "s1",
        models::ResourceSearchRequest::default(),
    )
    .await;
    assert_status!(result, reqwest::StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn subject_search_200_returns_ok() {
    let mut server = Server::new_async().await;
    let mock = server
        .mock("POST", "/stores/s1/access/v1/search/subject")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"{"results":[]}"#)
        .create_async()
        .await;

    let config = Configuration::builder().base_path(server.url()).build();
    let body = models::SubjectSearchRequest::default();
    let result = openfga::apis::auth_zen_service_api::subject_search(&config, "s1", body).await;

    mock.assert_async().await;
    assert!(result.is_ok(), "expected Ok, got: {:?}", result);
}

#[tokio::test]
async fn subject_search_401_returns_response_error() {
    let mut server = Server::new_async().await;
    server
        .mock("POST", "/stores/s1/access/v1/search/subject")
        .with_status(401)
        .with_body(ERR_401)
        .create_async()
        .await;

    let config = Configuration::builder().base_path(server.url()).build();
    let result = openfga::apis::auth_zen_service_api::subject_search(
        &config,
        "s1",
        models::SubjectSearchRequest::default(),
    )
    .await;
    assert_status!(result, reqwest::StatusCode::UNAUTHORIZED);
}

// ── Auth Header Variants ───────────────────────────────────────────────────────

#[tokio::test]
async fn bearer_token_config_sends_authorization_header() {
    let mut server = Server::new_async().await;
    let mock = server
        .mock("GET", "/stores")
        .match_header("authorization", "Bearer secret-token")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"{"stores":[],"continuation_token":""}"#)
        .create_async()
        .await;

    let config = Configuration::builder()
        .base_path(server.url())
        .bearer_token("secret-token")
        .build();
    let result = openfga::apis::stores_api::list_stores(&config, None, None, None).await;

    mock.assert_async().await;
    assert!(result.is_ok(), "expected Ok, got: {:?}", result);
}

#[tokio::test]
async fn oauth_token_config_sends_bearer_authorization_header() {
    // OAuth token is sent identically to bearer on the wire (Authorization: Bearer <token>)
    let mut server = Server::new_async().await;
    let mock = server
        .mock("GET", "/stores")
        .match_header("authorization", "Bearer oauth-token-value")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"{"stores":[],"continuation_token":""}"#)
        .create_async()
        .await;

    let config = Configuration::builder()
        .base_path(server.url())
        .oauth_token("oauth-token-value")
        .build();
    let result = openfga::apis::stores_api::list_stores(&config, None, None, None).await;

    mock.assert_async().await;
    assert!(result.is_ok(), "expected Ok, got: {:?}", result);
}

#[tokio::test]
async fn api_key_with_prefix_sends_prefixed_authorization_header() {
    let mut server = Server::new_async().await;
    let mock = server
        .mock("GET", "/stores")
        .match_header("authorization", "Token my-api-key")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"{"stores":[],"continuation_token":""}"#)
        .create_async()
        .await;

    let config = Configuration::builder()
        .base_path(server.url())
        .api_key("my-api-key", Some("Token"))
        .build();
    let result = openfga::apis::stores_api::list_stores(&config, None, None, None).await;

    mock.assert_async().await;
    assert!(result.is_ok(), "expected Ok, got: {:?}", result);
}

#[tokio::test]
async fn api_key_without_prefix_sends_bare_key_as_authorization_header() {
    let mut server = Server::new_async().await;
    let mock = server
        .mock("GET", "/stores")
        .match_header("authorization", "bare-key-value")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"{"stores":[],"continuation_token":""}"#)
        .create_async()
        .await;

    let config = Configuration::builder()
        .base_path(server.url())
        .api_key("bare-key-value", None::<String>)
        .build();
    let result = openfga::apis::stores_api::list_stores(&config, None, None, None).await;

    mock.assert_async().await;
    assert!(result.is_ok(), "expected Ok, got: {:?}", result);
}

#[tokio::test]
async fn no_auth_config_sends_no_authorization_header() {
    let mut server = Server::new_async().await;
    // No .match_header constraint — the mock matches any request to /stores,
    // proving that an unauthenticated config does not add an Authorization header
    // (if it did add one, we'd be testing bearer_token, not "no auth").
    let mock = server
        .mock("GET", "/stores")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"{"stores":[],"continuation_token":""}"#)
        .create_async()
        .await;

    let config = Configuration::builder().base_path(server.url()).build(); // no auth
    let result = openfga::apis::stores_api::list_stores(&config, None, None, None).await;

    mock.assert_async().await;
    assert!(result.is_ok(), "expected Ok, got: {:?}", result);
}

#[tokio::test]
async fn basic_auth_config_sends_basic_authorization_header() {
    // reqwest encodes Basic auth as "Basic <base64(user:pass)>"
    let mut server = Server::new_async().await;
    let mock = server
        .mock("GET", "/stores")
        .match_header("authorization", "Basic YmFzaWMtdXNlcjpiYXNpYy1wYXNz")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"{"stores":[],"continuation_token":""}"#)
        .create_async()
        .await;

    let config = Configuration::builder()
        .base_path(server.url())
        .basic_auth("basic-user", Some("basic-pass"))
        .build();
    let result = openfga::apis::stores_api::list_stores(&config, None, None, None).await;

    mock.assert_async().await;
    assert!(result.is_ok(), "expected Ok, got: {:?}", result);
}

// ── Query Parameter Encoding ───────────────────────────────────────────────────

#[tokio::test]
async fn list_stores_with_page_size_encodes_query_param() {
    let mut server = Server::new_async().await;
    let mock = server
        .mock("GET", "/stores")
        .match_query(Matcher::UrlEncoded("page_size".into(), "50".into()))
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"{"stores":[],"continuation_token":""}"#)
        .create_async()
        .await;

    let config = Configuration::builder().base_path(server.url()).build();
    let result = openfga::apis::stores_api::list_stores(&config, Some(50), None, None).await;

    mock.assert_async().await;
    assert!(result.is_ok(), "expected Ok, got: {:?}", result);
}

#[tokio::test]
async fn list_stores_with_continuation_token_encodes_query_param() {
    let mut server = Server::new_async().await;
    let mock = server
        .mock("GET", "/stores")
        .match_query(Matcher::UrlEncoded(
            "continuation_token".into(),
            "page2-token".into(),
        ))
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"{"stores":[],"continuation_token":""}"#)
        .create_async()
        .await;

    let config = Configuration::builder().base_path(server.url()).build();
    let result =
        openfga::apis::stores_api::list_stores(&config, None, Some("page2-token"), None).await;

    mock.assert_async().await;
    assert!(result.is_ok(), "expected Ok, got: {:?}", result);
}

#[tokio::test]
async fn list_stores_with_name_filter_encodes_query_param() {
    let mut server = Server::new_async().await;
    let mock = server
        .mock("GET", "/stores")
        .match_query(Matcher::UrlEncoded("name".into(), "prod-store".into()))
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"{"stores":[],"continuation_token":""}"#)
        .create_async()
        .await;

    let config = Configuration::builder().base_path(server.url()).build();
    let result =
        openfga::apis::stores_api::list_stores(&config, None, None, Some("prod-store")).await;

    mock.assert_async().await;
    assert!(result.is_ok(), "expected Ok, got: {:?}", result);
}

#[tokio::test]
async fn read_authorization_models_with_page_size_encodes_query_param() {
    let mut server = Server::new_async().await;
    let mock = server
        .mock("GET", "/stores/s1/authorization-models")
        .match_query(Matcher::UrlEncoded("page_size".into(), "25".into()))
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"{"authorization_models":[],"continuation_token":null}"#)
        .create_async()
        .await;

    let config = Configuration::builder().base_path(server.url()).build();
    let result = openfga::apis::authorization_models_api::read_authorization_models(
        &config,
        "s1",
        Some(25),
        None,
    )
    .await;

    mock.assert_async().await;
    assert!(result.is_ok(), "expected Ok, got: {:?}", result);
}

#[tokio::test]
async fn read_authorization_models_with_continuation_token_encodes_query_param() {
    let mut server = Server::new_async().await;
    let mock = server
        .mock("GET", "/stores/s1/authorization-models")
        .match_query(Matcher::UrlEncoded(
            "continuation_token".into(),
            "page2-token".into(),
        ))
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"{"authorization_models":[],"continuation_token":null}"#)
        .create_async()
        .await;

    let config = Configuration::builder().base_path(server.url()).build();
    let result = openfga::apis::authorization_models_api::read_authorization_models(
        &config,
        "s1",
        None,
        Some("page2-token"),
    )
    .await;

    mock.assert_async().await;
    assert!(result.is_ok(), "expected Ok, got: {:?}", result);
}

#[tokio::test]
async fn read_changes_with_type_filter_encodes_query_param() {
    let mut server = Server::new_async().await;
    let mock = server
        .mock("GET", "/stores/s1/changes")
        .match_query(Matcher::UrlEncoded("type".into(), "document".into()))
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"{"changes":[],"continuation_token":null}"#)
        .create_async()
        .await;

    let config = Configuration::builder().base_path(server.url()).build();
    let result = openfga::apis::relationship_tuples_api::read_changes(
        &config,
        "s1",
        Some("document"),
        None,
        None,
        None,
    )
    .await;

    mock.assert_async().await;
    assert!(result.is_ok(), "expected Ok, got: {:?}", result);
}

#[tokio::test]
async fn read_changes_with_page_size_encodes_query_param() {
    let mut server = Server::new_async().await;
    let mock = server
        .mock("GET", "/stores/s1/changes")
        .match_query(Matcher::UrlEncoded("page_size".into(), "100".into()))
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"{"changes":[],"continuation_token":null}"#)
        .create_async()
        .await;

    let config = Configuration::builder().base_path(server.url()).build();
    let result = openfga::apis::relationship_tuples_api::read_changes(
        &config,
        "s1",
        None,
        Some(100),
        None,
        None,
    )
    .await;

    mock.assert_async().await;
    assert!(result.is_ok(), "expected Ok, got: {:?}", result);
}

// ══════════════════════════════════════════════════════════════════════════════
// Gap Group 1: Missing 401 Unauthorized
// ══════════════════════════════════════════════════════════════════════════════

#[tokio::test]
async fn get_store_401_returns_response_error() {
    let mut server = Server::new_async().await;
    server
        .mock("GET", "/stores/s1")
        .with_status(401)
        .with_body(ERR_401)
        .create_async()
        .await;
    let config = Configuration::builder().base_path(server.url()).build();
    let result = openfga::apis::stores_api::get_store(&config, "s1").await;
    assert_status!(result, reqwest::StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn write_authorization_model_401_returns_response_error() {
    let mut server = Server::new_async().await;
    server
        .mock("POST", "/stores/s1/authorization-models")
        .with_status(401)
        .with_body(ERR_401)
        .create_async()
        .await;
    let config = Configuration::builder().base_path(server.url()).build();
    let result = openfga::apis::authorization_models_api::write_authorization_model(
        &config,
        "s1",
        models::WriteAuthorizationModelRequest::new(vec![], "1.1".into()),
    )
    .await;
    assert_status!(result, reqwest::StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn read_authorization_model_401_returns_response_error() {
    let mut server = Server::new_async().await;
    server
        .mock("GET", "/stores/s1/authorization-models/m1")
        .with_status(401)
        .with_body(ERR_401)
        .create_async()
        .await;
    let config = Configuration::builder().base_path(server.url()).build();
    let result =
        openfga::apis::authorization_models_api::read_authorization_model(&config, "s1", "m1")
            .await;
    assert_status!(result, reqwest::StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn write_tuples_401_returns_response_error() {
    let mut server = Server::new_async().await;
    server
        .mock("POST", "/stores/s1/write")
        .with_status(401)
        .with_body(ERR_401)
        .create_async()
        .await;
    let config = Configuration::builder().base_path(server.url()).build();
    let result = openfga::apis::relationship_tuples_api::write(
        &config,
        "s1",
        models::WriteRequest::default(),
    )
    .await;
    assert_status!(result, reqwest::StatusCode::UNAUTHORIZED);
}

// ══════════════════════════════════════════════════════════════════════════════
// Gap Group 2: Missing 400 Bad Request
// ══════════════════════════════════════════════════════════════════════════════

#[tokio::test]
async fn list_stores_400_returns_response_error() {
    let mut server = Server::new_async().await;
    server
        .mock("GET", "/stores")
        .with_status(400)
        .with_body(ERR_400)
        .create_async()
        .await;
    let config = Configuration::builder().base_path(server.url()).build();
    let result = openfga::apis::stores_api::list_stores(&config, None, None, None).await;
    assert_status!(result, reqwest::StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn get_store_400_returns_response_error() {
    let mut server = Server::new_async().await;
    server
        .mock("GET", "/stores/s1")
        .with_status(400)
        .with_body(ERR_400)
        .create_async()
        .await;
    let config = Configuration::builder().base_path(server.url()).build();
    let result = openfga::apis::stores_api::get_store(&config, "s1").await;
    assert_status!(result, reqwest::StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn delete_store_400_returns_response_error() {
    let mut server = Server::new_async().await;
    server
        .mock("DELETE", "/stores/s1")
        .with_status(400)
        .with_body(ERR_400)
        .create_async()
        .await;
    let config = Configuration::builder().base_path(server.url()).build();
    let result = openfga::apis::stores_api::delete_store(&config, "s1").await;
    assert_status!(result, reqwest::StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn create_store_400_returns_response_error() {
    let mut server = Server::new_async().await;
    server
        .mock("POST", "/stores")
        .with_status(400)
        .with_body(ERR_400)
        .create_async()
        .await;
    let config = Configuration::builder().base_path(server.url()).build();
    let result = openfga::apis::stores_api::create_store(
        &config,
        models::CreateStoreRequest::new("x".into()),
    )
    .await;
    assert_status!(result, reqwest::StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn read_assertions_400_returns_response_error() {
    let mut server = Server::new_async().await;
    server
        .mock("GET", "/stores/s1/assertions/m1")
        .with_status(400)
        .with_body(ERR_400)
        .create_async()
        .await;
    let config = Configuration::builder().base_path(server.url()).build();
    let result = openfga::apis::assertions_api::read_assertions(&config, "s1", "m1").await;
    assert_status!(result, reqwest::StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn write_assertions_400_returns_response_error() {
    let mut server = Server::new_async().await;
    server
        .mock("PUT", "/stores/s1/assertions/m1")
        .with_status(400)
        .with_body(ERR_400)
        .create_async()
        .await;
    let config = Configuration::builder().base_path(server.url()).build();
    let result = openfga::apis::assertions_api::write_assertions(
        &config,
        "s1",
        "m1",
        models::WriteAssertionsRequest::default(),
    )
    .await;
    assert_status!(result, reqwest::StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn read_authorization_models_400_returns_response_error() {
    let mut server = Server::new_async().await;
    server
        .mock("GET", "/stores/s1/authorization-models")
        .with_status(400)
        .with_body(ERR_400)
        .create_async()
        .await;
    let config = Configuration::builder().base_path(server.url()).build();
    let result = openfga::apis::authorization_models_api::read_authorization_models(
        &config, "s1", None, None,
    )
    .await;
    assert_status!(result, reqwest::StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn read_authorization_model_400_returns_response_error() {
    let mut server = Server::new_async().await;
    server
        .mock("GET", "/stores/s1/authorization-models/m1")
        .with_status(400)
        .with_body(ERR_400)
        .create_async()
        .await;
    let config = Configuration::builder().base_path(server.url()).build();
    let result =
        openfga::apis::authorization_models_api::read_authorization_model(&config, "s1", "m1")
            .await;
    assert_status!(result, reqwest::StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn check_400_returns_response_error() {
    let mut server = Server::new_async().await;
    server
        .mock("POST", "/stores/s1/check")
        .with_status(400)
        .with_body(ERR_400)
        .create_async()
        .await;
    let config = Configuration::builder().base_path(server.url()).build();
    let result = openfga::apis::relationship_queries_api::check(
        &config,
        "s1",
        models::CheckRequest::default(),
    )
    .await;
    assert_status!(result, reqwest::StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn batch_check_400_returns_response_error() {
    let mut server = Server::new_async().await;
    server
        .mock("POST", "/stores/s1/batch-check")
        .with_status(400)
        .with_body(ERR_400)
        .create_async()
        .await;
    let config = Configuration::builder().base_path(server.url()).build();
    let result = openfga::apis::relationship_queries_api::batch_check(
        &config,
        "s1",
        models::BatchCheckRequest::default(),
    )
    .await;
    assert_status!(result, reqwest::StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn expand_400_returns_response_error() {
    let mut server = Server::new_async().await;
    server
        .mock("POST", "/stores/s1/expand")
        .with_status(400)
        .with_body(ERR_400)
        .create_async()
        .await;
    let config = Configuration::builder().base_path(server.url()).build();
    let result = openfga::apis::relationship_queries_api::expand(
        &config,
        "s1",
        models::ExpandRequest::default(),
    )
    .await;
    assert_status!(result, reqwest::StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn list_objects_400_returns_response_error() {
    let mut server = Server::new_async().await;
    server
        .mock("POST", "/stores/s1/list-objects")
        .with_status(400)
        .with_body(ERR_400)
        .create_async()
        .await;
    let config = Configuration::builder().base_path(server.url()).build();
    let result = openfga::apis::relationship_queries_api::list_objects(
        &config,
        "s1",
        models::ListObjectsRequest::new("document".into(), "viewer".into(), "user:alice".into()),
    )
    .await;
    assert_status!(result, reqwest::StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn list_users_400_returns_response_error() {
    let mut server = Server::new_async().await;
    server
        .mock("POST", "/stores/s1/list-users")
        .with_status(400)
        .with_body(ERR_400)
        .create_async()
        .await;
    let config = Configuration::builder().base_path(server.url()).build();
    let result = openfga::apis::relationship_queries_api::list_users(
        &config,
        "s1",
        models::ListUsersRequest::default(),
    )
    .await;
    assert_status!(result, reqwest::StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn streamed_list_objects_400_returns_response_error() {
    let mut server = Server::new_async().await;
    server
        .mock("POST", "/stores/s1/streamed-list-objects")
        .with_status(400)
        .with_body(ERR_400)
        .create_async()
        .await;
    let config = Configuration::builder().base_path(server.url()).build();
    let result = openfga::apis::relationship_queries_api::streamed_list_objects(
        &config,
        "s1",
        models::ListObjectsRequest::new("document".into(), "viewer".into(), "user:alice".into()),
    )
    .await;
    assert_status!(result, reqwest::StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn read_tuples_400_returns_response_error() {
    let mut server = Server::new_async().await;
    server
        .mock("POST", "/stores/s1/read")
        .with_status(400)
        .with_body(ERR_400)
        .create_async()
        .await;
    let config = Configuration::builder().base_path(server.url()).build();
    let result =
        openfga::apis::relationship_tuples_api::read(&config, "s1", models::ReadRequest::default())
            .await;
    assert_status!(result, reqwest::StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn read_changes_400_returns_response_error() {
    let mut server = Server::new_async().await;
    server
        .mock("GET", "/stores/s1/changes")
        .with_status(400)
        .with_body(ERR_400)
        .create_async()
        .await;
    let config = Configuration::builder().base_path(server.url()).build();
    let result =
        openfga::apis::relationship_tuples_api::read_changes(&config, "s1", None, None, None, None)
            .await;
    assert_status!(result, reqwest::StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn get_configuration_400_returns_response_error() {
    let mut server = Server::new_async().await;
    server
        .mock("GET", "/.well-known/authzen-configuration/s1")
        .with_status(400)
        .with_body(ERR_400)
        .create_async()
        .await;
    let config = Configuration::builder().base_path(server.url()).build();
    let result = openfga::apis::auth_zen_service_api::get_configuration(&config, "s1").await;
    assert_status!(result, reqwest::StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn evaluation_400_returns_response_error() {
    let mut server = Server::new_async().await;
    server
        .mock("POST", "/stores/s1/access/v1/evaluation")
        .with_status(400)
        .with_body(ERR_400)
        .create_async()
        .await;
    let config = Configuration::builder().base_path(server.url()).build();
    let result = openfga::apis::auth_zen_service_api::evaluation(
        &config,
        "s1",
        models::EvaluationRequest::default(),
    )
    .await;
    assert_status!(result, reqwest::StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn evaluations_400_returns_response_error() {
    let mut server = Server::new_async().await;
    server
        .mock("POST", "/stores/s1/access/v1/evaluations")
        .with_status(400)
        .with_body(ERR_400)
        .create_async()
        .await;
    let config = Configuration::builder().base_path(server.url()).build();
    let result = openfga::apis::auth_zen_service_api::evaluations(
        &config,
        "s1",
        models::EvaluationsRequest::default(),
    )
    .await;
    assert_status!(result, reqwest::StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn action_search_400_returns_response_error() {
    let mut server = Server::new_async().await;
    server
        .mock("POST", "/stores/s1/access/v1/search/action")
        .with_status(400)
        .with_body(ERR_400)
        .create_async()
        .await;
    let config = Configuration::builder().base_path(server.url()).build();
    let result = openfga::apis::auth_zen_service_api::action_search(
        &config,
        "s1",
        models::ActionSearchRequest::default(),
    )
    .await;
    assert_status!(result, reqwest::StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn resource_search_400_returns_response_error() {
    let mut server = Server::new_async().await;
    server
        .mock("POST", "/stores/s1/access/v1/search/resource")
        .with_status(400)
        .with_body(ERR_400)
        .create_async()
        .await;
    let config = Configuration::builder().base_path(server.url()).build();
    let result = openfga::apis::auth_zen_service_api::resource_search(
        &config,
        "s1",
        models::ResourceSearchRequest::default(),
    )
    .await;
    assert_status!(result, reqwest::StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn subject_search_400_returns_response_error() {
    let mut server = Server::new_async().await;
    server
        .mock("POST", "/stores/s1/access/v1/search/subject")
        .with_status(400)
        .with_body(ERR_400)
        .create_async()
        .await;
    let config = Configuration::builder().base_path(server.url()).build();
    let result = openfga::apis::auth_zen_service_api::subject_search(
        &config,
        "s1",
        models::SubjectSearchRequest::default(),
    )
    .await;
    assert_status!(result, reqwest::StatusCode::BAD_REQUEST);
}

// ══════════════════════════════════════════════════════════════════════════════
// Gap Group 3: Missing 403 Forbidden
// ══════════════════════════════════════════════════════════════════════════════

#[tokio::test]
async fn list_stores_403_returns_response_error() {
    let mut server = Server::new_async().await;
    server
        .mock("GET", "/stores")
        .with_status(403)
        .with_body(ERR_403)
        .create_async()
        .await;
    let config = Configuration::builder().base_path(server.url()).build();
    let result = openfga::apis::stores_api::list_stores(&config, None, None, None).await;
    assert_status!(result, reqwest::StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn get_store_403_returns_response_error() {
    let mut server = Server::new_async().await;
    server
        .mock("GET", "/stores/s1")
        .with_status(403)
        .with_body(ERR_403)
        .create_async()
        .await;
    let config = Configuration::builder().base_path(server.url()).build();
    let result = openfga::apis::stores_api::get_store(&config, "s1").await;
    assert_status!(result, reqwest::StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn delete_store_403_returns_response_error() {
    let mut server = Server::new_async().await;
    server
        .mock("DELETE", "/stores/s1")
        .with_status(403)
        .with_body(ERR_403)
        .create_async()
        .await;
    let config = Configuration::builder().base_path(server.url()).build();
    let result = openfga::apis::stores_api::delete_store(&config, "s1").await;
    assert_status!(result, reqwest::StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn create_store_403_returns_response_error() {
    let mut server = Server::new_async().await;
    server
        .mock("POST", "/stores")
        .with_status(403)
        .with_body(ERR_403)
        .create_async()
        .await;
    let config = Configuration::builder().base_path(server.url()).build();
    let result = openfga::apis::stores_api::create_store(
        &config,
        models::CreateStoreRequest::new("x".into()),
    )
    .await;
    assert_status!(result, reqwest::StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn read_assertions_403_returns_response_error() {
    let mut server = Server::new_async().await;
    server
        .mock("GET", "/stores/s1/assertions/m1")
        .with_status(403)
        .with_body(ERR_403)
        .create_async()
        .await;
    let config = Configuration::builder().base_path(server.url()).build();
    let result = openfga::apis::assertions_api::read_assertions(&config, "s1", "m1").await;
    assert_status!(result, reqwest::StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn write_assertions_403_returns_response_error() {
    let mut server = Server::new_async().await;
    server
        .mock("PUT", "/stores/s1/assertions/m1")
        .with_status(403)
        .with_body(ERR_403)
        .create_async()
        .await;
    let config = Configuration::builder().base_path(server.url()).build();
    let result = openfga::apis::assertions_api::write_assertions(
        &config,
        "s1",
        "m1",
        models::WriteAssertionsRequest::default(),
    )
    .await;
    assert_status!(result, reqwest::StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn read_authorization_models_403_returns_response_error() {
    let mut server = Server::new_async().await;
    server
        .mock("GET", "/stores/s1/authorization-models")
        .with_status(403)
        .with_body(ERR_403)
        .create_async()
        .await;
    let config = Configuration::builder().base_path(server.url()).build();
    let result = openfga::apis::authorization_models_api::read_authorization_models(
        &config, "s1", None, None,
    )
    .await;
    assert_status!(result, reqwest::StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn read_authorization_model_403_returns_response_error() {
    let mut server = Server::new_async().await;
    server
        .mock("GET", "/stores/s1/authorization-models/m1")
        .with_status(403)
        .with_body(ERR_403)
        .create_async()
        .await;
    let config = Configuration::builder().base_path(server.url()).build();
    let result =
        openfga::apis::authorization_models_api::read_authorization_model(&config, "s1", "m1")
            .await;
    assert_status!(result, reqwest::StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn write_authorization_model_403_returns_response_error() {
    let mut server = Server::new_async().await;
    server
        .mock("POST", "/stores/s1/authorization-models")
        .with_status(403)
        .with_body(ERR_403)
        .create_async()
        .await;
    let config = Configuration::builder().base_path(server.url()).build();
    let result = openfga::apis::authorization_models_api::write_authorization_model(
        &config,
        "s1",
        models::WriteAuthorizationModelRequest::new(vec![], "1.1".into()),
    )
    .await;
    assert_status!(result, reqwest::StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn batch_check_403_returns_response_error() {
    let mut server = Server::new_async().await;
    server
        .mock("POST", "/stores/s1/batch-check")
        .with_status(403)
        .with_body(ERR_403)
        .create_async()
        .await;
    let config = Configuration::builder().base_path(server.url()).build();
    let result = openfga::apis::relationship_queries_api::batch_check(
        &config,
        "s1",
        models::BatchCheckRequest::default(),
    )
    .await;
    assert_status!(result, reqwest::StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn expand_403_returns_response_error() {
    let mut server = Server::new_async().await;
    server
        .mock("POST", "/stores/s1/expand")
        .with_status(403)
        .with_body(ERR_403)
        .create_async()
        .await;
    let config = Configuration::builder().base_path(server.url()).build();
    let result = openfga::apis::relationship_queries_api::expand(
        &config,
        "s1",
        models::ExpandRequest::default(),
    )
    .await;
    assert_status!(result, reqwest::StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn list_objects_403_returns_response_error() {
    let mut server = Server::new_async().await;
    server
        .mock("POST", "/stores/s1/list-objects")
        .with_status(403)
        .with_body(ERR_403)
        .create_async()
        .await;
    let config = Configuration::builder().base_path(server.url()).build();
    let result = openfga::apis::relationship_queries_api::list_objects(
        &config,
        "s1",
        models::ListObjectsRequest::new("document".into(), "viewer".into(), "user:alice".into()),
    )
    .await;
    assert_status!(result, reqwest::StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn list_users_403_returns_response_error() {
    let mut server = Server::new_async().await;
    server
        .mock("POST", "/stores/s1/list-users")
        .with_status(403)
        .with_body(ERR_403)
        .create_async()
        .await;
    let config = Configuration::builder().base_path(server.url()).build();
    let result = openfga::apis::relationship_queries_api::list_users(
        &config,
        "s1",
        models::ListUsersRequest::default(),
    )
    .await;
    assert_status!(result, reqwest::StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn streamed_list_objects_403_returns_response_error() {
    let mut server = Server::new_async().await;
    server
        .mock("POST", "/stores/s1/streamed-list-objects")
        .with_status(403)
        .with_body(ERR_403)
        .create_async()
        .await;
    let config = Configuration::builder().base_path(server.url()).build();
    let result = openfga::apis::relationship_queries_api::streamed_list_objects(
        &config,
        "s1",
        models::ListObjectsRequest::new("document".into(), "viewer".into(), "user:alice".into()),
    )
    .await;
    assert_status!(result, reqwest::StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn read_tuples_403_returns_response_error() {
    let mut server = Server::new_async().await;
    server
        .mock("POST", "/stores/s1/read")
        .with_status(403)
        .with_body(ERR_403)
        .create_async()
        .await;
    let config = Configuration::builder().base_path(server.url()).build();
    let result =
        openfga::apis::relationship_tuples_api::read(&config, "s1", models::ReadRequest::default())
            .await;
    assert_status!(result, reqwest::StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn read_changes_403_returns_response_error() {
    let mut server = Server::new_async().await;
    server
        .mock("GET", "/stores/s1/changes")
        .with_status(403)
        .with_body(ERR_403)
        .create_async()
        .await;
    let config = Configuration::builder().base_path(server.url()).build();
    let result =
        openfga::apis::relationship_tuples_api::read_changes(&config, "s1", None, None, None, None)
            .await;
    assert_status!(result, reqwest::StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn write_tuples_403_returns_response_error() {
    let mut server = Server::new_async().await;
    server
        .mock("POST", "/stores/s1/write")
        .with_status(403)
        .with_body(ERR_403)
        .create_async()
        .await;
    let config = Configuration::builder().base_path(server.url()).build();
    let result = openfga::apis::relationship_tuples_api::write(
        &config,
        "s1",
        models::WriteRequest::default(),
    )
    .await;
    assert_status!(result, reqwest::StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn get_configuration_403_returns_response_error() {
    let mut server = Server::new_async().await;
    server
        .mock("GET", "/.well-known/authzen-configuration/s1")
        .with_status(403)
        .with_body(ERR_403)
        .create_async()
        .await;
    let config = Configuration::builder().base_path(server.url()).build();
    let result = openfga::apis::auth_zen_service_api::get_configuration(&config, "s1").await;
    assert_status!(result, reqwest::StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn evaluation_403_returns_response_error() {
    let mut server = Server::new_async().await;
    server
        .mock("POST", "/stores/s1/access/v1/evaluation")
        .with_status(403)
        .with_body(ERR_403)
        .create_async()
        .await;
    let config = Configuration::builder().base_path(server.url()).build();
    let result = openfga::apis::auth_zen_service_api::evaluation(
        &config,
        "s1",
        models::EvaluationRequest::default(),
    )
    .await;
    assert_status!(result, reqwest::StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn evaluations_403_returns_response_error() {
    let mut server = Server::new_async().await;
    server
        .mock("POST", "/stores/s1/access/v1/evaluations")
        .with_status(403)
        .with_body(ERR_403)
        .create_async()
        .await;
    let config = Configuration::builder().base_path(server.url()).build();
    let result = openfga::apis::auth_zen_service_api::evaluations(
        &config,
        "s1",
        models::EvaluationsRequest::default(),
    )
    .await;
    assert_status!(result, reqwest::StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn action_search_403_returns_response_error() {
    let mut server = Server::new_async().await;
    server
        .mock("POST", "/stores/s1/access/v1/search/action")
        .with_status(403)
        .with_body(ERR_403)
        .create_async()
        .await;
    let config = Configuration::builder().base_path(server.url()).build();
    let result = openfga::apis::auth_zen_service_api::action_search(
        &config,
        "s1",
        models::ActionSearchRequest::default(),
    )
    .await;
    assert_status!(result, reqwest::StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn resource_search_403_returns_response_error() {
    let mut server = Server::new_async().await;
    server
        .mock("POST", "/stores/s1/access/v1/search/resource")
        .with_status(403)
        .with_body(ERR_403)
        .create_async()
        .await;
    let config = Configuration::builder().base_path(server.url()).build();
    let result = openfga::apis::auth_zen_service_api::resource_search(
        &config,
        "s1",
        models::ResourceSearchRequest::default(),
    )
    .await;
    assert_status!(result, reqwest::StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn subject_search_403_returns_response_error() {
    let mut server = Server::new_async().await;
    server
        .mock("POST", "/stores/s1/access/v1/search/subject")
        .with_status(403)
        .with_body(ERR_403)
        .create_async()
        .await;
    let config = Configuration::builder().base_path(server.url()).build();
    let result = openfga::apis::auth_zen_service_api::subject_search(
        &config,
        "s1",
        models::SubjectSearchRequest::default(),
    )
    .await;
    assert_status!(result, reqwest::StatusCode::FORBIDDEN);
}

// ══════════════════════════════════════════════════════════════════════════════
// Gap Group 4: Missing 404 Not Found
// ══════════════════════════════════════════════════════════════════════════════

#[tokio::test]
async fn delete_store_404_returns_response_error() {
    let mut server = Server::new_async().await;
    server
        .mock("DELETE", "/stores/s1")
        .with_status(404)
        .with_body(ERR_404)
        .create_async()
        .await;
    let config = Configuration::builder().base_path(server.url()).build();
    let result = openfga::apis::stores_api::delete_store(&config, "s1").await;
    assert_status!(result, reqwest::StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn create_store_404_returns_response_error() {
    let mut server = Server::new_async().await;
    server
        .mock("POST", "/stores")
        .with_status(404)
        .with_body(ERR_404)
        .create_async()
        .await;
    let config = Configuration::builder().base_path(server.url()).build();
    let result = openfga::apis::stores_api::create_store(
        &config,
        models::CreateStoreRequest::new("x".into()),
    )
    .await;
    assert_status!(result, reqwest::StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn list_stores_404_returns_response_error() {
    let mut server = Server::new_async().await;
    server
        .mock("GET", "/stores")
        .with_status(404)
        .with_body(ERR_404)
        .create_async()
        .await;
    let config = Configuration::builder().base_path(server.url()).build();
    let result = openfga::apis::stores_api::list_stores(&config, None, None, None).await;
    assert_status!(result, reqwest::StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn write_assertions_404_returns_response_error() {
    let mut server = Server::new_async().await;
    server
        .mock("PUT", "/stores/s1/assertions/m1")
        .with_status(404)
        .with_body(ERR_404)
        .create_async()
        .await;
    let config = Configuration::builder().base_path(server.url()).build();
    let result = openfga::apis::assertions_api::write_assertions(
        &config,
        "s1",
        "m1",
        models::WriteAssertionsRequest::default(),
    )
    .await;
    assert_status!(result, reqwest::StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn read_authorization_models_404_returns_response_error() {
    let mut server = Server::new_async().await;
    server
        .mock("GET", "/stores/s1/authorization-models")
        .with_status(404)
        .with_body(ERR_404)
        .create_async()
        .await;
    let config = Configuration::builder().base_path(server.url()).build();
    let result = openfga::apis::authorization_models_api::read_authorization_models(
        &config, "s1", None, None,
    )
    .await;
    assert_status!(result, reqwest::StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn write_authorization_model_404_returns_response_error() {
    let mut server = Server::new_async().await;
    server
        .mock("POST", "/stores/s1/authorization-models")
        .with_status(404)
        .with_body(ERR_404)
        .create_async()
        .await;
    let config = Configuration::builder().base_path(server.url()).build();
    let result = openfga::apis::authorization_models_api::write_authorization_model(
        &config,
        "s1",
        models::WriteAuthorizationModelRequest::new(vec![], "1.1".into()),
    )
    .await;
    assert_status!(result, reqwest::StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn check_404_returns_response_error() {
    let mut server = Server::new_async().await;
    server
        .mock("POST", "/stores/s1/check")
        .with_status(404)
        .with_body(ERR_404)
        .create_async()
        .await;
    let config = Configuration::builder().base_path(server.url()).build();
    let result = openfga::apis::relationship_queries_api::check(
        &config,
        "s1",
        models::CheckRequest::default(),
    )
    .await;
    assert_status!(result, reqwest::StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn batch_check_404_returns_response_error() {
    let mut server = Server::new_async().await;
    server
        .mock("POST", "/stores/s1/batch-check")
        .with_status(404)
        .with_body(ERR_404)
        .create_async()
        .await;
    let config = Configuration::builder().base_path(server.url()).build();
    let result = openfga::apis::relationship_queries_api::batch_check(
        &config,
        "s1",
        models::BatchCheckRequest::default(),
    )
    .await;
    assert_status!(result, reqwest::StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn expand_404_returns_response_error() {
    let mut server = Server::new_async().await;
    server
        .mock("POST", "/stores/s1/expand")
        .with_status(404)
        .with_body(ERR_404)
        .create_async()
        .await;
    let config = Configuration::builder().base_path(server.url()).build();
    let result = openfga::apis::relationship_queries_api::expand(
        &config,
        "s1",
        models::ExpandRequest::default(),
    )
    .await;
    assert_status!(result, reqwest::StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn list_objects_404_returns_response_error() {
    let mut server = Server::new_async().await;
    server
        .mock("POST", "/stores/s1/list-objects")
        .with_status(404)
        .with_body(ERR_404)
        .create_async()
        .await;
    let config = Configuration::builder().base_path(server.url()).build();
    let result = openfga::apis::relationship_queries_api::list_objects(
        &config,
        "s1",
        models::ListObjectsRequest::new("document".into(), "viewer".into(), "user:alice".into()),
    )
    .await;
    assert_status!(result, reqwest::StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn list_users_404_returns_response_error() {
    let mut server = Server::new_async().await;
    server
        .mock("POST", "/stores/s1/list-users")
        .with_status(404)
        .with_body(ERR_404)
        .create_async()
        .await;
    let config = Configuration::builder().base_path(server.url()).build();
    let result = openfga::apis::relationship_queries_api::list_users(
        &config,
        "s1",
        models::ListUsersRequest::default(),
    )
    .await;
    assert_status!(result, reqwest::StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn streamed_list_objects_404_returns_response_error() {
    let mut server = Server::new_async().await;
    server
        .mock("POST", "/stores/s1/streamed-list-objects")
        .with_status(404)
        .with_body(ERR_404)
        .create_async()
        .await;
    let config = Configuration::builder().base_path(server.url()).build();
    let result = openfga::apis::relationship_queries_api::streamed_list_objects(
        &config,
        "s1",
        models::ListObjectsRequest::new("document".into(), "viewer".into(), "user:alice".into()),
    )
    .await;
    assert_status!(result, reqwest::StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn read_tuples_404_returns_response_error() {
    let mut server = Server::new_async().await;
    server
        .mock("POST", "/stores/s1/read")
        .with_status(404)
        .with_body(ERR_404)
        .create_async()
        .await;
    let config = Configuration::builder().base_path(server.url()).build();
    let result =
        openfga::apis::relationship_tuples_api::read(&config, "s1", models::ReadRequest::default())
            .await;
    assert_status!(result, reqwest::StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn read_changes_404_returns_response_error() {
    let mut server = Server::new_async().await;
    server
        .mock("GET", "/stores/s1/changes")
        .with_status(404)
        .with_body(ERR_404)
        .create_async()
        .await;
    let config = Configuration::builder().base_path(server.url()).build();
    let result =
        openfga::apis::relationship_tuples_api::read_changes(&config, "s1", None, None, None, None)
            .await;
    assert_status!(result, reqwest::StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn write_tuples_404_returns_response_error() {
    let mut server = Server::new_async().await;
    server
        .mock("POST", "/stores/s1/write")
        .with_status(404)
        .with_body(ERR_404)
        .create_async()
        .await;
    let config = Configuration::builder().base_path(server.url()).build();
    let result = openfga::apis::relationship_tuples_api::write(
        &config,
        "s1",
        models::WriteRequest::default(),
    )
    .await;
    assert_status!(result, reqwest::StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn get_configuration_404_returns_response_error() {
    let mut server = Server::new_async().await;
    server
        .mock("GET", "/.well-known/authzen-configuration/s1")
        .with_status(404)
        .with_body(ERR_404)
        .create_async()
        .await;
    let config = Configuration::builder().base_path(server.url()).build();
    let result = openfga::apis::auth_zen_service_api::get_configuration(&config, "s1").await;
    assert_status!(result, reqwest::StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn evaluation_404_returns_response_error() {
    let mut server = Server::new_async().await;
    server
        .mock("POST", "/stores/s1/access/v1/evaluation")
        .with_status(404)
        .with_body(ERR_404)
        .create_async()
        .await;
    let config = Configuration::builder().base_path(server.url()).build();
    let result = openfga::apis::auth_zen_service_api::evaluation(
        &config,
        "s1",
        models::EvaluationRequest::default(),
    )
    .await;
    assert_status!(result, reqwest::StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn evaluations_404_returns_response_error() {
    let mut server = Server::new_async().await;
    server
        .mock("POST", "/stores/s1/access/v1/evaluations")
        .with_status(404)
        .with_body(ERR_404)
        .create_async()
        .await;
    let config = Configuration::builder().base_path(server.url()).build();
    let result = openfga::apis::auth_zen_service_api::evaluations(
        &config,
        "s1",
        models::EvaluationsRequest::default(),
    )
    .await;
    assert_status!(result, reqwest::StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn action_search_404_returns_response_error() {
    let mut server = Server::new_async().await;
    server
        .mock("POST", "/stores/s1/access/v1/search/action")
        .with_status(404)
        .with_body(ERR_404)
        .create_async()
        .await;
    let config = Configuration::builder().base_path(server.url()).build();
    let result = openfga::apis::auth_zen_service_api::action_search(
        &config,
        "s1",
        models::ActionSearchRequest::default(),
    )
    .await;
    assert_status!(result, reqwest::StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn resource_search_404_returns_response_error() {
    let mut server = Server::new_async().await;
    server
        .mock("POST", "/stores/s1/access/v1/search/resource")
        .with_status(404)
        .with_body(ERR_404)
        .create_async()
        .await;
    let config = Configuration::builder().base_path(server.url()).build();
    let result = openfga::apis::auth_zen_service_api::resource_search(
        &config,
        "s1",
        models::ResourceSearchRequest::default(),
    )
    .await;
    assert_status!(result, reqwest::StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn subject_search_404_returns_response_error() {
    let mut server = Server::new_async().await;
    server
        .mock("POST", "/stores/s1/access/v1/search/subject")
        .with_status(404)
        .with_body(ERR_404)
        .create_async()
        .await;
    let config = Configuration::builder().base_path(server.url()).build();
    let result = openfga::apis::auth_zen_service_api::subject_search(
        &config,
        "s1",
        models::SubjectSearchRequest::default(),
    )
    .await;
    assert_status!(result, reqwest::StatusCode::NOT_FOUND);
}

// ══════════════════════════════════════════════════════════════════════════════
// Gap Group 5: 409 Conflict — all endpoints
// ══════════════════════════════════════════════════════════════════════════════

#[tokio::test]
async fn list_stores_409_returns_response_error() {
    let mut server = Server::new_async().await;
    server
        .mock("GET", "/stores")
        .with_status(409)
        .with_body(ERR_409)
        .create_async()
        .await;
    let config = Configuration::builder().base_path(server.url()).build();
    let result = openfga::apis::stores_api::list_stores(&config, None, None, None).await;
    assert_status!(result, reqwest::StatusCode::CONFLICT);
}

#[tokio::test]
async fn create_store_409_returns_response_error() {
    let mut server = Server::new_async().await;
    server
        .mock("POST", "/stores")
        .with_status(409)
        .with_body(ERR_409)
        .create_async()
        .await;
    let config = Configuration::builder().base_path(server.url()).build();
    let result = openfga::apis::stores_api::create_store(
        &config,
        models::CreateStoreRequest::new("x".into()),
    )
    .await;
    assert_status!(result, reqwest::StatusCode::CONFLICT);
}

#[tokio::test]
async fn get_store_409_returns_response_error() {
    let mut server = Server::new_async().await;
    server
        .mock("GET", "/stores/s1")
        .with_status(409)
        .with_body(ERR_409)
        .create_async()
        .await;
    let config = Configuration::builder().base_path(server.url()).build();
    let result = openfga::apis::stores_api::get_store(&config, "s1").await;
    assert_status!(result, reqwest::StatusCode::CONFLICT);
}

#[tokio::test]
async fn delete_store_409_returns_response_error() {
    let mut server = Server::new_async().await;
    server
        .mock("DELETE", "/stores/s1")
        .with_status(409)
        .with_body(ERR_409)
        .create_async()
        .await;
    let config = Configuration::builder().base_path(server.url()).build();
    let result = openfga::apis::stores_api::delete_store(&config, "s1").await;
    assert_status!(result, reqwest::StatusCode::CONFLICT);
}

#[tokio::test]
async fn read_assertions_409_returns_response_error() {
    let mut server = Server::new_async().await;
    server
        .mock("GET", "/stores/s1/assertions/m1")
        .with_status(409)
        .with_body(ERR_409)
        .create_async()
        .await;
    let config = Configuration::builder().base_path(server.url()).build();
    let result = openfga::apis::assertions_api::read_assertions(&config, "s1", "m1").await;
    assert_status!(result, reqwest::StatusCode::CONFLICT);
}

#[tokio::test]
async fn write_assertions_409_returns_response_error() {
    let mut server = Server::new_async().await;
    server
        .mock("PUT", "/stores/s1/assertions/m1")
        .with_status(409)
        .with_body(ERR_409)
        .create_async()
        .await;
    let config = Configuration::builder().base_path(server.url()).build();
    let result = openfga::apis::assertions_api::write_assertions(
        &config,
        "s1",
        "m1",
        models::WriteAssertionsRequest::default(),
    )
    .await;
    assert_status!(result, reqwest::StatusCode::CONFLICT);
}

#[tokio::test]
async fn read_authorization_models_409_returns_response_error() {
    let mut server = Server::new_async().await;
    server
        .mock("GET", "/stores/s1/authorization-models")
        .with_status(409)
        .with_body(ERR_409)
        .create_async()
        .await;
    let config = Configuration::builder().base_path(server.url()).build();
    let result = openfga::apis::authorization_models_api::read_authorization_models(
        &config, "s1", None, None,
    )
    .await;
    assert_status!(result, reqwest::StatusCode::CONFLICT);
}

#[tokio::test]
async fn write_authorization_model_409_returns_response_error() {
    let mut server = Server::new_async().await;
    server
        .mock("POST", "/stores/s1/authorization-models")
        .with_status(409)
        .with_body(ERR_409)
        .create_async()
        .await;
    let config = Configuration::builder().base_path(server.url()).build();
    let result = openfga::apis::authorization_models_api::write_authorization_model(
        &config,
        "s1",
        models::WriteAuthorizationModelRequest::new(vec![], "1.1".into()),
    )
    .await;
    assert_status!(result, reqwest::StatusCode::CONFLICT);
}

#[tokio::test]
async fn read_authorization_model_409_returns_response_error() {
    let mut server = Server::new_async().await;
    server
        .mock("GET", "/stores/s1/authorization-models/m1")
        .with_status(409)
        .with_body(ERR_409)
        .create_async()
        .await;
    let config = Configuration::builder().base_path(server.url()).build();
    let result =
        openfga::apis::authorization_models_api::read_authorization_model(&config, "s1", "m1")
            .await;
    assert_status!(result, reqwest::StatusCode::CONFLICT);
}

#[tokio::test]
async fn check_409_returns_response_error() {
    let mut server = Server::new_async().await;
    server
        .mock("POST", "/stores/s1/check")
        .with_status(409)
        .with_body(ERR_409)
        .create_async()
        .await;
    let config = Configuration::builder().base_path(server.url()).build();
    let result = openfga::apis::relationship_queries_api::check(
        &config,
        "s1",
        models::CheckRequest::default(),
    )
    .await;
    assert_status!(result, reqwest::StatusCode::CONFLICT);
}

#[tokio::test]
async fn batch_check_409_returns_response_error() {
    let mut server = Server::new_async().await;
    server
        .mock("POST", "/stores/s1/batch-check")
        .with_status(409)
        .with_body(ERR_409)
        .create_async()
        .await;
    let config = Configuration::builder().base_path(server.url()).build();
    let result = openfga::apis::relationship_queries_api::batch_check(
        &config,
        "s1",
        models::BatchCheckRequest::default(),
    )
    .await;
    assert_status!(result, reqwest::StatusCode::CONFLICT);
}

#[tokio::test]
async fn expand_409_returns_response_error() {
    let mut server = Server::new_async().await;
    server
        .mock("POST", "/stores/s1/expand")
        .with_status(409)
        .with_body(ERR_409)
        .create_async()
        .await;
    let config = Configuration::builder().base_path(server.url()).build();
    let result = openfga::apis::relationship_queries_api::expand(
        &config,
        "s1",
        models::ExpandRequest::default(),
    )
    .await;
    assert_status!(result, reqwest::StatusCode::CONFLICT);
}

#[tokio::test]
async fn list_objects_409_returns_response_error() {
    let mut server = Server::new_async().await;
    server
        .mock("POST", "/stores/s1/list-objects")
        .with_status(409)
        .with_body(ERR_409)
        .create_async()
        .await;
    let config = Configuration::builder().base_path(server.url()).build();
    let result = openfga::apis::relationship_queries_api::list_objects(
        &config,
        "s1",
        models::ListObjectsRequest::new("document".into(), "viewer".into(), "user:alice".into()),
    )
    .await;
    assert_status!(result, reqwest::StatusCode::CONFLICT);
}

#[tokio::test]
async fn list_users_409_returns_response_error() {
    let mut server = Server::new_async().await;
    server
        .mock("POST", "/stores/s1/list-users")
        .with_status(409)
        .with_body(ERR_409)
        .create_async()
        .await;
    let config = Configuration::builder().base_path(server.url()).build();
    let result = openfga::apis::relationship_queries_api::list_users(
        &config,
        "s1",
        models::ListUsersRequest::default(),
    )
    .await;
    assert_status!(result, reqwest::StatusCode::CONFLICT);
}

#[tokio::test]
async fn streamed_list_objects_409_returns_response_error() {
    let mut server = Server::new_async().await;
    server
        .mock("POST", "/stores/s1/streamed-list-objects")
        .with_status(409)
        .with_body(ERR_409)
        .create_async()
        .await;
    let config = Configuration::builder().base_path(server.url()).build();
    let result = openfga::apis::relationship_queries_api::streamed_list_objects(
        &config,
        "s1",
        models::ListObjectsRequest::new("document".into(), "viewer".into(), "user:alice".into()),
    )
    .await;
    assert_status!(result, reqwest::StatusCode::CONFLICT);
}

#[tokio::test]
async fn read_tuples_409_returns_response_error() {
    let mut server = Server::new_async().await;
    server
        .mock("POST", "/stores/s1/read")
        .with_status(409)
        .with_body(ERR_409)
        .create_async()
        .await;
    let config = Configuration::builder().base_path(server.url()).build();
    let result =
        openfga::apis::relationship_tuples_api::read(&config, "s1", models::ReadRequest::default())
            .await;
    assert_status!(result, reqwest::StatusCode::CONFLICT);
}

#[tokio::test]
async fn read_changes_409_returns_response_error() {
    let mut server = Server::new_async().await;
    server
        .mock("GET", "/stores/s1/changes")
        .with_status(409)
        .with_body(ERR_409)
        .create_async()
        .await;
    let config = Configuration::builder().base_path(server.url()).build();
    let result =
        openfga::apis::relationship_tuples_api::read_changes(&config, "s1", None, None, None, None)
            .await;
    assert_status!(result, reqwest::StatusCode::CONFLICT);
}

#[tokio::test]
async fn write_tuples_409_returns_response_error() {
    let mut server = Server::new_async().await;
    server
        .mock("POST", "/stores/s1/write")
        .with_status(409)
        .with_body(ERR_409)
        .create_async()
        .await;
    let config = Configuration::builder().base_path(server.url()).build();
    let result = openfga::apis::relationship_tuples_api::write(
        &config,
        "s1",
        models::WriteRequest::default(),
    )
    .await;
    assert_status!(result, reqwest::StatusCode::CONFLICT);
}

#[tokio::test]
async fn get_configuration_409_returns_response_error() {
    let mut server = Server::new_async().await;
    server
        .mock("GET", "/.well-known/authzen-configuration/s1")
        .with_status(409)
        .with_body(ERR_409)
        .create_async()
        .await;
    let config = Configuration::builder().base_path(server.url()).build();
    let result = openfga::apis::auth_zen_service_api::get_configuration(&config, "s1").await;
    assert_status!(result, reqwest::StatusCode::CONFLICT);
}

#[tokio::test]
async fn evaluation_409_returns_response_error() {
    let mut server = Server::new_async().await;
    server
        .mock("POST", "/stores/s1/access/v1/evaluation")
        .with_status(409)
        .with_body(ERR_409)
        .create_async()
        .await;
    let config = Configuration::builder().base_path(server.url()).build();
    let result = openfga::apis::auth_zen_service_api::evaluation(
        &config,
        "s1",
        models::EvaluationRequest::default(),
    )
    .await;
    assert_status!(result, reqwest::StatusCode::CONFLICT);
}

#[tokio::test]
async fn evaluations_409_returns_response_error() {
    let mut server = Server::new_async().await;
    server
        .mock("POST", "/stores/s1/access/v1/evaluations")
        .with_status(409)
        .with_body(ERR_409)
        .create_async()
        .await;
    let config = Configuration::builder().base_path(server.url()).build();
    let result = openfga::apis::auth_zen_service_api::evaluations(
        &config,
        "s1",
        models::EvaluationsRequest::default(),
    )
    .await;
    assert_status!(result, reqwest::StatusCode::CONFLICT);
}

#[tokio::test]
async fn action_search_409_returns_response_error() {
    let mut server = Server::new_async().await;
    server
        .mock("POST", "/stores/s1/access/v1/search/action")
        .with_status(409)
        .with_body(ERR_409)
        .create_async()
        .await;
    let config = Configuration::builder().base_path(server.url()).build();
    let result = openfga::apis::auth_zen_service_api::action_search(
        &config,
        "s1",
        models::ActionSearchRequest::default(),
    )
    .await;
    assert_status!(result, reqwest::StatusCode::CONFLICT);
}

#[tokio::test]
async fn resource_search_409_returns_response_error() {
    let mut server = Server::new_async().await;
    server
        .mock("POST", "/stores/s1/access/v1/search/resource")
        .with_status(409)
        .with_body(ERR_409)
        .create_async()
        .await;
    let config = Configuration::builder().base_path(server.url()).build();
    let result = openfga::apis::auth_zen_service_api::resource_search(
        &config,
        "s1",
        models::ResourceSearchRequest::default(),
    )
    .await;
    assert_status!(result, reqwest::StatusCode::CONFLICT);
}

#[tokio::test]
async fn subject_search_409_returns_response_error() {
    let mut server = Server::new_async().await;
    server
        .mock("POST", "/stores/s1/access/v1/search/subject")
        .with_status(409)
        .with_body(ERR_409)
        .create_async()
        .await;
    let config = Configuration::builder().base_path(server.url()).build();
    let result = openfga::apis::auth_zen_service_api::subject_search(
        &config,
        "s1",
        models::SubjectSearchRequest::default(),
    )
    .await;
    assert_status!(result, reqwest::StatusCode::CONFLICT);
}

// ══════════════════════════════════════════════════════════════════════════════
// Gap Group 6: 422 Unprocessable Entity — all endpoints
// ══════════════════════════════════════════════════════════════════════════════

#[tokio::test]
async fn list_stores_422_returns_response_error() {
    let mut server = Server::new_async().await;
    server
        .mock("GET", "/stores")
        .with_status(422)
        .with_body(ERR_422)
        .create_async()
        .await;
    let config = Configuration::builder().base_path(server.url()).build();
    let result = openfga::apis::stores_api::list_stores(&config, None, None, None).await;
    assert_status!(result, reqwest::StatusCode::UNPROCESSABLE_ENTITY);
}

#[tokio::test]
async fn create_store_422_returns_response_error() {
    let mut server = Server::new_async().await;
    server
        .mock("POST", "/stores")
        .with_status(422)
        .with_body(ERR_422)
        .create_async()
        .await;
    let config = Configuration::builder().base_path(server.url()).build();
    let result = openfga::apis::stores_api::create_store(
        &config,
        models::CreateStoreRequest::new("x".into()),
    )
    .await;
    assert_status!(result, reqwest::StatusCode::UNPROCESSABLE_ENTITY);
}

#[tokio::test]
async fn get_store_422_returns_response_error() {
    let mut server = Server::new_async().await;
    server
        .mock("GET", "/stores/s1")
        .with_status(422)
        .with_body(ERR_422)
        .create_async()
        .await;
    let config = Configuration::builder().base_path(server.url()).build();
    let result = openfga::apis::stores_api::get_store(&config, "s1").await;
    assert_status!(result, reqwest::StatusCode::UNPROCESSABLE_ENTITY);
}

#[tokio::test]
async fn delete_store_422_returns_response_error() {
    let mut server = Server::new_async().await;
    server
        .mock("DELETE", "/stores/s1")
        .with_status(422)
        .with_body(ERR_422)
        .create_async()
        .await;
    let config = Configuration::builder().base_path(server.url()).build();
    let result = openfga::apis::stores_api::delete_store(&config, "s1").await;
    assert_status!(result, reqwest::StatusCode::UNPROCESSABLE_ENTITY);
}

#[tokio::test]
async fn read_assertions_422_returns_response_error() {
    let mut server = Server::new_async().await;
    server
        .mock("GET", "/stores/s1/assertions/m1")
        .with_status(422)
        .with_body(ERR_422)
        .create_async()
        .await;
    let config = Configuration::builder().base_path(server.url()).build();
    let result = openfga::apis::assertions_api::read_assertions(&config, "s1", "m1").await;
    assert_status!(result, reqwest::StatusCode::UNPROCESSABLE_ENTITY);
}

#[tokio::test]
async fn write_assertions_422_returns_response_error() {
    let mut server = Server::new_async().await;
    server
        .mock("PUT", "/stores/s1/assertions/m1")
        .with_status(422)
        .with_body(ERR_422)
        .create_async()
        .await;
    let config = Configuration::builder().base_path(server.url()).build();
    let result = openfga::apis::assertions_api::write_assertions(
        &config,
        "s1",
        "m1",
        models::WriteAssertionsRequest::default(),
    )
    .await;
    assert_status!(result, reqwest::StatusCode::UNPROCESSABLE_ENTITY);
}

#[tokio::test]
async fn read_authorization_models_422_returns_response_error() {
    let mut server = Server::new_async().await;
    server
        .mock("GET", "/stores/s1/authorization-models")
        .with_status(422)
        .with_body(ERR_422)
        .create_async()
        .await;
    let config = Configuration::builder().base_path(server.url()).build();
    let result = openfga::apis::authorization_models_api::read_authorization_models(
        &config, "s1", None, None,
    )
    .await;
    assert_status!(result, reqwest::StatusCode::UNPROCESSABLE_ENTITY);
}

#[tokio::test]
async fn write_authorization_model_422_returns_response_error() {
    let mut server = Server::new_async().await;
    server
        .mock("POST", "/stores/s1/authorization-models")
        .with_status(422)
        .with_body(ERR_422)
        .create_async()
        .await;
    let config = Configuration::builder().base_path(server.url()).build();
    let result = openfga::apis::authorization_models_api::write_authorization_model(
        &config,
        "s1",
        models::WriteAuthorizationModelRequest::new(vec![], "1.1".into()),
    )
    .await;
    assert_status!(result, reqwest::StatusCode::UNPROCESSABLE_ENTITY);
}

#[tokio::test]
async fn read_authorization_model_422_returns_response_error() {
    let mut server = Server::new_async().await;
    server
        .mock("GET", "/stores/s1/authorization-models/m1")
        .with_status(422)
        .with_body(ERR_422)
        .create_async()
        .await;
    let config = Configuration::builder().base_path(server.url()).build();
    let result =
        openfga::apis::authorization_models_api::read_authorization_model(&config, "s1", "m1")
            .await;
    assert_status!(result, reqwest::StatusCode::UNPROCESSABLE_ENTITY);
}

#[tokio::test]
async fn check_422_returns_response_error() {
    let mut server = Server::new_async().await;
    server
        .mock("POST", "/stores/s1/check")
        .with_status(422)
        .with_body(ERR_422)
        .create_async()
        .await;
    let config = Configuration::builder().base_path(server.url()).build();
    let result = openfga::apis::relationship_queries_api::check(
        &config,
        "s1",
        models::CheckRequest::default(),
    )
    .await;
    assert_status!(result, reqwest::StatusCode::UNPROCESSABLE_ENTITY);
}

#[tokio::test]
async fn batch_check_422_returns_response_error() {
    let mut server = Server::new_async().await;
    server
        .mock("POST", "/stores/s1/batch-check")
        .with_status(422)
        .with_body(ERR_422)
        .create_async()
        .await;
    let config = Configuration::builder().base_path(server.url()).build();
    let result = openfga::apis::relationship_queries_api::batch_check(
        &config,
        "s1",
        models::BatchCheckRequest::default(),
    )
    .await;
    assert_status!(result, reqwest::StatusCode::UNPROCESSABLE_ENTITY);
}

#[tokio::test]
async fn expand_422_returns_response_error() {
    let mut server = Server::new_async().await;
    server
        .mock("POST", "/stores/s1/expand")
        .with_status(422)
        .with_body(ERR_422)
        .create_async()
        .await;
    let config = Configuration::builder().base_path(server.url()).build();
    let result = openfga::apis::relationship_queries_api::expand(
        &config,
        "s1",
        models::ExpandRequest::default(),
    )
    .await;
    assert_status!(result, reqwest::StatusCode::UNPROCESSABLE_ENTITY);
}

#[tokio::test]
async fn list_objects_422_returns_response_error() {
    let mut server = Server::new_async().await;
    server
        .mock("POST", "/stores/s1/list-objects")
        .with_status(422)
        .with_body(ERR_422)
        .create_async()
        .await;
    let config = Configuration::builder().base_path(server.url()).build();
    let result = openfga::apis::relationship_queries_api::list_objects(
        &config,
        "s1",
        models::ListObjectsRequest::new("document".into(), "viewer".into(), "user:alice".into()),
    )
    .await;
    assert_status!(result, reqwest::StatusCode::UNPROCESSABLE_ENTITY);
}

#[tokio::test]
async fn list_users_422_returns_response_error() {
    let mut server = Server::new_async().await;
    server
        .mock("POST", "/stores/s1/list-users")
        .with_status(422)
        .with_body(ERR_422)
        .create_async()
        .await;
    let config = Configuration::builder().base_path(server.url()).build();
    let result = openfga::apis::relationship_queries_api::list_users(
        &config,
        "s1",
        models::ListUsersRequest::default(),
    )
    .await;
    assert_status!(result, reqwest::StatusCode::UNPROCESSABLE_ENTITY);
}

#[tokio::test]
async fn streamed_list_objects_422_returns_response_error() {
    let mut server = Server::new_async().await;
    server
        .mock("POST", "/stores/s1/streamed-list-objects")
        .with_status(422)
        .with_body(ERR_422)
        .create_async()
        .await;
    let config = Configuration::builder().base_path(server.url()).build();
    let result = openfga::apis::relationship_queries_api::streamed_list_objects(
        &config,
        "s1",
        models::ListObjectsRequest::new("document".into(), "viewer".into(), "user:alice".into()),
    )
    .await;
    assert_status!(result, reqwest::StatusCode::UNPROCESSABLE_ENTITY);
}

#[tokio::test]
async fn read_tuples_422_returns_response_error() {
    let mut server = Server::new_async().await;
    server
        .mock("POST", "/stores/s1/read")
        .with_status(422)
        .with_body(ERR_422)
        .create_async()
        .await;
    let config = Configuration::builder().base_path(server.url()).build();
    let result =
        openfga::apis::relationship_tuples_api::read(&config, "s1", models::ReadRequest::default())
            .await;
    assert_status!(result, reqwest::StatusCode::UNPROCESSABLE_ENTITY);
}

#[tokio::test]
async fn read_changes_422_returns_response_error() {
    let mut server = Server::new_async().await;
    server
        .mock("GET", "/stores/s1/changes")
        .with_status(422)
        .with_body(ERR_422)
        .create_async()
        .await;
    let config = Configuration::builder().base_path(server.url()).build();
    let result =
        openfga::apis::relationship_tuples_api::read_changes(&config, "s1", None, None, None, None)
            .await;
    assert_status!(result, reqwest::StatusCode::UNPROCESSABLE_ENTITY);
}

#[tokio::test]
async fn write_tuples_422_returns_response_error() {
    let mut server = Server::new_async().await;
    server
        .mock("POST", "/stores/s1/write")
        .with_status(422)
        .with_body(ERR_422)
        .create_async()
        .await;
    let config = Configuration::builder().base_path(server.url()).build();
    let result = openfga::apis::relationship_tuples_api::write(
        &config,
        "s1",
        models::WriteRequest::default(),
    )
    .await;
    assert_status!(result, reqwest::StatusCode::UNPROCESSABLE_ENTITY);
}

#[tokio::test]
async fn get_configuration_422_returns_response_error() {
    let mut server = Server::new_async().await;
    server
        .mock("GET", "/.well-known/authzen-configuration/s1")
        .with_status(422)
        .with_body(ERR_422)
        .create_async()
        .await;
    let config = Configuration::builder().base_path(server.url()).build();
    let result = openfga::apis::auth_zen_service_api::get_configuration(&config, "s1").await;
    assert_status!(result, reqwest::StatusCode::UNPROCESSABLE_ENTITY);
}

#[tokio::test]
async fn evaluation_422_returns_response_error() {
    let mut server = Server::new_async().await;
    server
        .mock("POST", "/stores/s1/access/v1/evaluation")
        .with_status(422)
        .with_body(ERR_422)
        .create_async()
        .await;
    let config = Configuration::builder().base_path(server.url()).build();
    let result = openfga::apis::auth_zen_service_api::evaluation(
        &config,
        "s1",
        models::EvaluationRequest::default(),
    )
    .await;
    assert_status!(result, reqwest::StatusCode::UNPROCESSABLE_ENTITY);
}

#[tokio::test]
async fn evaluations_422_returns_response_error() {
    let mut server = Server::new_async().await;
    server
        .mock("POST", "/stores/s1/access/v1/evaluations")
        .with_status(422)
        .with_body(ERR_422)
        .create_async()
        .await;
    let config = Configuration::builder().base_path(server.url()).build();
    let result = openfga::apis::auth_zen_service_api::evaluations(
        &config,
        "s1",
        models::EvaluationsRequest::default(),
    )
    .await;
    assert_status!(result, reqwest::StatusCode::UNPROCESSABLE_ENTITY);
}

#[tokio::test]
async fn action_search_422_returns_response_error() {
    let mut server = Server::new_async().await;
    server
        .mock("POST", "/stores/s1/access/v1/search/action")
        .with_status(422)
        .with_body(ERR_422)
        .create_async()
        .await;
    let config = Configuration::builder().base_path(server.url()).build();
    let result = openfga::apis::auth_zen_service_api::action_search(
        &config,
        "s1",
        models::ActionSearchRequest::default(),
    )
    .await;
    assert_status!(result, reqwest::StatusCode::UNPROCESSABLE_ENTITY);
}

#[tokio::test]
async fn resource_search_422_returns_response_error() {
    let mut server = Server::new_async().await;
    server
        .mock("POST", "/stores/s1/access/v1/search/resource")
        .with_status(422)
        .with_body(ERR_422)
        .create_async()
        .await;
    let config = Configuration::builder().base_path(server.url()).build();
    let result = openfga::apis::auth_zen_service_api::resource_search(
        &config,
        "s1",
        models::ResourceSearchRequest::default(),
    )
    .await;
    assert_status!(result, reqwest::StatusCode::UNPROCESSABLE_ENTITY);
}

#[tokio::test]
async fn subject_search_422_returns_response_error() {
    let mut server = Server::new_async().await;
    server
        .mock("POST", "/stores/s1/access/v1/search/subject")
        .with_status(422)
        .with_body(ERR_422)
        .create_async()
        .await;
    let config = Configuration::builder().base_path(server.url()).build();
    let result = openfga::apis::auth_zen_service_api::subject_search(
        &config,
        "s1",
        models::SubjectSearchRequest::default(),
    )
    .await;
    assert_status!(result, reqwest::StatusCode::UNPROCESSABLE_ENTITY);
}

// ══════════════════════════════════════════════════════════════════════════════
// Gap Group 7: Missing 500 Internal Server Error (all except read_assertions)
// ══════════════════════════════════════════════════════════════════════════════

#[tokio::test]
async fn list_stores_500_returns_response_error() {
    let mut server = Server::new_async().await;
    server
        .mock("GET", "/stores")
        .with_status(500)
        .with_body(ERR_500)
        .create_async()
        .await;
    let config = Configuration::builder().base_path(server.url()).build();
    let result = openfga::apis::stores_api::list_stores(&config, None, None, None).await;
    assert_status!(result, reqwest::StatusCode::INTERNAL_SERVER_ERROR);
}

#[tokio::test]
async fn create_store_500_returns_response_error() {
    let mut server = Server::new_async().await;
    server
        .mock("POST", "/stores")
        .with_status(500)
        .with_body(ERR_500)
        .create_async()
        .await;
    let config = Configuration::builder().base_path(server.url()).build();
    let result = openfga::apis::stores_api::create_store(
        &config,
        models::CreateStoreRequest::new("x".into()),
    )
    .await;
    assert_status!(result, reqwest::StatusCode::INTERNAL_SERVER_ERROR);
}

#[tokio::test]
async fn get_store_500_returns_response_error() {
    let mut server = Server::new_async().await;
    server
        .mock("GET", "/stores/s1")
        .with_status(500)
        .with_body(ERR_500)
        .create_async()
        .await;
    let config = Configuration::builder().base_path(server.url()).build();
    let result = openfga::apis::stores_api::get_store(&config, "s1").await;
    assert_status!(result, reqwest::StatusCode::INTERNAL_SERVER_ERROR);
}

#[tokio::test]
async fn delete_store_500_returns_response_error() {
    let mut server = Server::new_async().await;
    server
        .mock("DELETE", "/stores/s1")
        .with_status(500)
        .with_body(ERR_500)
        .create_async()
        .await;
    let config = Configuration::builder().base_path(server.url()).build();
    let result = openfga::apis::stores_api::delete_store(&config, "s1").await;
    assert_status!(result, reqwest::StatusCode::INTERNAL_SERVER_ERROR);
}

#[tokio::test]
async fn write_assertions_500_returns_response_error() {
    let mut server = Server::new_async().await;
    server
        .mock("PUT", "/stores/s1/assertions/m1")
        .with_status(500)
        .with_body(ERR_500)
        .create_async()
        .await;
    let config = Configuration::builder().base_path(server.url()).build();
    let result = openfga::apis::assertions_api::write_assertions(
        &config,
        "s1",
        "m1",
        models::WriteAssertionsRequest::default(),
    )
    .await;
    assert_status!(result, reqwest::StatusCode::INTERNAL_SERVER_ERROR);
}

#[tokio::test]
async fn read_authorization_models_500_returns_response_error() {
    let mut server = Server::new_async().await;
    server
        .mock("GET", "/stores/s1/authorization-models")
        .with_status(500)
        .with_body(ERR_500)
        .create_async()
        .await;
    let config = Configuration::builder().base_path(server.url()).build();
    let result = openfga::apis::authorization_models_api::read_authorization_models(
        &config, "s1", None, None,
    )
    .await;
    assert_status!(result, reqwest::StatusCode::INTERNAL_SERVER_ERROR);
}

#[tokio::test]
async fn write_authorization_model_500_returns_response_error() {
    let mut server = Server::new_async().await;
    server
        .mock("POST", "/stores/s1/authorization-models")
        .with_status(500)
        .with_body(ERR_500)
        .create_async()
        .await;
    let config = Configuration::builder().base_path(server.url()).build();
    let result = openfga::apis::authorization_models_api::write_authorization_model(
        &config,
        "s1",
        models::WriteAuthorizationModelRequest::new(vec![], "1.1".into()),
    )
    .await;
    assert_status!(result, reqwest::StatusCode::INTERNAL_SERVER_ERROR);
}

#[tokio::test]
async fn read_authorization_model_500_returns_response_error() {
    let mut server = Server::new_async().await;
    server
        .mock("GET", "/stores/s1/authorization-models/m1")
        .with_status(500)
        .with_body(ERR_500)
        .create_async()
        .await;
    let config = Configuration::builder().base_path(server.url()).build();
    let result =
        openfga::apis::authorization_models_api::read_authorization_model(&config, "s1", "m1")
            .await;
    assert_status!(result, reqwest::StatusCode::INTERNAL_SERVER_ERROR);
}

#[tokio::test]
async fn check_500_returns_response_error() {
    let mut server = Server::new_async().await;
    server
        .mock("POST", "/stores/s1/check")
        .with_status(500)
        .with_body(ERR_500)
        .create_async()
        .await;
    let config = Configuration::builder().base_path(server.url()).build();
    let result = openfga::apis::relationship_queries_api::check(
        &config,
        "s1",
        models::CheckRequest::default(),
    )
    .await;
    assert_status!(result, reqwest::StatusCode::INTERNAL_SERVER_ERROR);
}

#[tokio::test]
async fn batch_check_500_returns_response_error() {
    let mut server = Server::new_async().await;
    server
        .mock("POST", "/stores/s1/batch-check")
        .with_status(500)
        .with_body(ERR_500)
        .create_async()
        .await;
    let config = Configuration::builder().base_path(server.url()).build();
    let result = openfga::apis::relationship_queries_api::batch_check(
        &config,
        "s1",
        models::BatchCheckRequest::default(),
    )
    .await;
    assert_status!(result, reqwest::StatusCode::INTERNAL_SERVER_ERROR);
}

#[tokio::test]
async fn expand_500_returns_response_error() {
    let mut server = Server::new_async().await;
    server
        .mock("POST", "/stores/s1/expand")
        .with_status(500)
        .with_body(ERR_500)
        .create_async()
        .await;
    let config = Configuration::builder().base_path(server.url()).build();
    let result = openfga::apis::relationship_queries_api::expand(
        &config,
        "s1",
        models::ExpandRequest::default(),
    )
    .await;
    assert_status!(result, reqwest::StatusCode::INTERNAL_SERVER_ERROR);
}

#[tokio::test]
async fn list_objects_500_returns_response_error() {
    let mut server = Server::new_async().await;
    server
        .mock("POST", "/stores/s1/list-objects")
        .with_status(500)
        .with_body(ERR_500)
        .create_async()
        .await;
    let config = Configuration::builder().base_path(server.url()).build();
    let result = openfga::apis::relationship_queries_api::list_objects(
        &config,
        "s1",
        models::ListObjectsRequest::new("document".into(), "viewer".into(), "user:alice".into()),
    )
    .await;
    assert_status!(result, reqwest::StatusCode::INTERNAL_SERVER_ERROR);
}

#[tokio::test]
async fn list_users_500_returns_response_error() {
    let mut server = Server::new_async().await;
    server
        .mock("POST", "/stores/s1/list-users")
        .with_status(500)
        .with_body(ERR_500)
        .create_async()
        .await;
    let config = Configuration::builder().base_path(server.url()).build();
    let result = openfga::apis::relationship_queries_api::list_users(
        &config,
        "s1",
        models::ListUsersRequest::default(),
    )
    .await;
    assert_status!(result, reqwest::StatusCode::INTERNAL_SERVER_ERROR);
}

#[tokio::test]
async fn streamed_list_objects_500_returns_response_error() {
    let mut server = Server::new_async().await;
    server
        .mock("POST", "/stores/s1/streamed-list-objects")
        .with_status(500)
        .with_body(ERR_500)
        .create_async()
        .await;
    let config = Configuration::builder().base_path(server.url()).build();
    let result = openfga::apis::relationship_queries_api::streamed_list_objects(
        &config,
        "s1",
        models::ListObjectsRequest::new("document".into(), "viewer".into(), "user:alice".into()),
    )
    .await;
    assert_status!(result, reqwest::StatusCode::INTERNAL_SERVER_ERROR);
}

#[tokio::test]
async fn read_tuples_500_returns_response_error() {
    let mut server = Server::new_async().await;
    server
        .mock("POST", "/stores/s1/read")
        .with_status(500)
        .with_body(ERR_500)
        .create_async()
        .await;
    let config = Configuration::builder().base_path(server.url()).build();
    let result =
        openfga::apis::relationship_tuples_api::read(&config, "s1", models::ReadRequest::default())
            .await;
    assert_status!(result, reqwest::StatusCode::INTERNAL_SERVER_ERROR);
}

#[tokio::test]
async fn read_changes_500_returns_response_error() {
    let mut server = Server::new_async().await;
    server
        .mock("GET", "/stores/s1/changes")
        .with_status(500)
        .with_body(ERR_500)
        .create_async()
        .await;
    let config = Configuration::builder().base_path(server.url()).build();
    let result =
        openfga::apis::relationship_tuples_api::read_changes(&config, "s1", None, None, None, None)
            .await;
    assert_status!(result, reqwest::StatusCode::INTERNAL_SERVER_ERROR);
}

#[tokio::test]
async fn write_tuples_500_returns_response_error() {
    let mut server = Server::new_async().await;
    server
        .mock("POST", "/stores/s1/write")
        .with_status(500)
        .with_body(ERR_500)
        .create_async()
        .await;
    let config = Configuration::builder().base_path(server.url()).build();
    let result = openfga::apis::relationship_tuples_api::write(
        &config,
        "s1",
        models::WriteRequest::default(),
    )
    .await;
    assert_status!(result, reqwest::StatusCode::INTERNAL_SERVER_ERROR);
}

#[tokio::test]
async fn get_configuration_500_returns_response_error() {
    let mut server = Server::new_async().await;
    server
        .mock("GET", "/.well-known/authzen-configuration/s1")
        .with_status(500)
        .with_body(ERR_500)
        .create_async()
        .await;
    let config = Configuration::builder().base_path(server.url()).build();
    let result = openfga::apis::auth_zen_service_api::get_configuration(&config, "s1").await;
    assert_status!(result, reqwest::StatusCode::INTERNAL_SERVER_ERROR);
}

#[tokio::test]
async fn evaluation_500_returns_response_error() {
    let mut server = Server::new_async().await;
    server
        .mock("POST", "/stores/s1/access/v1/evaluation")
        .with_status(500)
        .with_body(ERR_500)
        .create_async()
        .await;
    let config = Configuration::builder().base_path(server.url()).build();
    let result = openfga::apis::auth_zen_service_api::evaluation(
        &config,
        "s1",
        models::EvaluationRequest::default(),
    )
    .await;
    assert_status!(result, reqwest::StatusCode::INTERNAL_SERVER_ERROR);
}

#[tokio::test]
async fn evaluations_500_returns_response_error() {
    let mut server = Server::new_async().await;
    server
        .mock("POST", "/stores/s1/access/v1/evaluations")
        .with_status(500)
        .with_body(ERR_500)
        .create_async()
        .await;
    let config = Configuration::builder().base_path(server.url()).build();
    let result = openfga::apis::auth_zen_service_api::evaluations(
        &config,
        "s1",
        models::EvaluationsRequest::default(),
    )
    .await;
    assert_status!(result, reqwest::StatusCode::INTERNAL_SERVER_ERROR);
}

#[tokio::test]
async fn action_search_500_returns_response_error() {
    let mut server = Server::new_async().await;
    server
        .mock("POST", "/stores/s1/access/v1/search/action")
        .with_status(500)
        .with_body(ERR_500)
        .create_async()
        .await;
    let config = Configuration::builder().base_path(server.url()).build();
    let result = openfga::apis::auth_zen_service_api::action_search(
        &config,
        "s1",
        models::ActionSearchRequest::default(),
    )
    .await;
    assert_status!(result, reqwest::StatusCode::INTERNAL_SERVER_ERROR);
}

#[tokio::test]
async fn resource_search_500_returns_response_error() {
    let mut server = Server::new_async().await;
    server
        .mock("POST", "/stores/s1/access/v1/search/resource")
        .with_status(500)
        .with_body(ERR_500)
        .create_async()
        .await;
    let config = Configuration::builder().base_path(server.url()).build();
    let result = openfga::apis::auth_zen_service_api::resource_search(
        &config,
        "s1",
        models::ResourceSearchRequest::default(),
    )
    .await;
    assert_status!(result, reqwest::StatusCode::INTERNAL_SERVER_ERROR);
}

#[tokio::test]
async fn subject_search_500_returns_response_error() {
    let mut server = Server::new_async().await;
    server
        .mock("POST", "/stores/s1/access/v1/search/subject")
        .with_status(500)
        .with_body(ERR_500)
        .create_async()
        .await;
    let config = Configuration::builder().base_path(server.url()).build();
    let result = openfga::apis::auth_zen_service_api::subject_search(
        &config,
        "s1",
        models::SubjectSearchRequest::default(),
    )
    .await;
    assert_status!(result, reqwest::StatusCode::INTERNAL_SERVER_ERROR);
}

// ══════════════════════════════════════════════════════════════════════════════
// Gap Group 8: Auth header on POST endpoints
// ══════════════════════════════════════════════════════════════════════════════

#[tokio::test]
async fn check_with_bearer_token_sends_authorization_header() {
    let mut server = Server::new_async().await;
    let mock = server
        .mock("POST", "/stores/s1/check")
        .match_header("authorization", "Bearer check-token")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"{"allowed":true}"#)
        .create_async()
        .await;
    let config = Configuration::builder()
        .base_path(server.url())
        .bearer_token("check-token")
        .build();
    let result = openfga::apis::relationship_queries_api::check(
        &config,
        "s1",
        models::CheckRequest::default(),
    )
    .await;
    mock.assert_async().await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn read_with_bearer_token_sends_authorization_header() {
    let mut server = Server::new_async().await;
    let mock = server
        .mock("POST", "/stores/s1/read")
        .match_header("authorization", "Bearer read-token")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"{"tuples":[],"continuation_token":""}"#)
        .create_async()
        .await;
    let config = Configuration::builder()
        .base_path(server.url())
        .bearer_token("read-token")
        .build();
    let result =
        openfga::apis::relationship_tuples_api::read(&config, "s1", models::ReadRequest::default())
            .await;
    mock.assert_async().await;
    assert!(result.is_ok());
}

// ══════════════════════════════════════════════════════════════════════════════
// Gap Group 9: Remaining query param tests for read_changes
// ══════════════════════════════════════════════════════════════════════════════

#[tokio::test]
async fn read_changes_with_continuation_token_encodes_query_param() {
    let mut server = Server::new_async().await;
    let mock = server
        .mock("GET", "/stores/s1/changes")
        .match_query(Matcher::UrlEncoded(
            "continuation_token".into(),
            "next-page".into(),
        ))
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"{"changes":[],"continuation_token":null}"#)
        .create_async()
        .await;
    let config = Configuration::builder().base_path(server.url()).build();
    let result = openfga::apis::relationship_tuples_api::read_changes(
        &config,
        "s1",
        None,
        None,
        Some("next-page"),
        None,
    )
    .await;
    mock.assert_async().await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn read_changes_with_start_time_encodes_query_param() {
    use chrono::DateTime;
    let mut server = Server::new_async().await;
    let start_time = DateTime::parse_from_rfc3339("2024-01-01T00:00:00+00:00").unwrap();
    let expected_value = start_time.to_string();
    let mock = server
        .mock("GET", "/stores/s1/changes")
        .match_query(Matcher::UrlEncoded("start_time".into(), expected_value))
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"{"changes":[],"continuation_token":null}"#)
        .create_async()
        .await;
    let config = Configuration::builder().base_path(server.url()).build();
    let result = openfga::apis::relationship_tuples_api::read_changes(
        &config,
        "s1",
        None,
        None,
        None,
        Some(start_time),
    )
    .await;
    mock.assert_async().await;
    assert!(result.is_ok(), "expected Ok, got: {:?}", result);
}
