# openfga

[![crates.io](https://img.shields.io/crates/v/openfga.svg)](https://crates.io/crates/openfga)
[![docs.rs](https://docs.rs/openfga/badge.svg)](https://docs.rs/openfga)
[![license](https://img.shields.io/crates/l/openfga.svg)](LICENSE)

Async Rust SDK for [OpenFGA](https://openfga.dev) — the open-source authorization system inspired by Google Zanzibar.

Generated from the [OpenFGA OpenAPI spec v1.x](https://raw.githubusercontent.com/openfga/api/main/docs/openapiv2/apidocs.swagger.json) using [OpenAPI Generator](https://openapi-generator.tech) 7.22.0, then hardened with security and correctness fixes.

## Installation

```toml
[dependencies]
openfga = "1.0.0"
```

TLS backend (choose one, `native-tls` is the default):

```toml
# Use the platform's native TLS (default)
openfga = "1.0.0"

# Use rustls instead
openfga = { version = "1.0.0", default-features = false, features = ["rustls"] }
```

## Quick start

```rust
use openfga::prelude::*;
use openfga::apis::{configuration::Configuration, stores_api};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = Configuration::builder()
        .base_path("https://api.fga.example.com")
        .bearer_token("my-token")
        .build();

    let stores = stores_api::list_stores(&config, None, None, None).await?;
    println!("{:?}", stores);
    Ok(())
}
```

## Authentication

Use `ConfigurationBuilder` to set exactly one auth method:

```rust
// Bearer token
let config = Configuration::builder()
    .base_path("https://api.fga.example.com")
    .bearer_token("my-token")
    .build();

// HTTP Basic
let config = Configuration::builder()
    .base_path("https://api.fga.example.com")
    .basic_auth("user", Some("pass"))
    .build();

// API key with prefix
let config = Configuration::builder()
    .base_path("https://api.fga.example.com")
    .api_key("my-key", Some("Token"))
    .build();

// Custom timeout (default: 30 s)
let config = Configuration::builder()
    .base_path("https://api.fga.example.com")
    .bearer_token("my-token")
    .timeout(std::time::Duration::from_secs(10))
    .build();
```

> **Security** — `AuthMethod` and `BasicAuth` implement `Debug` with all credential fields redacted as `[REDACTED]`. Tokens are never exposed through logging or panic output.

## API endpoints

### Stores (`apis::stores_api`)

| Function | Method | Path |
|----------|--------|------|
| `create_store` | POST | `/stores` |
| `list_stores` | GET | `/stores` |
| `get_store` | GET | `/stores/{store_id}` |
| `delete_store` | DELETE | `/stores/{store_id}` |

### Authorization Models (`apis::authorization_models_api`)

| Function | Method | Path |
|----------|--------|------|
| `write_authorization_model` | POST | `/stores/{store_id}/authorization-models` |
| `read_authorization_models` | GET | `/stores/{store_id}/authorization-models` |
| `read_authorization_model` | GET | `/stores/{store_id}/authorization-models/{id}` |

### Relationship Tuples (`apis::relationship_tuples_api`)

| Function | Method | Path |
|----------|--------|------|
| `write` | POST | `/stores/{store_id}/write` |
| `read` | POST | `/stores/{store_id}/read` |
| `read_changes` | GET | `/stores/{store_id}/changes` |

### Relationship Queries (`apis::relationship_queries_api`)

| Function | Method | Path |
|----------|--------|------|
| `check` | POST | `/stores/{store_id}/check` |
| `batch_check` | POST | `/stores/{store_id}/batch-check` |
| `expand` | POST | `/stores/{store_id}/expand` |
| `list_objects` | POST | `/stores/{store_id}/list-objects` |
| `streamed_list_objects` | POST | `/stores/{store_id}/streamed-list-objects` |
| `list_users` | POST | `/stores/{store_id}/list-users` |

### Assertions (`apis::assertions_api`)

| Function | Method | Path |
|----------|--------|------|
| `write_assertions` | PUT | `/stores/{store_id}/assertions/{authorization_model_id}` |
| `read_assertions` | GET | `/stores/{store_id}/assertions/{authorization_model_id}` |

### AuthZen (`apis::auth_zen_service_api`)

| Function | Method | Path |
|----------|--------|------|
| `evaluation` | POST | `/access/v1/evaluation` |
| `evaluations` | POST | `/access/v1/evaluations` |
| `action_search` | POST | `/access/v1/search/actions` |
| `subject_search` | POST | `/access/v1/search/subjects` |
| `resource_search` | POST | `/access/v1/search/resources` |
| `get_configuration` | GET | `/access/v1/configuration` |

## Error handling

Every API function returns `Result<T, apis::Error<E>>` where `E` is the endpoint-specific typed error enum:

```rust
use openfga::prelude::*;
use openfga::apis::{configuration::Configuration, stores_api, stores_api::GetStoreError};

match stores_api::get_store(&config, "my-store-id").await {
    Ok(store) => println!("store: {:?}", store),
    Err(Error::ResponseError(rc)) => match rc.entity {
        Some(GetStoreError::Status404(_)) => println!("store not found"),
        Some(GetStoreError::Status401(_)) => println!("unauthorized"),
        _ => println!("HTTP {}: {}", rc.status, rc.content),
    },
    Err(e) => eprintln!("request error: {}", e),
}
```

## License

MIT — see [LICENSE](LICENSE).

## Links

- [OpenFGA docs](https://openfga.dev/docs)
- [crates.io](https://crates.io/crates/openfga)
- [docs.rs](https://docs.rs/openfga)
- [Source](https://github.com/mirabo-global/openfga-rs)
- [OpenFGA community](https://discord.gg/8naAwJfWN6)
