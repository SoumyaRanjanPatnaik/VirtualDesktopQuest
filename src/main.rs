mod shm;
use shm::ShmFactoryBuilder;
use std::{
    collections::VecDeque,
    error::Error,
    fs::{File, Permissions},
    io::{Read, Write},
    mem::take,
    os::unix::prelude::PermissionsExt,
    sync::mpsc::{self, Receiver, Sender},
    thread,
};
use wayland_client::{
    globals::Global,
    protocol::{
        __interfaces::{WL_OUTPUT_INTERFACE, WL_SHM_INTERFACE},
        wl_output, wl_registry, wl_shm, wl_shm_pool,
    },
    Connection, Dispatch, DispatchError, EventQueue, QueueHandle,
};
use wayland_protocols_wlr::{
    export_dmabuf::v1::client::zwlr_export_dmabuf_frame_v1::{self, ZwlrExportDmabufFrameV1},
    screencopy::v1::client::{
        __interfaces::ZWLR_SCREENCOPY_MANAGER_V1_INTERFACE,
        zwlr_screencopy_frame_v1::ZwlrScreencopyFrameV1,
        zwlr_screencopy_manager_v1::ZwlrScreencopyManagerV1,
    },
};

type FrameEventVec = VecDeque<zwlr_export_dmabuf_frame_v1::Event>;

#[derive(Debug, Default)]
struct Capturer {
    frame: FrameEventVec,
    objects: FrameEventVec,
    ready: Option<zwlr_export_dmabuf_frame_v1::Event>,
}

impl Capturer {
    pub fn insert(&mut self, event: zwlr_export_dmabuf_frame_v1::Event) {
        use zwlr_export_dmabuf_frame_v1::Event as FrameEvent;
        match event {
            FrameEvent::Frame { .. } => self.frame.push_back(event),
            FrameEvent::Object { .. } => {
                self.objects.push_back(event);
            }
            FrameEvent::Ready { .. } => {
                self.ready = Some(event);
            }
            FrameEvent::Cancel { .. } => {
                self.frame.clear();
                self.objects.clear();
            }
            _ => {}
        }
    }
    fn do_capture(
        mut self,
        rx: Receiver<zwlr_export_dmabuf_frame_v1::Event>,
    ) -> Result<(), Box<dyn Error>> {
        use zwlr_export_dmabuf_frame_v1::Event;
        let mut f = File::create("./capture.out")?;
        let mut buf = Box::new([0u8; 1920 * 1080 * 10]);
        while self.objects.len() > 0 {
            if let Some(Event::Object { fd, .. }) = self.objects.pop_back() {
                let mut reader: File = From::from(fd);
                let reader_perms = Permissions::from_mode(7u32);
                reader.set_permissions(reader_perms)?;

                loop {
                    let event_recieved = rx.try_recv();
                    if let Ok(event) = event_recieved {
                        if let Event::Cancel { reason } = event {
                            dbg!("Capture Canceled: {}", reason);
                            break;
                        }
                    }
                    reader.read(&mut *buf)?;
                    f.write_all(&*buf)?;
                }
            }
        }
        Ok(())
    }
    pub fn start_capture(self) -> Sender<zwlr_export_dmabuf_frame_v1::Event> {
        let (tx, rx) = mpsc::channel();
        let _ = thread::Builder::new()
            .stack_size(1024 * 1024 * 100)
            .spawn(|| self.do_capture(rx).expect("Failed to write"));
        return tx;
    }
}

#[derive(Debug, Default)]
struct AppData {
    globals_list: Vec<Global>,
    capturer: Option<Box<Capturer>>,
}

impl AppData {
    pub fn get_global_by_interface<'a, 'b>(&self, interface: &str) -> Vec<Global> {
        self.globals_list
            .iter()
            .filter(|global| global.interface == interface)
            .map(|gobal_ref| gobal_ref.clone())
            .collect()
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
                state.globals_list.push(Global {
                    name,
                    interface,
                    version,
                });
            }
            wl_registry::Event::GlobalRemove { name } => {
                let Some(index) = state
                    .globals_list
                    .iter()
                    .position(|global| global.name == name) else {
                    return;
                };
                state.globals_list.swap_remove(index);
            }
            _ => panic!("unknown event recieved when binding handling dispatch"),
        }
    }
}

