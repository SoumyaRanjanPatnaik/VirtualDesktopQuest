use std::{
    error::Error,
    io::{Stdin, Stdout},
};

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

use super::output_info::{OutputMetadataVariant, WlOutputMapping};
#[derive(Debug, Default)]
pub struct CapturerState {
    pub outputs_old: Vec<WlOutput>,
    pub outputs: Vec<Option<WlOutputMapping>>,
    globals_list: Vec<Global>,
}

impl CapturerState {
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

impl Dispatch<wl_registry::WlRegistry, ()> for CapturerState {
    fn event(
        state: &mut Self,
        _: &wl_registry::WlRegistry,
        event: wl_registry::Event,
        _: &(),
        _: &Connection,
        _qh: &QueueHandle<CapturerState>,
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

impl Dispatch<wl_output::WlOutput, ((), usize)> for CapturerState {
    fn event(
        state: &mut Self,
        wl_output: &wl_output::WlOutput,
        event: wl_output::Event,
        &(_, index): &((), usize),
        _: &Connection,
        _qh: &QueueHandle<CapturerState>,
    ) {
        let outputs = &mut state.outputs;
        if index >= outputs.len() {
            outputs.resize(index, None);
        }
        if outputs[index].is_none() {
            outputs[index] = Some(WlOutputMapping::new(&wl_output));
        }
        let curr_output = outputs[index].as_mut().unwrap();
        match event {
            wl_output::Event::Name { name } => {
                curr_output.metadata.to_partial();
                match &mut curr_output.metadata {
                    OutputMetadataVariant::Partial(meta) => meta.name = Some(name),
                    OutputMetadataVariant::Complete(_) => (),
                }
            }
            wl_output::Event::Mode {
                width,
                height,
                refresh,
                ..
            } => {
                curr_output.metadata.to_partial();
                match &mut curr_output.metadata {
                    OutputMetadataVariant::Partial(meta) => {
                        meta.mode = Some((height, width, refresh))
                    }
                    OutputMetadataVariant::Complete(_) => (),
                }
            }

            wl_output::Event::Description { description } => {
                curr_output.metadata.to_partial();
                match &mut curr_output.metadata {
                    OutputMetadataVariant::Partial(meta) => meta.description = Some(description),
                    OutputMetadataVariant::Complete(_) => (),
                }
            }
            wl_output::Event::Scale { factor } => {
                curr_output.metadata.to_partial();
                match &mut curr_output.metadata {
                    OutputMetadataVariant::Partial(meta) => meta.scale = Some(factor),
                    OutputMetadataVariant::Complete(_) => (),
                }
            }
            wl_output::Event::Done => {
                if let Err(e) = curr_output.metadata.to_complete() {
                    panic!("{}", e);
                };
                ()
            }
            _ => (),
        }
    }
}

impl Dispatch<wl_shm::WlShm, ()> for CapturerState {
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

impl Dispatch<wl_shm_pool::WlShmPool, ()> for CapturerState {
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
