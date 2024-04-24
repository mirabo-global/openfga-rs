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
pub struct ConditionParamTypeRef {
    #[serde(rename = "type_name")]
    pub type_name: models::TypeName,
    #[serde(rename = "generic_types", skip_serializing_if = "Option::is_none")]
    pub generic_types: Option<Vec<models::ConditionParamTypeRef>>,
}

impl ConditionParamTypeRef {
    pub fn new(type_name: models::TypeName) -> ConditionParamTypeRef {
        ConditionParamTypeRef {
            type_name,
            generic_types: None,
        }
    }
}
