use std::{collections::HashMap, error::Error, fs::OpenOptions, slice};

use image::{ImageBuffer, ImageOutputFormat, Rgba};
use smithay_client_toolkit::{
    delegate_output, delegate_registry, delegate_shm,
    globals::GlobalData,
    output::{OutputData, OutputHandler, OutputState},
    reexports::protocols::xdg::xdg_output::zv1::client::{zxdg_output_manager_v1, zxdg_output_v1},
    registry::{ProvidesRegistryState, RegistryState},
    registry_handlers,
    shm::{multi::MultiPool, Shm, ShmHandler},
};
use wayland_client::{
    globals::GlobalList,
    protocol::{
        wl_output::WlOutput,
        wl_shm::{Format, WlShm},
    },
    Connection, Dispatch, QueueHandle,
};

use wayland_protocols_wlr::screencopy::v1::client::{
    zwlr_screencopy_frame_v1::{self, ZwlrScreencopyFrameV1},
    zwlr_screencopy_manager_v1::ZwlrScreencopyManagerV1,
};

#[derive(Debug)]
pub struct CapturerState {
    output_state: OutputState,
    registry_state: RegistryState,
    shm: Shm,
    shm_pool: MultiPool<WlOutput>,
    output_buffers: HashMap<WlOutput, (i32, i32, i32, Format, Option<(*const u8, usize)>)>,
}

impl CapturerState {
    pub fn new<D>(global_list: &GlobalList, qh: &QueueHandle<D>) -> Result<Self, Box<dyn Error>>
    where
        D: Dispatch<WlOutput, OutputData>
            + Dispatch<zxdg_output_v1::ZxdgOutputV1, OutputData>
            + Dispatch<zxdg_output_manager_v1::ZxdgOutputManagerV1, GlobalData>
            + Dispatch<WlShm, GlobalData>
            + ShmHandler
            + 'static,
    {
        let output_state = OutputState::new(&global_list, qh);
        let registry_state = RegistryState::new(&global_list);
        let shm = Shm::bind(&global_list, qh)?;
        let capture_pool = MultiPool::new(&shm)?;
        let output_buffers = HashMap::new();
        Ok(Self {
            output_state,
            registry_state,
            shm,
            shm_pool: capture_pool,
            output_buffers,
        })
    }
}

delegate_registry!(CapturerState);
impl ProvidesRegistryState for CapturerState {
    fn registry(&mut self) -> &mut RegistryState {
        &mut self.registry_state
    }
    registry_handlers!(OutputState);
}

delegate_output!(CapturerState);
impl OutputHandler for CapturerState {
    fn output_destroyed(&mut self, _conn: &Connection, _qh: &QueueHandle<Self>, _output: WlOutput) {
    }
    fn update_output(&mut self, _conn: &Connection, _qh: &QueueHandle<Self>, _output: WlOutput) {}
    fn new_output(&mut self, _conn: &Connection, _qh: &QueueHandle<Self>, _output: WlOutput) {}
    fn output_state(&mut self) -> &mut OutputState {
        &mut self.output_state
    }
}

delegate_shm!(CapturerState);
impl ShmHandler for CapturerState {
    fn shm_state(&mut self) -> &mut Shm {
        &mut self.shm
    }
}

impl Dispatch<ZwlrScreencopyManagerV1, ()> for CapturerState {
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
impl Dispatch<ZwlrScreencopyFrameV1, WlOutput> for CapturerState {
    fn event(
        state: &mut Self,
        proxy: &ZwlrScreencopyFrameV1,
        event: <ZwlrScreencopyFrameV1 as wayland_client::Proxy>::Event,
        output: &WlOutput,
        conn: &Connection,
        _qhandle: &QueueHandle<Self>,
    ) {
        println!("Recieved frame event");
        use zwlr_screencopy_frame_v1::Event;
        match event {
            Event::Ready { .. } => {
                let Some(&(width,_string, height, fmt , Some((buff_ptr, buff_size)))) = state.output_buffers.get(output)else {
                    dbg!("Failed to get buffer parameters for display");
                    return;
                };
                let file_result = OpenOptions::new()
                    .write(true)
                    .read(true)
                    .create(true)
                    .open("./capture.png");
                let Ok(mut file) = file_result else {
                    dbg!("Falied to write");
                    return;
                };
                let buff: Vec<u8> = unsafe {
                    slice::from_raw_parts(buff_ptr, buff_size)
                        .chunks(4)
                        .flat_map(|pixel| match fmt {
                            Format::Xbgr8888 => [pixel[0], pixel[1], pixel[2], pixel[3]],
                            Format::Argb8888 => [pixel[1], pixel[2], pixel[3], pixel[0]],
                            _ => {
                                dbg!(fmt);
                                panic!("Unsupported format");
                            }
                        })
                        .collect()
                };
                let framebuffer: ImageBuffer<Rgba<u8>, Vec<u8>> = ImageBuffer::from_raw(
                    width.try_into().unwrap(),
                    height.try_into().unwrap(),
                    buff,
                )
                .unwrap();
                framebuffer
                    .write_to(&mut file, ImageOutputFormat::Png)
                    .expect("Failed to write image");
            }
            Event::Buffer {
                format,
                width,
                height,
                stride,
            } => match format {
                wayland_client::WEnum::Value(fmt) => {
                    let (width, height, stride) = (
                        width.try_into().unwrap(),
                        height.try_into().unwrap(),
                        stride.try_into().unwrap(),
                    );
                    state
                        .output_buffers
                        .insert(output.clone(), (width, stride, height, fmt, None));
                }
                _ => (),
            },
            Event::BufferDone => {
                let Some(mut buff_meta) = state.output_buffers.remove(output) else {
                    dbg!("Failed to get buffer parameters for display");
                    return;
                };
                let &(width, stride, height, fmt, ..) = &buff_meta;
                let buff_create_result = state
                    .shm_pool
                    .create_buffer(width, stride, height, output, fmt);
                let buff_info = match buff_create_result {
                    Ok(buff_info) => buff_info,
                    Err(e) => {
                        dbg!(e);
                        return;
                    }
                };
                let buff_contents = buff_info.2;
                buff_meta.4 = Some((buff_contents.as_ptr(), buff_contents.len()));
                state.output_buffers.insert(output.clone(), buff_meta);
                proxy.copy(buff_info.1);
                let _ = conn.flush();
                let _ = conn.roundtrip();
            }
            _ => (),
        }
    }
}
