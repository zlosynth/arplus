#![no_main]
#![no_std]

use arplus_firmware as _; // Global logger and panicking behavior.

#[rtic::app(device = stm32h7xx_hal::pac, peripherals = true, dispatchers = [EXTI0, EXTI1, EXTI2])]
mod app {
    use fugit::ExtU64;
    use heapless::spsc::{Consumer, Producer, Queue};
    use systick_monotonic::Systick;

    use arplus_control::{Controller, InputSnapshot, Save};
    use arplus_dsp::{Attributes as InstrumentAttributes, Instrument};
    use arplus_firmware::output_manager::OutputManager;
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
        controller: Controller,
        output_manager: OutputManager,
        instrument_attributes_producer: Producer<'static, InstrumentAttributes, 8>,
        instrument_attributes_consumer: Consumer<'static, InstrumentAttributes, 8>,
        _input_snapshot_producer: Producer<'static, InputSnapshot, 8>,
        input_snapshot_consumer: Consumer<'static, InputSnapshot, 8>,
        save_producer: Producer<'static, Save, 8>,
        _save_consumer: Consumer<'static, Save, 8>,
    }

    #[init(
        local = [
            instrument_attributes_queue: Queue<InstrumentAttributes, 8> = Queue::new(),
            input_snapshot_queue: Queue<InputSnapshot, 8> = Queue::new(),
            save_queue: Queue<Save, 8> = Queue::new(),
        ]
    )]
    fn init(cx: init::Context) -> (Shared, Local, init::Monotonics) {
        defmt::info!("Starting the firmware, initializing resources");

        let (instrument_attributes_producer, instrument_attributes_consumer) =
            cx.local.instrument_attributes_queue.split();
        let (input_snapshot_producer, input_snapshot_consumer) =
            cx.local.input_snapshot_queue.split();
        let (save_producer, save_consumer) = cx.local.save_queue.split();

        let system = System::init(cx.core, cx.device);
        let mono = system.mono;
        let mut audio_interface = system.audio_interface;
        let output_manager = system.output_manager;

        let instrument = Instrument::new(SAMPLE_RATE as f32);
        let controller = Controller::new();
        let version_indicator = VersionIndicator::new(BLINKS, system.status_led);

        defmt::info!("Initialization was completed, starting tasks");

        audio_interface.spawn();
        version_indicator_loop::spawn().unwrap();
        control_loop::spawn().unwrap();

        (
            Shared {},
            Local {
                version_indicator,
                audio_interface,
                instrument,
                controller,
                output_manager,
                instrument_attributes_producer,
                instrument_attributes_consumer,
                _input_snapshot_producer: input_snapshot_producer,
                input_snapshot_consumer,
                save_producer,
                _save_consumer: save_consumer,
            },
            init::Monotonics(mono),
        )
    }

    #[task(
        local = [
            controller,
            output_manager,
            instrument_attributes_producer,
            input_snapshot_consumer,
            save_producer,
        ],
        priority = 3,
    )]
    fn control_loop(cx: control_loop::Context) {
        control_loop::spawn_after(1.millis()).ok().unwrap();

        let controller = cx.local.controller;
        let output_manager = cx.local.output_manager;
        let instrument_attributes_producer = cx.local.instrument_attributes_producer;
        let input_snapshot_consumer = cx.local.input_snapshot_consumer;
        let save_producer = cx.local.save_producer;

        queue_utils::warn_about_capacity("input_snapshot", input_snapshot_consumer);

        if let Some(snapshot) = queue_utils::dequeue_last(input_snapshot_consumer) {
            let result = controller.apply_input_snapshot(snapshot);
            if let Some(save) = result.save {
                let _ = save_producer.enqueue(save);
            }
            let _ = instrument_attributes_producer.enqueue(result.instrument_attributes);
        }

        controller.tick();
        output_manager.set_state(&controller.desired_output_state());
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
    fn version_indicator_loop(cx: version_indicator_loop::Context) {
        let version_indicator = cx.local.version_indicator;
        let required_sleep = version_indicator.cycle();
        version_indicator_loop::spawn_after(required_sleep).unwrap();
    }
}
