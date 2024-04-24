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
pub struct Assertion {
    #[serde(rename = "tuple_key")]
    pub tuple_key: Box<models::AssertionTupleKey>,
    #[serde(rename = "expectation")]
    pub expectation: bool,
}

impl Assertion {
    pub fn new(tuple_key: models::AssertionTupleKey, expectation: bool) -> Assertion {
        Assertion {
            tuple_key: Box::new(tuple_key),
            expectation,
        }
    }
}

