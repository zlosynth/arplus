use super::primitives::toggle::{PersistentConfig as TogglePersistentConfig, Toggle};
use arplus_dsp as dsp;

// ALLOW: All the values are constructed via `try_from_index`.
#[allow(dead_code)]
#[repr(u8)]
#[derive(Clone, Copy, PartialEq, Debug, defmt::Format)]
pub enum StereoMode {
    PingPong = 0,
    RootRest,
    Haas,
}

impl From<StereoMode> for dsp::StereoMode {
    fn from(value: StereoMode) -> Self {
        match value {
            StereoMode::PingPong => dsp::StereoMode::PingPong,
            StereoMode::RootRest => dsp::StereoMode::RootRest,
            StereoMode::Haas => dsp::StereoMode::Haas,
        }
    }
}

pub struct StereoModeHandler {
    toggle: Toggle,
}

#[derive(Default, PartialEq, Debug, Clone, Copy, defmt::Format)]
pub struct PersistentConfig {
    toggle: TogglePersistentConfig,
}

impl StereoModeHandler {
    pub fn new(config: PersistentConfig) -> Self {
        Self {
            toggle: Toggle::new(config.toggle, StereoMode::LAST_MODE as usize + 1),
        }
    }

    pub fn copy_config(&self) -> PersistentConfig {
        PersistentConfig {
            toggle: self.toggle.copy_config(),
        }
    }

    pub fn reconcile_button(&mut self, toggle: bool) -> bool {
        self.toggle.reconcile(toggle)
    }

    pub fn selected(&self) -> StereoMode {
        let selected_value = self.toggle.selected_value();
        // PANIC: Parameter values used to get arp index are statically limited
        // by the maximum number of modes. However, this maximum is defined by
        // a manually set constant on `StereoMode`. This is error-prone - variant can
        // be removed without updating the constant. Because of that, this
        // gracefully returns a fall-back value and logs and error on failure.
        StereoMode::try_from_index(selected_value).unwrap_or_else(|| {
            defmt::error!("Attempted to create Mode from invalid index");
            StereoMode::default()
        })
    }
}

impl Default for StereoMode {
    fn default() -> Self {
        Self::RootRest
    }
}

impl StereoMode {
    pub const LAST_MODE: Self = Self::Haas;

    pub fn index(self) -> usize {
        self as usize
    }

    pub fn try_from_index(index: usize) -> Option<Self> {
        if index <= Self::LAST_MODE.index() {
            Some(unsafe { core::mem::transmute::<u8, Self>(index as u8) })
        } else {
            None
        }
    }
}
