use std::error::Error;

use wayland_client::{
    globals::Global,
    protocol::{
        wl_output::{self, WlOutput},
        wl_registry, wl_shm, wl_shm_pool,
    },
    Connection, Dispatch, QueueHandle,
};

use wayland_protocols_wlr::screencopy::v1::client::{
    zwlr_screencopy_frame_v1::ZwlrScreencopyFrameV1,
    zwlr_screencopy_manager_v1::ZwlrScreencopyManagerV1,
};
#[derive(Debug, Default)]
pub struct AppData {
    pub outputs: Vec<WlOutput>,
    globals_list: Vec<Global>,
}

impl AppData {
    /// Get info on globals that match the interface names
    pub fn get_globals_by_interface_name<'a, 'b>(&self, interface: &str) -> Vec<Global> {
        self.globals_list
            .iter()
            .filter(|global| global.interface == interface)
            .map(|gobal_ref| gobal_ref.clone())
            .collect()
    }
    pub fn get_global_by_interface_name<'a, 'b>(
        &self,
        interface: &str,
    ) -> Result<Global, Box<dyn Error>> {
        let global = self
            .globals_list
            .iter()
            .filter(|global| global.interface == interface)
            .map(|gobal_ref| gobal_ref.clone())
            .nth(0)
            .ok_or(format!("No global found with interface name: {interface}"))?;
        Ok(global)
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
