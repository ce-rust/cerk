use chrono::naive::NaiveDateTime;

#[derive(Clone, Debug)]
pub enum Data {
    String(String),
    None,
}

#[derive(Clone, Debug)]
pub struct CloudEvent {
    pub id: String,
    pub event_type: String,
    pub spec_version: String,
    pub time: Option<NaiveDateTime>,
    pub source: String,
    pub subject: String,
    pub data_schema: String,
    pub content_type: String,
    pub data: Data,
}
