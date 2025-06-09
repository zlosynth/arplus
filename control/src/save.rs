use core::mem;

use crc::{Crc, CRC_16_USB};

use crate::inputs::PersistentConfig as InputsPersistentConfig;
use crate::parameters::PersistentConfig as ParametersPersistentConfig;
use crate::quantized_output::PersistentConfig as QuantizedOutputConfig;

/// Subset of control structures needed for recovery after restart.
#[derive(Debug, Default, Clone, Copy, PartialEq, defmt::Format)]
pub struct Save {
    pub parameters: ParametersPersistentConfig,
    pub inputs: InputsPersistentConfig,
    pub quantized_output: QuantizedOutputConfig,
}

impl Save {
    const SIZE: usize = mem::size_of::<Self>();

    fn from_bytes(bytes: [u8; Self::SIZE]) -> Self {
        unsafe { mem::transmute(bytes) }
    }

    fn to_bytes(self) -> [u8; Self::SIZE] {
        unsafe { mem::transmute(self) }
    }
}

// This constant is used to invalidate data when needed
const TOKEN: u16 = 1;
const CRC: Crc<u16> = Crc::<u16>::new(&CRC_16_USB);
pub struct InvalidData;

#[derive(Clone, Copy)]
pub struct WrappedSave {
    version: u32,
    token: u16,
    save_raw: [u8; Save::SIZE],
    crc: u16,
}

impl WrappedSave {
    pub const SIZE: usize = mem::size_of::<Self>();

    pub fn new(save: Save, version: u32) -> Self {
        let save_raw = save.to_bytes();
        let crc = CRC.checksum(&save_raw);
        Self {
            version,
            save_raw,
            crc,
            token: TOKEN,
        }
    }

    /// # Errors
    ///
    /// This fails with `InvalidData` when recovered save does not pass CRC
    /// check.
    pub fn from_bytes(bytes: [u8; Self::SIZE]) -> Result<Self, InvalidData> {
        let store: Self = unsafe { mem::transmute(bytes) };

        if store.token != TOKEN {
            return Err(InvalidData);
        }

        let crc = CRC.checksum(&store.save_raw);
        if crc == store.crc {
            Ok(store)
        } else {
            Err(InvalidData)
        }
    }

    pub fn to_bytes(self) -> [u8; Self::SIZE] {
        unsafe { mem::transmute(self) }
    }

    pub fn save(&self) -> Save {
        Save::from_bytes(self.save_raw)
    }

    pub fn version(&self) -> u32 {
        self.version
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn initialize_store() {
        let _store = WrappedSave::new(Save::default(), 0);
    }

    #[test]
    fn get_save_from_store() {
        let save = Save::default();
        let store = WrappedSave::new(save, 0);
        assert!(store.save() == save);
    }

    #[test]
    fn get_version_from_store() {
        let store = WrappedSave::new(Save::default(), 10);
        assert_eq!(store.version(), 10);
    }

    #[test]
    fn initialize_store_from_bytes() {
        let store_a = WrappedSave::new(Save::default(), 0);
        let bytes = store_a.to_bytes();
        let store_b = WrappedSave::from_bytes(bytes).ok().unwrap();
        assert!(store_a.save() == store_b.save());
    }

    #[test]
    fn detect_invalid_crc_while_initializing_from_bytes() {
        let store = WrappedSave::new(Save::default(), 0);
        let mut bytes = store.to_bytes();
        bytes[5] = 0x13;
        assert!(WrappedSave::from_bytes(bytes).is_err());
    }

    #[test]
    fn dump_store_as_bytes() {
        let save_a = Save::default();
        let store_a = WrappedSave::new(save_a, 0);
        let bytes_a = store_a.to_bytes();

        let save_b = Save::default();
        let store_b = WrappedSave::new(save_b, 1);
        let bytes_b = store_b.to_bytes();

        assert!(bytes_a != bytes_b);
    }

    #[test]
    fn store_fits_in_two_pages() {
        // XXX: If this is changed, `firmware/src/flash_memory.rs:SAVE_SIZE_IN_SECTORS`
        // should be adjusted accordingly.
        let page_size = 256;
        let store_size = mem::size_of::<WrappedSave>();
        assert!(store_size <= page_size * 2);
    }
}