impl Dispatch<wl_output::WlOutput, ()> for AppData {
    fn event(
        _state: &mut Self,
        _: &wl_output::WlOutput,
        event: wl_output::Event,
        _: &(),
        _: &Connection,
        _qh: &QueueHandle<AppData>,
    ) {
        match event {
            wl_output::Event::Name { name } => println!("{name}"),
            _ => (),
        }
    }
}

impl Dispatch<wl_shm::WlShm, ()> for AppData {
    fn event(
        _state: &mut Self,
        _proxy: &wl_shm::WlShm,
        _event: <wl_shm::WlShm as wayland_client::Proxy>::Event,
        _data: &(),
        _conn: &Connection,
        _qhandle: &QueueHandle<Self>,
    ) {
    }
}
impl Dispatch<ZwlrScreencopyManagerV1, ()> for AppData {
    fn event(
        _state: &mut Self,
        _proxy: &ZwlrScreencopyManagerV1,
        _event: <ZwlrScreencopyManagerV1 as wayland_client::Proxy>::Event,
        _data: &(),
        _conn: &Connection,
        _qhandle: &QueueHandle<Self>,
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
        (|| -> Option<Sender<zwlr_export_dmabuf_frame_v1::Event>> {
            println!("{:#?}", event);
            if let None = state.capturer {
                state.capturer = Some(Box::new(Capturer::default()))
            }
            let capture_data = state.capturer.as_mut().unwrap();
            let is_ready_event = matches!(event, zwlr_export_dmabuf_frame_v1::Event::Ready { .. });
            let _ = capture_data.insert(event);
            if is_ready_event {
                let tx = take(&mut state.capturer)?.start_capture();
                Some(tx)
            } else {
                None
            }
        })();
    }
}
impl Dispatch<ZwlrScreencopyFrameV1, ()> for AppData {
    fn event(
        _state: &mut Self,
        _proxy: &ZwlrScreencopyFrameV1,
        _event: <ZwlrScreencopyFrameV1 as wayland_client::Proxy>::Event,
        _data: &(),
        _conn: &Connection,
        _qhandle: &QueueHandle<Self>,
    ) {
        // proxy.copy_with_damage()
    }
}

impl Dispatch<wl_shm_pool::WlShmPool, ()> for AppData {
    fn event(
        _state: &mut Self,
        _proxy: &wl_shm_pool::WlShmPool,
        _event: <wl_shm_pool::WlShmPool as wayland_client::Proxy>::Event,
        _data: &(),
        _conn: &Connection,
        _qhandle: &QueueHandle<Self>,
    ) {
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

    let output_globals = app_state.get_global_by_interface(WL_OUTPUT_INTERFACE.name);
    let screencopy_globals =
        app_state.get_global_by_interface(ZWLR_SCREENCOPY_MANAGER_V1_INTERFACE.name);
    let screencopy_obj = screencopy_globals
        .get(0)
        .ok_or("protocol unavailable: screencopy")?;

    let shm_globals = app_state.get_global_by_interface(WL_SHM_INTERFACE.name);
    let shm_obj = shm_globals.get(0).ok_or("protocol unavailable: wl_shm")?;

    let mut outputs = vec![];
    for output_obj in output_globals {
        let output: wl_output::WlOutput = with_roundtrip(&mut event_queue, &mut app_state, || {
            registry.bind(output_obj.name, output_obj.version, &qh, ())
        })?;
        outputs.push(output);
    }
    let screencopy_mgr: ZwlrScreencopyManagerV1 =
        registry.bind(screencopy_obj.name, screencopy_obj.version, &qh, ());

    let shm_mgr: wl_shm::WlShm = with_roundtrip(&mut event_queue, &mut app_state, || {
        registry.bind(shm_obj.name, shm_obj.version, &qh, ())
    })?;

    let shm_factory = ShmFactoryBuilder::default()
        .name("/wl_scrcpy_vd")
        .size(1920 * 1080)
        .rw(true)
        .create(true)
        .build()?;
    let shm = shm_factory.create_shm_map().ok_or("Failed to create shm")?;
    let _shm_pool = unsafe { shm_mgr.create_pool(shm.fd(), shm.len().try_into()?, &qh, ()) };
    // shm_pool.create_buffer(offset, width, height, stride, format, &qh, udata);

    for output in &outputs {
        let _frame = with_roundtrip(&mut event_queue, &mut app_state, || {
            screencopy_mgr.capture_output(1, output, &qh, ())
        })?;
        // frame.copy_with_damage()
    }

    loop {
        event_queue.blocking_dispatch(&mut app_state)?;
    }
}
