# CERK CloudEvents

## CloudEvent

The `CloudEvent` struct reflects the event envelope defined by the [CNCF CloudEvents 1.0 specification](https://github.com/cloudevents/spec/blob/v1.0/spec.md).
It supports version 1.0 of CloudEvents.

| **1.0**             | Property name             | CLR type                              |
| ------------------- | ------------------------- | ------------------------------------- |
| **id**              | `CloudEvent.id`           | `std::string::String`                 |
| **type**            | `CloudEvent.event_type`   | `std::string::String`                 |
| **specversion**     | `CloudEvent.spec_version` | `std::string::String`                 |
| **source**          | `CloudEvent.source`       | `std::string::String`                 |
| **time**            | `CloudEvent.time`         | `std::option::Option<chrono::naive::NaiveDateTime>`|
| **subject**         | `CloudEvent.subject`      | `std::option::Option<std::string::String>`                 |
| **dataschema**      | `CloudEvent.data_schema`  | `std::option::Option<std::string::String>`                 |
| **datacontenttype** | `CloudEvent.data_content_type` | `std::option::Option<std::string::String>`                 |
| **data**            | `CloudEvent.data`         | `cloudevents::Data`                  |

## References

This Readme is inspried by the [CloudEvents sdk-csharp Readme](https://github.com/cloudevents/sdk-csharp/blob/master/README.md).

* [CNCF CloudEvents 1.0 specification](https://github.com/cloudevents/spec/blob/v1.0/spec.md)
