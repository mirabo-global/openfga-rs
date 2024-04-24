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

/// NullValue : `NullValue` is a singleton enumeration to represent the null value for the `Value` type union.  The JSON representation for `NullValue` is JSON `null`.   - NULL_VALUE: Null value.
/// `NullValue` is a singleton enumeration to represent the null value for the `Value` type union.  The JSON representation for `NullValue` is JSON `null`.   - NULL_VALUE: Null value.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize)]
pub enum NullValue {
    #[serde(rename = "NULL_VALUE")]
    NullValue,

}

impl ToString for NullValue {
    fn to_string(&self) -> String {
        match self {
            Self::NullValue => String::from("NULL_VALUE"),
        }
    }
}

impl Default for NullValue {
    fn default() -> NullValue {
        Self::NullValue
    }
}

