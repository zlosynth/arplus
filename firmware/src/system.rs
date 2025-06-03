pub use stm32h7xx_hal as hal;

use daisy::led::LedUser;
use fugit::Hertz;
use hal::adc::{AdcSampleTime, Resolution};
use hal::delay::DelayFromCountDownTimer;
use hal::pac::CorePeripherals;
use hal::pac::Peripherals as DevicePeripherals;
use hal::prelude::*;
use systick_monotonic::Systick;

use crate::audio::AudioInterface;
use crate::audio_probe::AudioProbeInterface;
use crate::control_input::{
    ButtonsPins as ControlInputButtonsPins, Config as ControlInputConfig, ControlInputInterface,
    CvsPins as ControlInputCvsPins, GatesPins as ControlInputGatesPins,
    MultiplexerPins as ControlInputMultiplexerPins, PotsPins as ControlInputPotsPins,
};
use crate::control_output::{
    Config as ControlOutputConfig, ControlOutputInterface, Pins as ControlOutputPins,
};
use crate::flash_memory::FlashMemoryInterface;
use crate::random_generator::RandomGenerator;

pub struct System {
    pub frequency: Hertz<u32>,
    pub mono: Systick<1000>,
    pub status_led: LedUser,
    pub random_generator: RandomGenerator,
    pub audio_interface: AudioInterface,
    pub flash_memory_interface: FlashMemoryInterface,
    pub control_input_interface: ControlInputInterface,
    pub control_output_interface: ControlOutputInterface,
    pub audio_probe_interface: AudioProbeInterface,
}

impl System {
    /// Initialize system abstraction.
    ///
    /// # Panics
    ///
    /// The system can be initialized only once. It panics otherwise.
    pub fn init(mut cp: CorePeripherals, dp: DevicePeripherals) -> Self {
        enable_cache(&mut cp);

        // PANIC: This is safe, since it's only called once.
        let board = daisy::Board::take().unwrap();
        let ccdr = daisy::board_freeze_clocks!(board, dp);
        let pins = daisy::board_split_gpios!(board, ccdr, dp);

        let system_frequency = ccdr.clocks.sys_ck();
        let mono = Systick::new(cp.SYST, system_frequency.raw());
        let status_led = daisy::board_split_leds!(pins).USER;
        let random_generator =
            RandomGenerator::from_rng(dp.RNG.constrain(ccdr.peripheral.RNG, &ccdr.clocks));
        let audio_interface = AudioInterface::new(daisy::board_split_audio!(ccdr, pins));
        let flash_memory_interface =
            FlashMemoryInterface::new(daisy::board_split_flash!(ccdr, dp, pins));
        let mut delay = DelayFromCountDownTimer::new(dp.TIM2.timer(
            100.Hz(),
            ccdr.peripheral.TIM2,
            &ccdr.clocks,
        ));
        let control_input_interface = {
            let (adc_1, adc_2) = {
                let (mut adc_1, mut adc_2) = hal::adc::adc12(
                    dp.ADC1,
                    dp.ADC2,
                    4.MHz(),
                    &mut delay,
                    ccdr.peripheral.ADC12,
                    &ccdr.clocks,
                );
                adc_1.set_resolution(Resolution::SixteenBit);
                adc_1.set_sample_time(AdcSampleTime::T_16);
                adc_2.set_resolution(Resolution::SixteenBit);
                adc_2.set_sample_time(AdcSampleTime::T_16);
                (adc_1.enable(), adc_2.enable())
            };
            ControlInputInterface::new(ControlInputConfig {
                buttons_pins: ControlInputButtonsPins {
                    button_mux: pins.GPIO.PIN_B7.into_floating_input(),
                },
                pots_pins: ControlInputPotsPins {
                    pot_mux: pins.GPIO.PIN_A2.into_analog(),
                    pot_9: pins.GPIO.PIN_C5.into_analog(),
                    pot_10: pins.GPIO.PIN_C6.into_analog(),
                },
                cvs_pins: ControlInputCvsPins {
                    cv_1: pins.GPIO.PIN_C9.into_analog(),
                    cv_2: pins.GPIO.PIN_C8.into_analog(),
                    cv_3: pins.GPIO.PIN_C7.into_analog(),
                    cv_4: pins.GPIO.PIN_C2.into_analog(),
                    cv_5: pins.GPIO.PIN_C3.into_analog(),
                    cv_6: pins.GPIO.PIN_C4.into_analog(),
                },
                gates_pins: ControlInputGatesPins {
                    gate_1: pins.GPIO.PIN_B10.into_floating_input(),
                    gate_2: pins.GPIO.PIN_B9.into_floating_input(),
                },
                // FIXME: Based on the layout, this should be B5. There is a mismatch between
                // datasheet https://static1.squarespace.com/static/58d03fdc1b10e3bf442567b8/t/628bc1307a1e2b5bc04af099/1653326133665/ES_Patch_SM_datasheet_v1.0.4.pdf
                // and lib daisy https://github.com/electro-smith/libDaisy/blob/master/src/daisy_patch_sm.cpp
                // report this issue, and fix it in my daisy library.
                probe_pin: pins.GPIO.PIN_B6.into_push_pull_output(),
                multiplexer_pins: ControlInputMultiplexerPins {
                    address_a: pins.GPIO.PIN_A9.into_push_pull_output(),
                    address_b: pins.GPIO.PIN_A8.into_push_pull_output(),
                    address_c: pins.GPIO.PIN_A3.into_push_pull_output(),
                },
                adc_1,
                adc_2,
            })
        };
        let control_output_interface = {
            let dac = dp
                .DAC
                .dac(pins.GPIO.PIN_C10, ccdr.peripheral.DAC12)
                .calibrate_buffer(&mut delay)
                .enable();
            ControlOutputInterface::new(ControlOutputConfig {
                pins: ControlOutputPins {
                    led_ser: pins.GPIO.PIN_D1.into_push_pull_output(),
                    led_srclk: pins.GPIO.PIN_D2.into_push_pull_output(),
                    led_rclk: pins.GPIO.PIN_D3.into_push_pull_output(),
                },
                dac,
            })
        };
        // FIXME: Based on the layout, this should be B6. There is a mismatch between
        // datasheet https://static1.squarespace.com/static/58d03fdc1b10e3bf442567b8/t/628bc1307a1e2b5bc04af099/1653326133665/ES_Patch_SM_datasheet_v1.0.4.pdf
        // and lib daisy https://github.com/electro-smith/libDaisy/blob/master/src/daisy_patch_sm.cpp
        // report this issue, and fix it in my daisy library.
        let audio_probe_interface =
            AudioProbeInterface::new(pins.GPIO.PIN_B5.into_push_pull_output());

        Self {
            frequency: system_frequency,
            mono,
            status_led,
            random_generator,
            audio_interface,
            flash_memory_interface,
            control_input_interface,
            control_output_interface,
            audio_probe_interface,
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
