//! A Rust create to facilitate the creation of Rust Lambda Powered Custom Resources
//! for AWS Cloudformation. It does not cast an opinion on which aws lambda custom
//! runtime that the function is executing in.
//!
//!
//! A simple example is below:
//!
//! ```
//! use cfn_custom_resource::CustomResourceEvent;
//! use serde::Deserialize;
//!
//! #[derive(Debug, Deserialize)]
//! #[serde(rename_all = "PascalCase")]
//! struct MyParameters {
//!     value_one: i64,
//!     value_two: i64,
//! }
//!
//! async fn my_handler_func(event: CustomResourceEvent<MyParameters>) {
//!     match event {
//!         CustomResourceEvent::Create(data) => {
//!             println!(
//!                 "{}",
//!                 data.resource_properties.value_one + data.resource_properties.value_two
//!             );
//!             data.respond_with_success("all done")
//!                 .finish()
//!                 .await
//!                 .unwrap();
//!         }
//!         CustomResourceEvent::Update(data) => {
//!             println!("got an update");
//!             data.respond_with_success("all done")
//!                 .finish()
//!                 .await
//!                 .unwrap();
//!         }
//!         CustomResourceEvent::Delete(data) => {
//!             println!("got a delete");
//!             data.respond_with_success("all done")
//!                 .finish()
//!                 .await
//!                 .unwrap();
//!         }
//!     }
//! }
//! ```

#![deny(clippy::all, clippy::pedantic, clippy::nursery)]
#![deny(
    missing_docs,
    missing_debug_implementations,
    missing_copy_implementations,
    trivial_casts,
    trivial_numeric_casts,
    unsafe_code,
    unstable_features,
    unused_import_braces,
    unused_qualifications
)]
#![allow(clippy::must_use_candidate, clippy::used_underscore_binding)]

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use std::collections::HashMap;

/// Defines the data that encapsualtes the payload sent by cloudformation to a lambda function.
/// This includes all data in the payload except the inner event type. This structure is
/// encapsulated by the [`CustomResourceEvent`](enum.CustomResourceEvent.html) that
/// defines that event type as well.
#[derive(Debug, Deserialize, PartialEq)]
#[serde(rename_all = "PascalCase")]
pub struct CloudformationPayload<T> {
    /// A unique ID for the request.
    ///
    /// Combining the StackId with the RequestId forms a value that you can use to
    /// uniquely identify a request on a particular custom resource.
    pub request_id: String,

    /// The response URL identifies a presigned S3 bucket that receives responses from the
    /// custom resource provider to AWS CloudFormation.
    #[serde(rename = "ResponseURL")]
    pub response_url: String,

    /// The template developer-chosen resource type of the custom resource in the AWS CloudFormation template.
    /// Custom resource type names can be up to 60 characters long and can
    /// include alphanumeric and the following characters: _@-.
    pub resource_type: String,

    /// The template developer-chosen name (logical ID) of the custom resource in the AWS CloudFormation template.
    /// This is provided to facilitate communication between the custom resource provider and the template developer.
    pub logical_resource_id: String,

    /// The Amazon Resource Name (ARN) that identifies the stack that contains the custom resource.
    ///
    /// Combining the StackId with the RequestId forms a value that you can use to uniquely
    /// identify a request on a particular custom resource.
    pub stack_id: String,

    /// A required custom resource provider-defined physical ID that is unique for that provider.
    /// Always sent with Update and Delete requests; never sent with Create.
    pub physical_resource_id: Option<Uuid>,

    /// This field contains the contents of the Properties object sent by the template developer.
    /// Its contents are defined by the custom resource provider (you) and therefore is generic to accomedate
    /// any data set that you might have.
    pub resource_properties: T,
}

impl<T> CloudformationPayload<T> {
    /// Creates a response that indicates a failure.
    pub fn respond_with_success(self, reason: &str) -> CustomResourceResponse {
        CustomResourceResponse {
            status: ResponseType::Success,
            reason: reason.to_owned(),
            physical_resource_id: self.physical_resource_id.unwrap_or_else(Uuid::new_v4),
            stack_id: self.stack_id,
            request_id: self.request_id,
            logical_resource_id: self.logical_resource_id,
            response_url: self.response_url,
            no_echo: None,
            data: None,
        }
    }

