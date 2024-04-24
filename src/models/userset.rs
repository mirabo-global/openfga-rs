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
pub struct Userset {
    /// A DirectUserset is a sentinel message for referencing the direct members specified by an object/relation mapping.
    #[serde(rename = "this", skip_serializing_if = "Option::is_none")]
    pub this: Option<serde_json::Value>,
    #[serde(rename = "computedUserset", skip_serializing_if = "Option::is_none")]
    pub computed_userset: Option<Box<models::ObjectRelation>>,
    #[serde(rename = "tupleToUserset", skip_serializing_if = "Option::is_none")]
    pub tuple_to_userset: Option<Box<models::V1PeriodTupleToUserset>>,
    #[serde(rename = "union", skip_serializing_if = "Option::is_none")]
    pub union: Option<Box<models::Usersets>>,
    #[serde(rename = "intersection", skip_serializing_if = "Option::is_none")]
    pub intersection: Option<Box<models::Usersets>>,
    #[serde(rename = "difference", skip_serializing_if = "Option::is_none")]
    pub difference: Option<Box<models::V1PeriodDifference>>,
}

impl Userset {
    pub fn new() -> Userset {
        Userset {
            this: None,
            computed_userset: None,
            tuple_to_userset: None,
            union: None,
            intersection: None,
            difference: None,
        }
    }
}

