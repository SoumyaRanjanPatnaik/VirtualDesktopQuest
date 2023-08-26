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
        wl_buffer::WlBuffer,
        wl_output::WlOutput,
        wl_shm::{Format, WlShm},
    },
    Connection, Dispatch, QueueHandle,
};

use wayland_protocols_wlr::screencopy::v1::client::{
    __interfaces::ZWLR_SCREENCOPY_MANAGER_V1_INTERFACE,
    zwlr_screencopy_frame_v1::{self, ZwlrScreencopyFrameV1},
    zwlr_screencopy_manager_v1::ZwlrScreencopyManagerV1,
};

use crate::types::{CaptureType, CapturerStatus};

#[derive(Debug)]
pub struct CapturerState {
    pub status: CapturerStatus,
    pub(super) global_list: GlobalList,
    output_state: OutputState,
    registry_state: RegistryState,
    shm: Shm,
    shm_pool: MultiPool<WlOutput>,
    output_buffers:
        HashMap<WlOutput, (i32, i32, i32, Format, Option<(*const u8, usize, WlBuffer)>)>,
}

impl CapturerState {
    pub fn new<D>(global_list: GlobalList, qh: &QueueHandle<D>) -> Result<Self, Box<dyn Error>>
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
        let shm_pool = MultiPool::new(&shm)?;
        let output_buffers = HashMap::new();
        let status = CapturerStatus::NotStarted;
        Ok(Self {
            status,
            global_list,
            output_state,
            registry_state,
            shm,
            shm_pool,
            output_buffers,
        })
    }

    pub(super) fn poll_capture(
        &mut self,
        qh: &QueueHandle<Self>,
        capture_type: CaptureType,
        output: &WlOutput,
    ) -> Result<(), Box<dyn Error>> {
        let screencopy_mgr: ZwlrScreencopyManagerV1 =
            self.global_list
                .bind(&qh, 1..=ZWLR_SCREENCOPY_MANAGER_V1_INTERFACE.version, ())?;

        self.status.initialize(capture_type)?;

        screencopy_mgr.capture_output(1, &output, &qh, output.clone());
        Ok(())
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

pub(self) fn write_image(buff: &[u8], fmt: Format, dimensions: (u32, u32)) {
    let rgba_buff: Vec<u8> = buff
        .chunks(4)
        .flat_map(|pixel| match fmt {
            Format::Xbgr8888 => [pixel[0], pixel[1], pixel[2], pixel[3]],
            Format::Argb8888 => [pixel[1], pixel[2], pixel[3], pixel[0]],
            _ => {
                dbg!(fmt);
                panic!("Unsupported format");
            }
        })
        .collect();
    let file_result = OpenOptions::new()
        .write(true)
        .read(true)
        .create(true)
        .open("./capture.png");
    let Ok(mut file) = file_result else {
                                dbg!("Falied to write");
                                return;
                            };
    let (width, height) = dimensions;
    let framebuffer: ImageBuffer<Rgba<u8>, Vec<u8>> =
        ImageBuffer::from_raw(width, height, rgba_buff).unwrap();
    framebuffer
        .write_to(&mut file, ImageOutputFormat::Png)
        .expect("Failed to write image");
}

fn calc_sec(sec_lo: u32, sec_hi: u32, nsec: u32) -> f64 {
    let tv_sec: u64 = ((sec_hi as u64) << 32) | sec_lo as u64;
    let secs: f64 = tv_sec as f64 + nsec as f64 / 1000000000.0;
    secs
}

impl Dispatch<ZwlrScreencopyFrameV1, WlOutput> for CapturerState {
    fn event(
        state: &mut Self,
        proxy: &ZwlrScreencopyFrameV1,
        event: <ZwlrScreencopyFrameV1 as wayland_client::Proxy>::Event,
        output: &WlOutput,
        _conn: &Connection,
        _qhandle: &QueueHandle<Self>,
    ) {
        use zwlr_screencopy_frame_v1::Event;
        match event {
            Event::Ready {
                tv_sec_hi,
                tv_sec_lo,
                tv_nsec,
                ..
            } => {
                let curr_time = calc_sec(tv_sec_lo, tv_sec_hi, tv_nsec);
                let Some(&(width, _stride, height, fmt , Some((buff_ptr, buff_size, .. )))) = state.output_buffers.get(output) else {
                    dbg!("Failed to get buffer parameters for display");
                    return;
                };
                let buff = unsafe { slice::from_raw_parts(buff_ptr, buff_size) };
                let CapturerStatus::Running { r#type, last_frame_at } = state.status.clone() else {
                    return
                };
                match r#type {
                    crate::types::CaptureType::Frame => write_image(
                        buff,
                        fmt,
                        (width.try_into().unwrap(), height.try_into().unwrap()),
                    ),
                    crate::types::CaptureType::Stream => {
                        println!("{curr_time}");
                        if let Some(prev_sec) = last_frame_at {
                            println!("FPS: {}", 1.0 / (prev_sec - curr_time))
                        }
                    }
                }

                state.shm_pool.remove(output);
                state.status.done().unwrap();
                state
                    .poll_capture(_qhandle, r#type.clone(), output)
                    .unwrap();
                proxy.destroy();
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
                buff_meta.4 = Some((
                    buff_contents.as_ptr(),
                    buff_contents.len(),
                    buff_info.1.clone(),
                ));
                state.output_buffers.insert(output.clone(), buff_meta);
                if let Err(e) = state.status.start() {
                    panic!("{e}")
                };
                proxy.copy(buff_info.1);
            }
            Event::Failed => {
                println!("FAILED")
            }
            _ => println!("OTHER"),
        }
    }
}
