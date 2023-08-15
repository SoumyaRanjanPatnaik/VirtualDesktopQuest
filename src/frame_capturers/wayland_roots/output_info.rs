use crate::types::OutputMetadata;
use derive_is_enum_variant::is_enum_variant;
use std::mem;
use wayland_client::protocol::wl_output::WlOutput;

#[derive(Debug, Clone, is_enum_variant)]
pub enum OutputMetadataVariant {
    Partial(WlOutputMetadataPartial),
    Complete(OutputMetadata),
}

impl OutputMetadataVariant {
    pub fn to_partial(&mut self) {
        match self {
            OutputMetadataVariant::Partial(_) => (),
            OutputMetadataVariant::Complete(meta) => {
                *self = OutputMetadataVariant::Partial(WlOutputMetadataPartial {
                    name: Some(meta.name.clone()),
                    description: meta.description.clone(),
                    mode: Some(meta.mode),
                    scale: Some(meta.scale),
                })
            }
        };
    }
    pub fn to_complete(&mut self) -> Result<(), String> {
        let missing_err =
            |name| format!("Cannot construct type OutputMetadata: Missing property {name} ");
        match self {
            OutputMetadataVariant::Partial(meta) => {
                let name = mem::take(&mut meta.name).ok_or(missing_err("name"))?;
                let description = mem::take(&mut meta.description);
                let scale = mem::take(&mut meta.scale).ok_or(missing_err("scale"))?;
                let mode = mem::take(&mut meta.mode).ok_or(missing_err("mode"))?;
                *self = OutputMetadataVariant::Complete(OutputMetadata {
                    name,
                    description,
                    scale,
                    mode,
                });
            }
            OutputMetadataVariant::Complete(_) => (),
        }
        Ok(())
    }
}

impl Default for OutputMetadataVariant {
    fn default() -> Self {
        Self::Partial(WlOutputMetadataPartial::default())
    }
}

#[derive(Debug, Clone, Default)]
pub struct WlOutputMetadataPartial {
    pub name: Option<String>,
    pub description: Option<String>,
    pub mode: Option<(i32, i32, i32)>,
    pub scale: Option<i32>,
}

#[derive(Debug, Clone)]
pub struct WlOutputMapping {
    pub wl_output: WlOutput,
    pub metadata: OutputMetadataVariant,
}

impl WlOutputMapping {
    pub fn new(wl_output: &WlOutput) -> Self {
        Self {
            wl_output: wl_output.clone(),
            metadata: OutputMetadataVariant::default(),
        }
    }
}
