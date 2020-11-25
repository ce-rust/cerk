use crate::config_parser::parse_json_to_start_options;
use crate::file_reader::read_file;
use crate::start_links::ComponentStartLinks;
use anyhow::Result;
use cerk::kernel::{bootstrap, StartOptions};
use std::env;

/// Starts cerk with a ComponentStartLinks set and a init config provided in the given path.
pub fn load_by_path<'a>(path: String, links: ComponentStartLinks<'static>) -> Result<StartOptions> {
    info!("loading loader config from {}", path);
    let content = read_file(path.as_str())?;
    parse_json_to_start_options(content, links)
}

fn load<'a>(links: ComponentStartLinks<'static>) -> Result<StartOptions> {
    let path = env::var("CONFIG_PATH").unwrap_or(String::from("./init.json"));
    load_by_path(path, links)
}

/// Starts cerk with a ComponentStartLinks set and a init config provided in path `$CONFIG_PATH` (fallback `./init.json`).
pub fn start(links: ComponentStartLinks<'static>) {
    match load(links) {
        Ok(c) => bootstrap(c),
        Err(e) => panic!("failed to load config {:?}", e),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use cerk::kernel::KernelFn;
    use cerk::runtime::channel::{BoxedReceiver, BoxedSender};
    use cerk::runtime::{InternalServerFn, InternalServerId, ScheduleFn};

    fn dummy_scheduler(_: StartOptions, _: KernelFn) {}

    fn dummy_router(_: InternalServerId, _: BoxedReceiver, _: BoxedSender) {}

    fn dummy_config_loader(_: InternalServerId, _: BoxedReceiver, _: BoxedSender) {}

    fn dummy_port(_: InternalServerId, _: BoxedReceiver, _: BoxedSender) {}

    #[test]
    fn load_by_path_test() -> Result<()> {
        let link = ComponentStartLinks {
            schedulers: [(
                "myschedulertype".to_string(),
                &(dummy_scheduler as ScheduleFn),
            )]
            .iter()
            .cloned()
            .collect(),
            routers: [(
                "myroutertype".to_string(),
                &(dummy_router as InternalServerFn),
            )]
            .iter()
            .cloned()
            .collect(),
            config_loaders: [(
                "myconfig_loadertype".to_string(),
                &(dummy_config_loader as InternalServerFn),
            )]
            .iter()
            .cloned()
            .collect(),
            ports: [("myporttype".to_string(), &(dummy_port as InternalServerFn))]
                .iter()
                .cloned()
                .collect(),
        };

        let start_options = load_by_path("./testdata/init.json".to_string(), link)?;
        assert_eq!(start_options.scheduler, &(dummy_scheduler as ScheduleFn));
        assert_eq!(start_options.router, &(dummy_router as InternalServerFn));
        assert_eq!(
            start_options.config_loader,
            &(dummy_config_loader as InternalServerFn)
        );
        assert_eq!(start_options.ports.len(), 1);
        assert_eq!(start_options.ports[0].id, "myport");
        assert_eq!(
            start_options.ports[0].function,
            &(dummy_port as InternalServerFn)
        );

        Ok(())
    }
}
