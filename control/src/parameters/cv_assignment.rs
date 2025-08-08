use super::primitives::toggle::{PersistentConfig as TogglePersistentConfig, Toggle};

pub struct CvAssignmentHandler {
    toggle: Toggle,
    just_changed: bool,
}

#[derive(Clone, Copy, defmt::Format, PartialEq, Default, Debug)]
pub struct PersistentConfig {
    toggle: TogglePersistentConfig,
}

// ALLOW: `None` is constructed as the default from usize.
#[allow(dead_code)]
#[repr(u8)]
#[derive(Clone, Copy, PartialEq, Debug, defmt::Format)]
pub enum CvAssignment {
    Tonic = 0,
    Size,
    Arp,
    Group,
    Scale,
    Pluck,
    Strings,
    Width,
}

impl CvAssignmentHandler {
    pub fn new(config: PersistentConfig) -> Self {
        Self {
            toggle: Toggle::new(config.toggle, CvAssignment::LAST_ATTRIBUTE as usize + 1),
            just_changed: false,
        }
    }

    pub fn copy_config(&self) -> PersistentConfig {
        PersistentConfig {
            toggle: self.toggle.copy_config(),
        }
    }

    pub fn reconcile_button(&mut self, toggle: bool) -> bool {
        let changed = self.toggle.reconcile(toggle);
        self.just_changed = changed;
        changed
    }

    pub fn selected(&self) -> CvAssignment {
        let selected_value = self.toggle.selected_value();
        // PANIC: Parameter values used to get assignment index are statically limited
        // by the maximum number of attibutes. However, this maximum is defined by
        // a manually set constant on `CvAssignment`. This is error-prone - variant can
        // be removed without updating the constant. Because of that, this
        // gracefully returns a fall-back value and logs and error on failure.
        CvAssignment::try_from_index(selected_value).unwrap_or_else(|| {
            defmt::error!("Attempted to create CvAssignment from invalid index");
            CvAssignment::default()
        })
    }

    pub fn just_changed(&self) -> bool {
        self.just_changed
    }
}

impl Default for CvAssignment {
    fn default() -> Self {
        Self::Tonic
    }
}

impl CvAssignment {
    pub const LAST_ATTRIBUTE: Self = Self::Width;

    pub fn index(self) -> usize {
        self as usize
    }

    pub fn try_from_index(index: usize) -> Option<Self> {
        if index <= Self::LAST_ATTRIBUTE.index() {
            Some(unsafe { core::mem::transmute::<u8, Self>(index as u8) })
        } else {
            None
        }
    }
}
