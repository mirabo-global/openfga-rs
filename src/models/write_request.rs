/*
 * OpenFGA
 *
 * A high performance and flexible authorization/permission engine built for developers and inspired by Google Zanzibar.
 *
 * The version of the OpenAPI document: 0.1
 * Contact: community@openfga.dev
 * Generated by: https://openapi-generator.tech
 */

use crate::models;

#[derive(Clone, Default, Debug, PartialEq, Serialize, Deserialize)]
pub struct WriteRequest {
    #[serde(rename = "writes", skip_serializing_if = "Option::is_none")]
    pub writes: Option<Box<models::WriteRequestWrites>>,
    #[serde(rename = "deletes", skip_serializing_if = "Option::is_none")]
    pub deletes: Option<Box<models::WriteRequestDeletes>>,
    #[serde(rename = "authorization_model_id", skip_serializing_if = "Option::is_none")]
    pub authorization_model_id: Option<String>,
}

impl WriteRequest {
    pub fn new() -> WriteRequest {
        WriteRequest {
            writes: None,
            deletes: None,
            authorization_model_id: None,
        }
    }
}

