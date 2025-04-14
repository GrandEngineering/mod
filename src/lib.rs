use enginelib::Identifier;
use enginelib::RegisterCgrpcEventHandler;
use enginelib::RegisterEventHandler;
use enginelib::Registry;
use enginelib::api::EngineAPI;
use enginelib::api::deserialize;
use enginelib::event::Event;
use enginelib::event::EventCTX;
use enginelib::event::EventHandler;
use enginelib::event::info;
use enginelib::events;
use enginelib::events::ID;
use enginelib::events::cgrpc_event::CgrpcEvent;
use enginelib::plugin::LibraryMetadata;
use enginelib::prelude::macros::Verifiable;
use enginelib::prelude::macros::metadata;
use enginelib::prelude::macros::module;
use enginelib::register_event;
use enginelib::task::Task;
use enginelib::task::Verifiable;
use serde::Deserialize;
use serde::Serialize;
use std::any::Any;
use std::fmt::Debug;
#[derive(Debug, Clone, Default, Serialize, Deserialize, Verifiable)]
pub struct FibTask {
    pub iter: u64,
    pub result: u64,
}
impl Task for FibTask {
    fn get_id(&self) -> Identifier {
        ("engine_core".to_string(), "fib".to_string())
    }
    fn clone_box(&self) -> Box<dyn Task> {
        Box::new(self.clone())
    }
    fn run_cpu(&mut self) {
        let mut a = 0;
        let mut b = 1;
        for _ in 0..self.iter {
            let tmp = a;
            a = b;
            b += tmp;
        }
        self.result = a;
    }
    fn from_bytes(&self, bytes: &[u8]) -> Box<dyn Task> {
        let task: FibTask = deserialize(bytes).unwrap();
        Box::new(task)
    }
    fn to_bytes(&self) -> Vec<u8> {
        enginelib::api::serialize(self).unwrap()
    }
}

#[metadata]
pub fn metadata() -> LibraryMetadata {
    let meta: LibraryMetadata = LibraryMetadata {
        mod_id: "engine_core".to_owned(),
        mod_author: "@ign-styly".to_string(),
        mod_name: "Engine Core External".to_string(),
        mod_version: "0.0.1".to_string(),
        ..Default::default()
    };
    meta
}
#[derive(Clone, Debug)]
struct CustomEvent {
    pub id: Identifier,
    pub cancelled: bool,
}
impl Event for CustomEvent {
    fn clone_box(&self) -> Box<dyn Event> {
        Box::new(self.clone())
    }

    fn cancel(&mut self) {
        self.cancelled = true;
    }
    fn is_cancelled(&self) -> bool {
        self.cancelled
    }
    fn get_id(&self) -> Identifier {
        self.id.clone()
    }
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

#[module]
pub fn run(api: &mut EngineAPI) {
    EngineAPI::setup_logger();
    let mod_id = "engine_core".to_string();
    let task_id = "fib".to_string();
    let meta: LibraryMetadata = metadata();
    let mod_ctx = Arc::new(meta.clone());
    register_event!(
        api,
        engine_core,
        custom_event,
        CustomEvent {
            cancelled: false,
            id: ID("engine_core", "custom_event")
        }
    );
    RegisterCgrpcEventHandler!(cgrpcHandler, engine_core, grpc, |event: &mut CgrpcEvent| {
        event.output.write().unwrap().append(event.payload.as_mut());
        println!("grpc event!");
    });
    RegisterEventHandler!(
        CustomEventHandler,
        CustomEvent,
        |event: &mut CustomEvent| {
            info!("Custom Event",);
        }
    );

    RegisterEventHandler!(
        StartEventHandler,
        events::start_event::StartEvent,
        LibraryMetadata,
        |event: &mut events::start_event::StartEvent, mod_ctx: &Arc<LibraryMetadata>| {
            for n in event.modules.clone() {
                info!("Module: {:?}", n);
            }
            info!(
                "Event {:?} Handled by: {:?}, made by {}",
                event.id, &mod_ctx.mod_name, &mod_ctx.mod_author,
            );
        }
    );
    let tsk_ref = Arc::new(FibTask::default());
    api.task_registry
        .register(tsk_ref, (mod_id.clone(), task_id.clone()));
    api.event_bus.event_handler_registry.register_handler(
        StartEventHandler { mod_ctx },
        ("core".to_string(), "start_event".to_string()),
    );
    api.event_bus.event_handler_registry.register_handler(
        cgrpcHandler {},
        ("core".to_string(), "cgrpc_event".to_string()),
    );
    api.event_bus.event_handler_registry.register_handler(
        CustomEventHandler {},
        ("engine_core".to_string(), "custom_event".to_string()),
    );
    api.event_bus.handle(
        ("engine_core".into(), "custom_event".into()),
        &mut CustomEvent {
            cancelled: false,
            id: ID("engine_core", "custom_event"),
        },
    );
}
