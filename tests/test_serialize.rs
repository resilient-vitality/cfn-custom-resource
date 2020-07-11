use pretty_assertions::assert_eq;

use cfn_custom_resource::{CloudformationPayload, CustomResourceEvent};
use serde::Deserialize;

#[derive(Debug, Deserialize, PartialEq)]
struct PayloadData {
    a: String,
    b: Vec<String>,
}

#[test]
fn test_serialize() {
    let data = std::fs::read_to_string("tests/test_delete_event.json").unwrap();
    let actual: CustomResourceEvent<PayloadData> = serde_json::from_str(&data).unwrap();
    let resource_properties = PayloadData {
        a: "string".to_owned(),
        b: vec!["list".to_owned()],
    };
    let exepected_payload = CloudformationPayload {
        request_id: "unique id for this delete request".to_owned(),
        response_url: "pre-signed-url-for-delete-response".to_owned(),
        resource_type: "Custom::MyCustomResourceType".to_owned(),
        stack_id: "arn:aws:cloudformation:us-east-2:namespace:stack/stack-name/guid".to_owned(),
        logical_resource_id: "name of resource in template".to_owned(),
        physical_resource_id: None,
        resource_properties,
    };
    let expected = CustomResourceEvent::Create(exepected_payload);
    assert_eq!(expected, actual);
}
