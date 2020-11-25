use cerk::runtime::{InternalServerFnRef, ScheduleFnRefStatic};
use std::collections::HashMap;

pub struct ComponentStartLinks<'a> {
    pub schedulers: HashMap<String, ScheduleFnRefStatic>,
    pub routers: HashMap<String, InternalServerFnRef<'a>>,
    pub config_loaders: HashMap<String, InternalServerFnRef<'a>>,
    pub ports: HashMap<String, InternalServerFnRef<'a>>,
}

pub mod helpers {
    #[macro_export]
    macro_rules! fn_to_link {
        ($f:ident) => {{
            (stringify!($f).to_string(), $f)
        }};
    }

    #[macro_export]
    macro_rules! fn_to_links {
        ( $($f:ident), *) => ({
            [
            $(
                fn_to_link![$f],
            )*
            ].iter().cloned().collect()
        })
    }

    #[cfg(test)]
    mod tests {
        use super::super::*;

        use cerk::kernel::{KernelFn, StartOptions};
        use cerk::runtime::ScheduleFn;

        #[test]
        fn fn_to_link_test() {
            fn dummy_scheduler(_: StartOptions, _: KernelFn) {}
            const DUMMY: ScheduleFnRefStatic = &(dummy_scheduler as ScheduleFn);
            let schedulers: HashMap<String, ScheduleFnRefStatic> =
                [("DUMMY".to_string(), DUMMY)].iter().cloned().collect();
            let schedulers_macro: HashMap<String, ScheduleFnRefStatic> =
                [fn_to_link!(DUMMY)].iter().cloned().collect();
            assert_eq!(schedulers, schedulers_macro)
        }

        #[test]
        fn fn_to_links_test() {
            fn dummy_scheduler(_: StartOptions, _: KernelFn) {}
            const DUMMY: ScheduleFnRefStatic = &(dummy_scheduler as ScheduleFn);
            let schedulers: HashMap<String, ScheduleFnRefStatic> =
                [("DUMMY".to_string(), DUMMY)].iter().cloned().collect();
            let schedulers_macro: HashMap<String, ScheduleFnRefStatic> = fn_to_links![DUMMY];
            assert_eq!(schedulers, schedulers_macro)
        }

        #[test]
        fn fn_to_links_multiple_test() {
            fn dummy_scheduler(_: StartOptions, _: KernelFn) {}
            const DUMMY: ScheduleFnRefStatic = &(dummy_scheduler as ScheduleFn);
            const DUMMY2: ScheduleFnRefStatic = &(dummy_scheduler as ScheduleFn);
            let schedulers: HashMap<String, ScheduleFnRefStatic> =
                [("DUMMY".to_string(), DUMMY), ("DUMMY2".to_string(), DUMMY2)]
                    .iter()
                    .cloned()
                    .collect();
            let schedulers_macro: HashMap<String, ScheduleFnRefStatic> =
                fn_to_links![DUMMY, DUMMY2];
            assert_eq!(schedulers, schedulers_macro)
        }
    }
}
