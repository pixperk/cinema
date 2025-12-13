use std::{
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    time::Duration,
};

use cinema::{Actor, ActorSystem, Context, Handler, Message};

struct Crash;

impl Message for Crash {
    type Result = ();
}

struct Ping;
impl Message for Ping {
    type Result = ();
}

struct CrashActor {
    stop_called: Arc<AtomicBool>,
}

impl Actor for CrashActor {
    fn stopped(&mut self, _ctx: &mut Context<Self>) {
        self.stop_called.store(true, Ordering::SeqCst);
    }
}

impl Handler<Crash> for CrashActor {
    fn handle(&mut self, _msg: Crash, _ctx: &mut Context<Self>) {
        panic!("Intentional crash!");
    }
}
impl Handler<Ping> for CrashActor {
    fn handle(&mut self, _msg: Ping, _ctx: &mut Context<Self>) {
        // Do nothing
    }
}

#[tokio::test]
async fn actor_panic_stops_gracefully() {
    let stopped_called = Arc::new(AtomicBool::new(false));
    let actor = CrashActor {
        stop_called: stopped_called.clone(),
    };

    let sys = ActorSystem::new();
    let addr = sys.spawn(actor);

    tokio::time::sleep(Duration::from_millis(10)).await;

    addr.do_send(Crash);

    tokio::time::sleep(Duration::from_millis(100)).await;

    assert!(stopped_called.load(Ordering::SeqCst));
}

#[tokio::test]
async fn actor_continues_after_normal_messages() {
    let stopped_called = Arc::new(AtomicBool::new(false));
    let actor = CrashActor {
        stop_called: stopped_called.clone(),
    };

    let sys = ActorSystem::new();
    let addr = sys.spawn(actor);

    addr.do_send(Ping);
    addr.do_send(Ping);
    addr.do_send(Ping);

    tokio::time::sleep(std::time::Duration::from_millis(100)).await;

    assert!(!stopped_called.load(Ordering::SeqCst));

    sys.shutdown();
    tokio::time::sleep(std::time::Duration::from_millis(100)).await;

    assert!(stopped_called.load(Ordering::SeqCst));
}
