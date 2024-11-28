use super::primitives::discrete::Discrete;
use super::primitives::discrete::PersistentConfig as DiscretePersistentConfig;

pub struct CvMapping {
    chord_size: Discrete,
    scale_group: Discrete,
    scale: Discrete,
    arp: Discrete,
    tonic: Discrete,
    pluck: Discrete,
}

#[derive(Clone, Copy, defmt::Format, PartialEq, Default, Debug)]
pub struct PersistentConfig {
    chord_size: DiscretePersistentConfig,
    scale_group: DiscretePersistentConfig,
    scale: DiscretePersistentConfig,
    arp: DiscretePersistentConfig,
    tonic: DiscretePersistentConfig,
    pluck: DiscretePersistentConfig,
}

// ALLOW: `None` is constructed as the default from usize.
#[allow(dead_code)]
#[repr(usize)]
#[derive(Clone, Copy, defmt::Format, PartialEq, Debug)]
pub enum Socket {
    None = 0,
    Tone,
    Chord,
    Resonance,
    Cutoff,
    Contour,
}

impl CvMapping {
    pub fn new(config: PersistentConfig) -> Self {
        Self {
            chord_size: Discrete::new(config.chord_size, 7, 0.1, 1.0),
            scale_group: Discrete::new(config.scale_group, 7, 0.1, 1.0),
            scale: Discrete::new(config.scale, 7, 0.1, 1.0),
            arp: Discrete::new(config.arp, 7, 0.1, 1.0),
            tonic: Discrete::new(config.tonic, 7, 0.1, 1.0),
            pluck: Discrete::new(config.tonic, 7, 0.1, 1.0),
        }
    }

    pub fn copy_config(&self) -> PersistentConfig {
        PersistentConfig {
            chord_size: self.chord_size.copy_config(),
            scale_group: self.scale_group.copy_config(),
            scale: self.scale.copy_config(),
            arp: self.arp.copy_config(),
            tonic: self.tonic.copy_config(),
            pluck: self.tonic.copy_config(),
        }
    }

    pub fn reconcile_scale_group_mapping(&mut self, input_level: f32) -> bool {
        self.scale_group.reconcile(input_level)
    }

    pub fn scale_group_socket(&self) -> Socket {
        // SAFETY: Maximum selected value is limited.
        self.scale_group.selected_value().try_into().unwrap()
    }

    pub fn reconcile_scale_mapping(&mut self, input_level: f32) -> bool {
        self.scale.reconcile(input_level)
    }

    pub fn scale_socket(&self) -> Socket {
        // SAFETY: Maximum selected value is limited.
        self.scale.selected_value().try_into().unwrap()
    }

    pub fn reconcile_arp_mapping(&mut self, input_level: f32) -> bool {
        self.arp.reconcile(input_level)
    }

    pub fn arp_socket(&self) -> Socket {
        // SAFETY: Maximum selected value is limited.
        self.arp.selected_value().try_into().unwrap()
    }

    pub fn reconcile_tonic_mapping(&mut self, input_level: f32) -> bool {
        self.tonic.reconcile(input_level)
    }

    pub fn tonic_socket(&self) -> Socket {
        // SAFETY: Maximum selected value is limited.
        self.tonic.selected_value().try_into().unwrap()
    }

    pub fn reconcile_chord_size_mapping(&mut self, input_level: f32) -> bool {
        self.chord_size.reconcile(input_level)
    }

    pub fn chord_size_socket(&self) -> Socket {
        // SAFETY: Maximum selected value is limited.
        self.chord_size.selected_value().try_into().unwrap()
    }

    pub fn reconcile_pluck_mapping(&mut self, input_level: f32) -> bool {
        self.pluck.reconcile(input_level)
    }

    pub fn pluck_socket(&self) -> Socket {
        // SAFETY: Maximum selected value is limited.
        self.pluck.selected_value().try_into().unwrap()
    }

    pub fn is_socket_remapped(&self, socket: Socket) -> bool {
        self.scale_group_socket() == socket
            || self.scale_socket() == socket
            || self.arp_socket() == socket
            || self.tonic_socket() == socket
            || self.chord_size_socket() == socket
            || self.pluck_socket() == socket
    }
}

impl Socket {
    pub fn is_some(&self) -> bool {
        !self.is_none()
    }

    pub fn is_none(&self) -> bool {
        matches!(self, Self::None)
    }
}

impl TryFrom<usize> for Socket {
    type Error = ();

    fn try_from(index: usize) -> Result<Self, Self::Error> {
        if index >= 7 {
            return Err(());
        }
        Ok(unsafe { core::mem::transmute(index) })
    }
}
