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
    pub source: String,
    pub time: Option<NaiveDateTime>,
    pub subject: Option<String>,
    pub data_schema: Option<String>,
    pub data_content_type: Option<String>,
    pub data: Data,
}
