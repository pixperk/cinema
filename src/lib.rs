pub mod actor;
pub mod address;
pub mod context;
pub mod message;

pub use actor::{Actor, Handler};
pub use address::Addr;
pub use context::Context;
pub use message::Message;

#[cfg(test)]
mod tests {
    use super::*;

    struct Greet(String);

    impl Message for Greet {
        type Result = String;
    }

    //Actor
    struct Greeter {
        prefix: String,
    }

    impl Actor for Greeter {}

    impl Handler<Greet> for Greeter {
        fn handle(&mut self, msg: Greet, _ctx: &mut Context<Self>) -> String {
            format!("{}, {}!", self.prefix, msg.0)
        }
    }
    #[test]
    fn actor_compiles() {
        let mut greeter = Greeter {
            prefix: "Hello".into(),
        };
        let mut ctx = Context::new();

        let result = greeter.handle(Greet("World".into()), &mut ctx);
        assert_eq!(result, "Hello, World!");
    }
}
