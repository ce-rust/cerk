pub mod kernel;
pub mod ports;
pub mod routing;
pub mod runtime;

#[macro_use]
extern crate log;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
