//! # OpenFGA Rust SDK
//!
//! Async Rust client for [OpenFGA](https://openfga.dev) — the open-source
//! authorization system inspired by Google Zanzibar.
//!
//! ## Quick start
//!
//! ```no_run
//! use openfga::prelude::*;
//! use openfga::apis::{configuration::Configuration, stores_api};
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let config = Configuration::builder()
//!         .base_path("https://api.fga.example.com")
//!         .bearer_token("my-token")
//!         .build();
//!
//!     let stores = stores_api::list_stores(&config, None, None, None).await?;
//!     println!("{:?}", stores);
//!     Ok(())
//! }
//! ```
//!
//! ## Auth methods
//!
//! Use [`configuration::ConfigurationBuilder`] to set exactly one auth method:
//!
//! | Method | Builder call |
//! |--------|-------------|
//! | Bearer token | `.bearer_token("…")` |
//! | OAuth2 token | `.oauth_token("…")` |
//! | Basic auth | `.basic_auth("user", Some("pass"))` |
//! | API key | `.api_key("key", Some("prefix"))` |

#![allow(clippy::too_many_arguments)]

pub mod apis;
pub mod models;

/// Re-exports of the most commonly used types.
///
/// `use openfga::prelude::*;` to bring [`Configuration`],
/// [`ConfigurationBuilder`], and [`Error`] into scope.
pub mod prelude {
    pub use crate::apis::Error;
    pub use crate::apis::ResponseContent;
    pub use crate::apis::configuration::{
        AuthMethod, BasicAuth, Configuration, ConfigurationBuilder,
    };
}
