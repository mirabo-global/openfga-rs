# CheckRequest

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**tuple_key** | [**models::CheckRequestTupleKey**](CheckRequestTupleKey.md) |  | 
**contextual_tuples** | Option<[**models::ContextualTupleKeys**](ContextualTupleKeys.md)> |  | [optional]
**authorization_model_id** | Option<**String**> |  | [optional]
**trace** | Option<**bool**> | Defaults to false. Making it true has performance implications. | [optional][readonly]
**context** | Option<[**serde_json::Value**](.md)> | Additional request context that will be used to evaluate any ABAC conditions encountered in the query evaluation. | [optional]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


