use std::error::Error;

use smithay_client_toolkit::{
    delegate_output, delegate_registry, delegate_shm,
    globals::GlobalData,
    output::{OutputData, OutputHandler, OutputState},
    reexports::protocols::xdg::xdg_output::zv1::client::{zxdg_output_manager_v1, zxdg_output_v1},
    registry::{ProvidesRegistryState, RegistryState},
    registry_handlers,
    shm::{Shm, ShmHandler},
};
use wayland_client::{
    globals::GlobalList,
    protocol::{wl_output, wl_shm::WlShm},
    Connection, Dispatch, QueueHandle,
};

use wayland_protocols_wlr::screencopy::v1::client::{
    zwlr_screencopy_frame_v1::ZwlrScreencopyFrameV1,
    zwlr_screencopy_manager_v1::ZwlrScreencopyManagerV1,
};

#[derive(Debug)]
pub struct CapturerState {
    pub output_state: OutputState,
    registry: RegistryState,
    shm: Shm,
}

impl CapturerState {
    pub fn new<D>(global_list: &GlobalList, qh: &QueueHandle<D>) -> Result<Self, Box<dyn Error>>
    where
        D: Dispatch<wl_output::WlOutput, OutputData>
            + Dispatch<zxdg_output_v1::ZxdgOutputV1, OutputData>
            + Dispatch<zxdg_output_manager_v1::ZxdgOutputManagerV1, GlobalData>
            + Dispatch<WlShm, GlobalData>
            + ShmHandler
            + 'static,
    {
        let output_state = OutputState::new(&global_list, qh);
        let registry = RegistryState::new(&global_list);
        let shm = Shm::bind(&global_list, qh)?;
        Ok(Self {
            output_state,
            registry,
            shm,
        })
    }
}

delegate_registry!(CapturerState);
impl ProvidesRegistryState for CapturerState {
    fn registry(&mut self) -> &mut RegistryState {
        &mut self.registry
    }
    registry_handlers!(OutputState);
}

delegate_output!(CapturerState);
impl OutputHandler for CapturerState {
    fn output_destroyed(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _output: wl_output::WlOutput,
    ) {
    }
    fn update_output(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _output: wl_output::WlOutput,
    ) {
    }
    fn new_output(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _output: wl_output::WlOutput,
    ) {
    }
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
impl Dispatch<ZwlrScreencopyFrameV1, ()> for CapturerState {
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
