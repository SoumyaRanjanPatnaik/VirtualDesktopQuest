use std::{collections::HashMap, error::Error};
use wayland_client::{
    globals::Global,
    protocol::{wl_output, wl_registry},
    Connection, Dispatch, QueueHandle,
};
use wayland_protocols_wlr::export_dmabuf::v1::client::{
    zwlr_export_dmabuf_frame_v1::{self, ZwlrExportDmabufFrameV1},
    zwlr_export_dmabuf_manager_v1::{self, ZwlrExportDmabufManagerV1},
};
// use wayland_protocols_wlr::export_dmabuf::v1::client::zwlr_export_dmabuf_manager_v1;

const EXPORT_DMABUF_INTERFACE_NAME: &str = "zwlr_export_dmabuf_manager_v1";
const OUTPUT_INTERFACE_NAME: &str = "wl_output";

#[derive(Debug)]
struct AppData {
    globals_list: HashMap<u32, Global>,
}

impl AppData {
    pub fn get_global_by_interface<'a, 'b>(&self, interface: &str) -> Option<Global> {
        let global = self
            .globals_list
            .iter()
            .find(|global| global.1.interface == interface)?
            .1;
        Some(global.clone())
    }
}

impl Dispatch<wl_registry::WlRegistry, ()> for AppData {
    fn event(
        state: &mut Self,
        _: &wl_registry::WlRegistry,
        event: wl_registry::Event,
        _: &(),
        _: &Connection,
        _qh: &QueueHandle<AppData>,
    ) {
        match event {
            wl_registry::Event::Global {
                name,
                interface,
                version,
            } => {
                state.globals_list.insert(
                    name,
                    Global {
                        name,
                        interface,
                        version,
                    },
                );
            }
            wl_registry::Event::GlobalRemove { name } => {
                state.globals_list.remove(&name);
            }
            _ => panic!("unknown event recieved when binding handling dispatch"),
        }
    }
}

impl Dispatch<wl_output::WlOutput, ()> for AppData {
    fn event(
        _state: &mut Self,
        _: &wl_output::WlOutput,
        _event: wl_output::Event,
        _: &(),
        _: &Connection,
        _qh: &QueueHandle<AppData>,
    ) {
    }
}
impl Dispatch<ZwlrExportDmabufManagerV1, ()> for AppData {
    fn event(
        _state: &mut Self,
        _: &ZwlrExportDmabufManagerV1,
        _event: zwlr_export_dmabuf_manager_v1::Event,
        _: &(),
        _: &Connection,
        _qh: &QueueHandle<AppData>,
    ) {
    }
}
impl Dispatch<ZwlrExportDmabufFrameV1, ()> for AppData {
    fn event(
        _state: &mut Self,
        _: &ZwlrExportDmabufFrameV1,
        event: zwlr_export_dmabuf_frame_v1::Event,
        _: &(),
        _: &Connection,
        _qh: &QueueHandle<AppData>,
    ) {
        println!("{:#?}", event);
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let conn = Connection::connect_to_env()?;
    let display = conn.display();
    let mut event_queue = conn.new_event_queue();
    let qh = event_queue.handle();
    let registry = display.get_registry(&qh, ());
    let mut state = AppData {
        globals_list: HashMap::new(),
    };

    event_queue.roundtrip(&mut state)?;
    let output_global = state
        .get_global_by_interface(OUTPUT_INTERFACE_NAME)
        .unwrap();
    let export_dmabuf_global = state
        .get_global_by_interface(EXPORT_DMABUF_INTERFACE_NAME)
        .unwrap();

    let output: wl_output::WlOutput =
        registry.bind(output_global.name, output_global.version, &qh, ());

    event_queue.roundtrip(&mut state)?;
    let export_dmabuf_mgr: ZwlrExportDmabufManagerV1 = registry.bind(
        export_dmabuf_global.name,
        export_dmabuf_global.version,
        &qh,
        (),
    );

    event_queue.roundtrip(&mut state)?;
    export_dmabuf_mgr.capture_output(1, &output, &qh, ());

    loop {
        event_queue.blocking_dispatch(&mut state)?;
    }
}
