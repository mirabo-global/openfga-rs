# Userset

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**this** | Option<[**serde_json::Value**](.md)> | A DirectUserset is a sentinel message for referencing the direct members specified by an object/relation mapping. | [optional]
**computed_userset** | Option<[**models::ObjectRelation**](ObjectRelation.md)> |  | [optional]
**tuple_to_userset** | Option<[**models::V1PeriodTupleToUserset**](v1.TupleToUserset.md)> |  | [optional]
**union** | Option<[**models::Usersets**](Usersets.md)> |  | [optional]
**intersection** | Option<[**models::Usersets**](Usersets.md)> |  | [optional]
**difference** | Option<[**models::V1PeriodDifference**](v1.Difference.md)> |  | [optional]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


