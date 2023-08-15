use std::{
    error::Error,
    ops::{Deref, DerefMut},
};

use wayland_client::{
    globals::Global,
    protocol::{wl_display::WlDisplay, wl_output, wl_registry},
    Dispatch, QueueHandle,
};

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct Registry(wl_registry::WlRegistry);

#[allow(dead_code)]
impl Registry {
    pub fn new<D, U>(
        display: &WlDisplay,
        qh: &QueueHandle<D>,
        udata: U,
    ) -> Result<Registry, Box<dyn Error>>
    where
        U: Clone + Send + Sync + 'static,
        D: Dispatch<wl_registry::WlRegistry, U> + 'static,
    {
        let registry = display.get_registry(&qh, udata);
        Ok(Registry(registry))
    }

    pub fn get_outputs<D, U>(
        &self,
        output_globals: &[Global],
        qh: &QueueHandle<D>,
        udata: U,
    ) -> Vec<wl_output::WlOutput>
    where
        U: Clone + Send + Sync + 'static,
        D: Dispatch<wl_output::WlOutput, (U, usize)> + 'static,
    {
        let mut outputs = vec![];
        for (index, output_obj) in output_globals.iter().enumerate() {
            let output: wl_output::WlOutput = self.bind(
                output_obj.name,
                output_obj.version,
                &qh,
                (udata.to_owned(), index),
            );
            outputs.push(output);
        }
        outputs
    }

    pub fn bind_global<I, D, U>(&self, global: &Global, qh: &QueueHandle<D>, udata: U) -> I
    where
        I: wayland_client::Proxy + 'static,
        U: Send + Sync + 'static,
        D: Dispatch<I, U> + 'static,
    {
        self.bind(global.name, global.version, qh, udata)
    }
}

impl Deref for Registry {
    type Target = wl_registry::WlRegistry;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Registry {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

pub struct EventQueue<S>(wayland_client::EventQueue<S>);

impl<S> EventQueue<S> {
    pub fn new(conn: &wayland_client::Connection) -> EventQueue<S> {
        let event_queue = conn.new_event_queue();
        EventQueue(event_queue)
    }

    pub fn wait_for<T>(
        &mut self,
        state: &mut S,
        method: impl Fn() -> T,
    ) -> Result<T, Box<dyn Error>> {
        let result = method();
        self.0.roundtrip(state)?;
        Ok(result)
    }
}

impl<S> Deref for EventQueue<S> {
    type Target = wayland_client::EventQueue<S>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<S> DerefMut for EventQueue<S> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
