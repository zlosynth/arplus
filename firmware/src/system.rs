pub use stm32h7xx_hal as hal;

use daisy::led::LedUser;
use hal::pac::CorePeripherals;
use hal::pac::Peripherals as DevicePeripherals;
use systick_monotonic::Systick;

use crate::audio::AudioInterface;
use crate::control_input::ControlInputInterface;
use crate::control_output::ControlOutputInterface;
use crate::flash_memory::FlashMemoryInterface;

pub struct System {
    pub mono: Systick<1000>,
    pub status_led: LedUser,
    pub audio_interface: AudioInterface,
    pub flash_memory_interface: FlashMemoryInterface,
    pub control_input_interface: ControlInputInterface,
    pub control_output_interface: ControlOutputInterface,
}

impl System {
    /// Initialize system abstraction.
    ///
    /// # Panics
    ///
    /// The system can be initialized only once. It panics otherwise.
    #[must_use]
    pub fn init(mut cp: CorePeripherals, dp: DevicePeripherals) -> Self {
        enable_cache(&mut cp);

        let board = daisy::Board::take().unwrap();
        let ccdr = daisy::board_freeze_clocks!(board, dp);
        let pins = daisy::board_split_gpios!(board, ccdr, dp);

        let system_frequency = ccdr.clocks.sys_ck();
        let mono = Systick::new(cp.SYST, system_frequency.raw());
        let status_led = daisy::board_split_leds!(pins).USER;
        let audio_interface = AudioInterface::init(daisy::board_split_audio!(ccdr, pins));
        let flash_memory_interface = FlashMemoryInterface;
        let control_input_interface = ControlInputInterface;
        let control_output_interface = ControlOutputInterface;

        Self {
            mono,
            status_led,
            audio_interface,
            flash_memory_interface,
            control_input_interface,
            control_output_interface,
        }
    }
}

/// AN5212: Improve application performance when fetching instruction and
/// data, from both internal andexternal memories.
fn enable_cache(cp: &mut CorePeripherals) {
    cp.SCB.enable_icache();
    // NOTE: This requires cache management around all use of DMA.
    cp.SCB.enable_dcache(&mut cp.CPUID);
}
