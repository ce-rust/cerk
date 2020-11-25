use crate::start_links::ComponentStartLinks;
use anyhow::{Context, Result};
use cerk::kernel::{ScheduleInternalServer, ScheduleInternalServerStatic, StartOptions};
use serde::Deserialize;
use std::collections::HashMap;

#[derive(Deserialize, Debug, PartialEq)]
struct Configuration {
    scheduler: String,
    router: String,
    config_loader: String,
    /// key: port name; value: port id
    ports: HashMap<String, String>,
}

fn parse_json_to_config(content: String) -> Result<Configuration> {
    let config = serde_json::from_str(content.as_str())?;
    Ok(config)
}

fn get_link<'a, T>(name: &String, links: &'a HashMap<String, T>) -> Result<&'a T> {
    links
        .get(name.as_str())
        .with_context(|| format!("was not able to find {}", name))
}

fn parse_config_to_start_options(
    links: &ComponentStartLinks<'static>,
    config: &Configuration,
) -> Result<StartOptions> {
    let ports: Vec<ScheduleInternalServerStatic> = config
        .ports
        .iter()
        .map(|(id, name)| ScheduleInternalServer {
            id: id.to_string(),
            function: get_link(name, &links.ports).unwrap(),
        })
        .collect();

    let config = StartOptions {
        scheduler: get_link(&config.scheduler, &links.schedulers)?,
        config_loader: get_link(&config.config_loader, &links.config_loaders)?,
        router: get_link(&config.router, &links.routers)?,
        ports,
    };

    Ok(config)
}

pub fn parse_json_to_start_options<'a>(
    config_content: String,
    links: ComponentStartLinks<'static>,
) -> Result<StartOptions> {
    let config = parse_json_to_config(config_content)?;
    parse_config_to_start_options(&links, &config)
}

#[cfg(test)]
mod tests {
    use super::*;
    use cerk::kernel::KernelFn;
    use cerk::runtime::channel::{BoxedReceiver, BoxedSender};
    use cerk::runtime::{InternalServerFn, InternalServerId, ScheduleFn};

    #[test]
    fn parse_json_to_config_test() -> Result<()> {
        let json = r#"
        {
            "scheduler": "myschedulertype",
            "router": "myroutertype",
            "config_loader": "myconfig_loadertype",
            "ports": {
                "myport": "myporttype"
            }
        }
        "#;
        let config = parse_json_to_config(json.to_string())?;
        assert_eq!(config.scheduler, "myschedulertype");
        assert_eq!(config.router, "myroutertype");
        assert_eq!(config.config_loader, "myconfig_loadertype");
        assert_eq!(config.ports.len(), 1);
        assert_eq!(config.ports.get("myport"), Some(&"myporttype".to_string()));

        Ok(())
    }

    fn dummy_scheduler(_: StartOptions, _: KernelFn) {}

    fn dummy_router(_: InternalServerId, _: BoxedReceiver, _: BoxedSender) {}

    fn dummy_config_loader(_: InternalServerId, _: BoxedReceiver, _: BoxedSender) {}

    fn dummy_port(_: InternalServerId, _: BoxedReceiver, _: BoxedSender) {}

    fn dummy_port_other(_: InternalServerId, _: BoxedReceiver, _: BoxedSender) {}

    #[test]
    fn parse_config_to_start_options_test() -> Result<()> {
        let config = Configuration {
            scheduler: "myschedulertype".to_string(),
            router: "myroutertype".to_string(),
            config_loader: "myconfig_loadertype".to_string(),
            ports: [("myport".to_string(), "myporttype".to_string())]
                .iter()
                .cloned()
                .collect(),
        };

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
            ports: [
                ("myporttype".to_string(), &(dummy_port as InternalServerFn)),
                (
                    "myporttypeother".to_string(),
                    &(dummy_port_other as InternalServerFn),
                ),
            ]
            .iter()
            .cloned()
            .collect(),
        };

        let start_options = parse_config_to_start_options(&link, &config)?;
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
