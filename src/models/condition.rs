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
pub struct Condition {
    #[serde(rename = "name")]
    pub name: String,
    /// A Google CEL expression, expressed as a string.
    #[serde(rename = "expression")]
    pub expression: String,
    /// A map of parameter names to the parameter's defined type reference.
    #[serde(rename = "parameters", skip_serializing_if = "Option::is_none")]
    pub parameters: Option<std::collections::HashMap<String, models::ConditionParamTypeRef>>,
    #[serde(rename = "metadata", skip_serializing_if = "Option::is_none")]
    pub metadata: Option<Box<models::ConditionMetadata>>,
}

impl Condition {
    pub fn new(name: String, expression: String) -> Condition {
        Condition {
            name,
            expression,
            parameters: None,
            metadata: None,
        }
    }
}

