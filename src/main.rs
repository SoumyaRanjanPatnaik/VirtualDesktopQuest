use std::{collections::HashMap, error::Error};
use wayland_client::{
    globals::Global,
    protocol::{wl_output, wl_registry},
    Connection, Dispatch, DispatchError, EventQueue, QueueHandle,
};
use wayland_protocols_wlr::export_dmabuf::v1::client::{
    zwlr_export_dmabuf_frame_v1::{self, ZwlrExportDmabufFrameV1},
    zwlr_export_dmabuf_manager_v1::{self, ZwlrExportDmabufManagerV1},
};
// use wayland_protocols_wlr::export_dmabuf::v1::client::zwlr_export_dmabuf_manager_v1;

const EXPORT_DMABUF_INTERFACE_NAME: &str = "zwlr_export_dmabuf_manager_v1";
const OUTPUT_INTERFACE_NAME: &str = "wl_output";

type FrameEventVec = Vec<zwlr_export_dmabuf_frame_v1::Event>;

#[derive(Debug, Default)]
struct Capturer {
    frame: FrameEventVec,
    objects: FrameEventVec,
    ready: Option<zwlr_export_dmabuf_frame_v1::Event>,
}

impl Capturer {
    fn insert(&mut self, event: zwlr_export_dmabuf_frame_v1::Event) {
        use zwlr_export_dmabuf_frame_v1::Event as FrameEvent;
        let mut frame_data = Self::default();
        match event {
            FrameEvent::Frame { .. } => frame_data.frame.push(event),
            FrameEvent::Object { .. } => frame_data.objects.push(event),
            FrameEvent::Ready { .. } => {
                frame_data.ready = Some(event);
            }
            FrameEvent::Cancel { .. } => {
                frame_data.frame.clear();
                frame_data.objects.clear();
            }
            _ => {}
        }
    }
    // fn start_capture()
}

#[derive(Debug, Default)]
struct AppData {
    globals_list: HashMap<u32, Global>,
    capturer: Option<Capturer>,
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
        state: &mut Self,
        _: &ZwlrExportDmabufFrameV1,
        event: zwlr_export_dmabuf_frame_v1::Event,
        _: &(),
        _: &Connection,
        _qh: &QueueHandle<AppData>,
    ) {
        println!("{:#?}", event);
        if let Some(capture_data) = &mut state.capturer {
            capture_data.insert(event);
        } else {
        }
    }
}

fn with_roundtrip<T, S>(
    event_queue: &mut EventQueue<S>,
    state: &mut S,
    method: impl Fn() -> T,
) -> Result<T, DispatchError> {
    let result = method();
    event_queue.roundtrip(state)?;
    Ok(result)
}

fn main() -> Result<(), Box<dyn Error>> {
    let conn = Connection::connect_to_env()?;
    let display = conn.display();
    let mut event_queue = conn.new_event_queue();

    let mut app_state = AppData::default();
    let qh = event_queue.handle();

    let registry = with_roundtrip(&mut event_queue, &mut app_state, || {
        display.get_registry(&qh, ())
    })?;

    let output_obj = app_state
        .get_global_by_interface(OUTPUT_INTERFACE_NAME)
        .unwrap();
    let export_dmabuf_obj = app_state
        .get_global_by_interface(EXPORT_DMABUF_INTERFACE_NAME)
        .unwrap();

    let output: wl_output::WlOutput = with_roundtrip(&mut event_queue, &mut app_state, || {
        registry.bind(output_obj.name, output_obj.version, &qh, ())
    })?;

    let export_dmabuf_mgr: ZwlrExportDmabufManagerV1 =
        with_roundtrip(&mut event_queue, &mut app_state, || {
            registry.bind(export_dmabuf_obj.name, export_dmabuf_obj.version, &qh, ())
        })?;

    with_roundtrip(&mut event_queue, &mut app_state, || {
        export_dmabuf_mgr.capture_output(1, &output, &qh, ());
    })?;

    loop {
        event_queue.blocking_dispatch(&mut app_state)?;
    }
}
