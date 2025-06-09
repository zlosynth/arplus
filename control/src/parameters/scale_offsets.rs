pub const MAX_STEPS: usize = 7;
const MAX_SCALES: usize = 8;
const MAX_GROUPS: usize = 5;

pub type Offsets = [[[i8; MAX_STEPS]; MAX_SCALES]; MAX_GROUPS];

#[derive(PartialEq, Default, Debug, Clone, Copy, defmt::Format)]
pub struct ScaleOffsets {
    offsets: Offsets,
    locked: bool,
}

#[derive(Default, PartialEq, Debug, Clone, Copy, defmt::Format)]
pub struct PersistentConfig {
    offsets: Offsets,
}

impl ScaleOffsets {
    pub fn new(config: PersistentConfig) -> Self {
        Self {
            offsets: config.offsets,
            locked: true,
        }
    }

    pub fn copy_config(&self) -> PersistentConfig {
        PersistentConfig {
            offsets: self.offsets,
        }
    }

    pub fn request_increase(&mut self, group: usize, scale: usize, step: usize) -> bool {
        if let Some(offset) = self.offset_mut(group, scale, step) {
            if *offset < 4 {
                *offset += 1;
                return true;
            }
        }
        false
    }

    pub fn request_decrease(&mut self, group: usize, scale: usize, step: usize) -> bool {
        if let Some(offset) = self.offset_mut(group, scale, step) {
            if *offset > -4 {
                *offset -= 1;
                return true;
            }
        }
        false
    }

    pub fn offset(&self, group: usize, scale: usize, step: usize) -> i8 {
        if !(group < MAX_GROUPS && scale < MAX_SCALES && step < MAX_STEPS) {
            return 0;
        }

        self.offsets[group][scale][step]
    }

    fn offset_mut(&mut self, group: usize, scale: usize, step: usize) -> Option<&mut i8> {
        if !(group < MAX_GROUPS && scale < MAX_SCALES && step < MAX_STEPS) {
            return None;
        }

        Some(&mut self.offsets[group][scale][step])
    }

    pub fn scale_offsets_ref(&self, group: usize, scale: usize) -> &[i8; MAX_STEPS] {
        if !(group < MAX_GROUPS && scale < MAX_SCALES) {
            return &[0; MAX_STEPS];
        }

        &self.offsets[group][scale]
    }

    pub fn reset_scale(&mut self, group: usize, scale: usize) -> bool {
        if !(group < MAX_GROUPS && scale < MAX_SCALES) {
            return false;
        }

        let scale = &mut self.offsets[group][scale];
        if scale.iter().any(|x| *x != 0) {
            *scale = [0; MAX_STEPS];
            return true;
        }

        false
    }

    pub fn locked(&self) -> bool {
        self.locked
    }

    pub fn toggle_lock(&mut self) -> bool {
        self.locked = !self.locked;
        self.locked
    }
}
