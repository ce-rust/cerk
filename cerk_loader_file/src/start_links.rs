use cerk::runtime::{InternalServerFnRef, ScheduleFnRefStatic};
use std::collections::HashMap;

pub struct ComponentStartLinks<'a> {
    pub schedulers: HashMap<String, ScheduleFnRefStatic>,
    pub routers: HashMap<String, InternalServerFnRef<'a>>,
    pub config_loaders: HashMap<String, InternalServerFnRef<'a>>,
    pub ports: HashMap<String, InternalServerFnRef<'a>>,
}
