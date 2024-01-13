#![no_main]
#![no_std]

use arplus_firmware as _; // Global logger and panicking behavior.

#[rtic::app(device = stm32h7xx_hal::pac, peripherals = true, dispatchers = [EXTI0, EXTI1, EXTI2])]
mod app {
    use heapless::spsc::{Consumer, Producer, Queue};
    use systick_monotonic::Systick;

    use arplus_dsp::{Attributes as InstrumentAttributes, Instrument};
    use arplus_firmware::queue_utils;
    use arplus_firmware::system::audio::{AudioInterface, SAMPLE_RATE};
    use arplus_firmware::system::System;
    use arplus_firmware::version_indicator::VersionIndicator;

    // Blinks on the PCB's LED signalize the current revision.
    const BLINKS: u8 = 1;

    // 1 kHz granularity for task scheduling.
    #[monotonic(binds = SysTick, default = true)]
    type Mono = Systick<1000>;

    #[shared]
    struct Shared {}

    #[local]
    struct Local {
        version_indicator: VersionIndicator,
        audio_interface: AudioInterface,
        instrument: Instrument,
        _instrument_attributes_producer: Producer<'static, InstrumentAttributes, 8>,
        instrument_attributes_consumer: Consumer<'static, InstrumentAttributes, 8>,
    }

    #[init(
        local = [
            instrument_attributes_queue: Queue<InstrumentAttributes, 8> = Queue::new(),
        ]
    )]
    fn init(cx: init::Context) -> (Shared, Local, init::Monotonics) {
        defmt::info!("Starting the firmware, initializing resources");

        let (instrument_attributes_producer, instrument_attributes_consumer) =
            cx.local.instrument_attributes_queue.split();

        let system = System::init(cx.core, cx.device);
        let mono = system.mono;
        let mut audio_interface = system.audio_interface;

        let version_indicator = VersionIndicator::new(BLINKS, system.status_led);
        let instrument = Instrument::new(SAMPLE_RATE as f32);

        defmt::info!("Initialization was completed, starting tasks");

        audio_interface.spawn();
        version_indicator_alarm::spawn().unwrap();

        (
            Shared {},
            Local {
                version_indicator,
                audio_interface,
                instrument,
                _instrument_attributes_producer: instrument_attributes_producer,
                instrument_attributes_consumer,
            },
            init::Monotonics(mono),
        )
    }

    #[task(
        binds = DMA1_STR1,
        local = [
            audio_interface,
            instrument,
            instrument_attributes_consumer,
        ],
        priority = 4,
    )]
    fn dsp_loop(cx: dsp_loop::Context) {
        let audio_interface = cx.local.audio_interface;
        let instrument = cx.local.instrument;
        let instrument_attributes_consumer = cx.local.instrument_attributes_consumer;

        queue_utils::warn_about_capacity("instrument_attributes", instrument_attributes_consumer);

        if let Some(attributes) = queue_utils::dequeue_last(instrument_attributes_consumer) {
            instrument.set_attributes(attributes);
        }

        audio_interface.update_buffer(|buffer| {
            instrument.process(buffer);
        });
    }

    #[task(local = [version_indicator])]
    fn version_indicator_alarm(cx: version_indicator_alarm::Context) {
        let version_indicator = cx.local.version_indicator;
        let required_sleep = version_indicator.cycle();
        version_indicator_alarm::spawn_after(required_sleep).unwrap();
    }
}
