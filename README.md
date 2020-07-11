# Rust cfn-custom-resource

A Rust create to facilate the creation of Rust Lambda Powered Custom Resources
for AWS Cloudformation. It does not cast an opinion on which aws lambda custom
runtime that the function is executing in.

```rust
use cfn_custom_resource::CustomResourceEvent;
use serde::Deserialize;
#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct MyParameters {
    value_one: i64,
    value_two: i64,
}
async fn my_handler_func(event: CustomResourceEvent<MyParameters>) {
    match event {
        CustomResourceEvent::Create(data) => {
            println!(
                "{}",
                data.resource_properties.value_one + data.resource_properties.value_two
            );
            data.respond_with_success("all done")
                .finish()
                .await
                .unwrap();
        }
        CustomResourceEvent::Update(data) => {
            println!("got an update");
            data.respond_with_success("all done")
                .finish()
                .await
                .unwrap();
        }
        CustomResourceEvent::Delete(data) => {
            println!("got a delete");
            data.respond_with_success("all done")
                .finish()
                .await
                .unwrap();
        }
    }
}
```
