use enginelib::api::EngineAPI;
use enginelib::api::{deserialize, serialize};
use enginelib::event::info;
use enginelib::event::Event;
use enginelib::event::EventCTX;
use enginelib::event::EventHandler;
use enginelib::events;
use enginelib::plugin::LibraryMetadata;
use enginelib::task::Task;
use enginelib::BuildEventHandler;
use enginelib::Identifier;
use enginelib::Registry;
use serde::Deserialize;
use serde::Serialize;
use std::fmt::Debug;
use std::sync::Arc;
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
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

#[no_mangle]
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
#[no_mangle]
pub fn run(api: &mut EngineAPI) {
    EngineAPI::setup_logger();
    let mod_id = "engine_core".to_string();
    let task_id = "fib".to_string();

    let meta: LibraryMetadata = metadata();
    let mod_ctx = Arc::new(meta.clone());
    BuildEventHandler!(
        OnStartEventHandler,
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
        OnStartEventHandler { mod_ctx },
        ("core".to_string(), "start_event".to_string()),
    );
}
