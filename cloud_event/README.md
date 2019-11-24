# CERK CloudEvents

## CloudEvent

The `CloudEvent` struct reflects the event envelope defined by the [CNCF CloudEvents 1.0 specification](https://github.com/cloudevents/spec/blob/v1.0/spec.md).
It supports version 1.0 of CloudEvents.

| **1.0**             | Property name             | CLR type                              |
| ------------------- | ------------------------- | ------------------------------------- |
| **id**              | `CloudEvent.id`           | `std::string::String`                 |
| **type**            | `CloudEvent.event_type`   | `std::string::String`                 |
| **specversion**     | `CloudEvent.spec_version` | `std::string::String`                 |
| **time**            | `CloudEvent.time`         | `Option<chrono::naive::NaiveDateTime>`|
| **source**          | `CloudEvent.source`       | `std::string::String`                 |
| **subject**         | `CloudEvent.subject`      | `std::string::String`                 |
| **dataschema**      | `CloudEvent.data_schema`  | `std::string::String`                 |
| **datacontenttype** | `CloudEvent.content_type` | `std::string::String`                 |
| **data**            | `CloudEvent.data`         | `cloud_events::Data`                  |

## References

This Readme is inspried by the [CloudEvents sdk-csharp Readme](https://github.com/cloudevents/sdk-csharp/blob/master/README.md).

* [CNCF CloudEvents 1.0 specification](https://github.com/cloudevents/spec/blob/v1.0/spec.md)
