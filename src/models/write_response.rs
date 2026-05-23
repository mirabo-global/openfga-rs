/*
 * OpenFGA
 *
 * A high performance and flexible authorization/permission engine built for developers and inspired by Google Zanzibar.
 *
 * The version of the OpenAPI document: 1.x
 * Contact: community@openfga.dev
 */

use serde::{Deserialize, Serialize};

/// Response returned by the Write API on success.
///
/// The server returns an empty JSON object (`{}`); this struct captures that
/// contract so callers get a typed value rather than a raw [`serde_json::Value`].
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct WriteResponse {}
