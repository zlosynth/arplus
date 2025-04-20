use daisy::audio::{self, Interface};

pub const BLOCK_LENGTH: usize = audio::BLOCK_LENGTH;
// pub const SAMPLE_RATE: u32 = audio::FS.to_Hz();
// NOTE: The SAMPLE_RATE needs to be adjusted. Probably because the clock on
// STM32 is unable to exactly match the speed. With this value, it is perfectly
// in tune on A4.
pub const SAMPLE_RATE: u32 = 47_793 * 2;

pub struct AudioInterface {
    interface: Option<Interface>,
}

impl AudioInterface {
    pub fn new(interface: daisy::audio::Interface) -> Self {
        Self {
            interface: Some(interface),
        }
    }

    /// Spawn audio processing.
    ///
    /// # Panics
    ///
    /// Audio processing can be spawned only once. It panics otherwise.
    pub fn spawn(&mut self) {
        // PANIC: Documented panic. Should work fine. If this fails, it's very
        // bad and the module would not function anyway.
        self.interface = Some(self.interface.take().unwrap().spawn().unwrap());
    }

    /// Process audio buffer.
    ///
    /// # Panics
    ///
    /// This can panic if executed outside of `DMA1_STR1` interrupt or if it
    /// took too long to process.
    pub fn update_buffer(&mut self, callback: impl FnMut(&mut [(f32, f32); BLOCK_LENGTH])) {
        self.interface
            .as_mut()
            // PANIC: This won't panic unless this method is called before
            // spawn. That would be a serious bug impossible to recover from
            // so this is ok to keep.
            .unwrap()
            .handle_interrupt_dma1_str1(callback)
            // PANIC: This would fail if timing is bad, or if the method is
            // not used correctly. Both are unlikely, and if they happened,
            // they are a serious bug that should not be ignored.
            .unwrap();
    }
}