    /// Creates a response that indicates a failure.
    pub fn respond_with_failure(self, reason: &str) -> CustomResourceResponse {
        CustomResourceResponse {
            status: ResponseType::Failed,
            reason: reason.to_owned(),
            physical_resource_id: self.physical_resource_id.unwrap_or_else(Uuid::new_v4),
            stack_id: self.stack_id,
            request_id: self.request_id,
            logical_resource_id: self.logical_resource_id,
            response_url: self.response_url,
            no_echo: None,
            data: None,
        }
    }

    /// Creates a response based on the request in question as well as the result of
    /// some operation executed prior this. If the result is Ok(_), then the response
    /// will be a success. If the response is Err(e), the response will be a failure
    /// where the error message is the error string.
    pub fn respond_with<D, E>(self, result: Result<D, E>) -> CustomResourceResponse
    where
        E: std::error::Error,
    {
        match result {
            Ok(_) => self.respond_with_success("Success"),
            Err(e) => self.respond_with_failure(&format!("{:?}", e)),
        }
    }
}

/// Defines an event payload that is sent to a lambda function by
/// AWS Cloud Formation. This payload is defined by AWS as the following:
///
/// ```json
/// {
///   "RequestType" : "Delete",
///   "RequestId" : "unique id for this delete request",
///   "ResponseURL" : "pre-signed-url-for-delete-response",
///   "ResourceType" : "Custom::MyCustomResourceType",
///   "LogicalResourceId" : "name of resource in template",
///   "StackId" : "arn:aws:cloudformation:us-east-2:namespace:stack/stack-name/guid",
///   "PhysicalResourceId" : "custom resource provider-defined physical id",
///   "ResourceProperties" : {
///      "key1" : "string",
///      "key2" : [ "list" ],
///      "key3" : { "key4" : "map" }
///   }
///}
///```
///
/// The entire payload is the same for each custom resource except for the `ResourceProperties` section, so
/// that is available as a generic type for which we are able to create the schema for the resource properties
/// on your own.
#[derive(Debug, Deserialize, PartialEq)]
#[serde(tag = "RequestType")]
pub enum CustomResourceEvent<T> {
    /// Defines a cloudformation payload that happens on a create of the resource.
    Create(CloudformationPayload<T>),
    /// Defines a cloudforamtion payload that happens on ab update to the resource.
    Update(CloudformationPayload<T>),
    /// Defines a cloudforamtion payload that happens on the deletion the resource.
    Delete(CloudformationPayload<T>),
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "UPPERCASE")]
enum ResponseType {
    Success,
    Failed,
}

/// Define a response payload for the execution of a cloud formation custom resource data that is
/// sent back to cloud formation after the process of executing the custom resource is complete.
#[derive(Debug, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct CustomResourceResponse {
    status: ResponseType,
    reason: String,
    physical_resource_id: Uuid,
    stack_id: String,
    request_id: String,
    logical_resource_id: String,
    no_echo: Option<bool>,
    data: Option<HashMap<String, String>>,
    #[serde(skip)]
    response_url: String,
}

impl CustomResourceResponse {
    /// Indicates whether to mask the output of the custom resource when retrieved by using the `Fn::GetAtt` function.
    /// If set to true, all returned values are masked with asterisks (*****), except for those stored in the Metadata
    /// section of the template. Cloud Formation does not transform, modify, or redact any information you include in
    /// the Metadata section. The default value is false.
    pub fn set_no_echo(mut self, value: bool) -> Self {
        self.no_echo = Some(value);
        self
    }

    /// Writes a new key value pair to the data section of the response.
    ///
    /// The custom resource provider-defined name-value pairs to send with the response.
    /// You can access the values provided here by name in the template with `Fn::GetAtt`.
    pub fn add_data(mut self, key: &str, value: &str) -> Self {
        if self.data.is_none() {
            self.data = Some(HashMap::new());
        }

        self.data
            .as_mut()
            .unwrap()
            .insert(key.to_owned(), value.to_owned());
        self
    }

    /// Sends the response object (self) to the response url of the custom resource.
    ///
    /// This finishes the cloudformation object change (create/update/delete).
    ///
    /// # Errors
    /// When the request fails to execute the request
    pub async fn finish(self) -> Result<(), reqwest::Error> {
        let client = reqwest::Client::new();
        client.post(&self.response_url).json(&self).send().await?;
        Ok(())
    }
}
