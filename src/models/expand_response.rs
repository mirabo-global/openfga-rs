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
pub struct ExpandResponse {
    #[serde(rename = "tree", skip_serializing_if = "Option::is_none")]
    pub tree: Option<Box<models::UsersetTree>>,
}

impl ExpandResponse {
    pub fn new() -> ExpandResponse {
        ExpandResponse {
            tree: None,
        }
    }
}

