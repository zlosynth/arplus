pub use daisy::flash::Flash;

use arplus_control::{Save, WrappedSave};

const NUM_SECTORS: usize = 2048;

pub struct FlashMemoryInterface {
    flash: Flash,
    version: u32,
}

impl FlashMemoryInterface {
    pub fn new(flash: Flash) -> Self {
        Self { flash, version: 0 }
    }

    pub fn save(&mut self, save: Save) {
        defmt::info!("Saving version={:?}: {:?}", self.version, save);
        let data = WrappedSave::new(save, self.version).to_bytes();
        self.flash
            .write(sector_address(self.version as usize % NUM_SECTORS), &data);
        self.version = self.version.wrapping_add(1);
    }

    pub fn load(&mut self) -> Save {
        let mut latest_store: Option<WrappedSave> = None;

        for i in 0..NUM_SECTORS {
            let mut store_buffer = [0; WrappedSave::SIZE];

            self.flash.read(sector_address(i), &mut store_buffer);

            if let Ok(store) = WrappedSave::from_bytes(store_buffer) {
                if let Some(latest) = latest_store {
                    if store.version() > latest.version() {
                        latest_store = Some(store);
                    }
                } else {
                    latest_store = Some(store);
                }
            }
        }

        if let Some(latest) = latest_store {
            let save = latest.save();
            defmt::info!("Loaded save version={:?}: {:?}", latest.version(), save);
            self.version = latest.version() + 1;
            save
        } else {
            defmt::info!("No valid save was found");
            Save::default()
        }
    }
}

fn sector_address(sector_index: usize) -> u32 {
    (sector_index << 12) as u32
}
