#![no_main]
#![no_std]

use arplus_firmware as _; // Global logger and panicking behavior.

#[rtic::app(device = stm32h7xx_hal::pac, peripherals = true, dispatchers = [EXTI0, EXTI1, EXTI2])]
mod app {
    use fugit::ExtU64;
    use heapless::spsc::{Consumer, Producer, Queue};
    use systick_monotonic::Systick;

    use arplus_control::save::Save;
    use arplus_control::{ControlInputSnapshot, Controller};
    use arplus_dsp::{Attributes as InstrumentAttributes, Instrument};
    use arplus_firmware::audio::{AudioInterface, SAMPLE_RATE};
    use arplus_firmware::control_input::ControlInputInterface;
    use arplus_firmware::control_output::ControlOutputInterface;
    use arplus_firmware::flash_memory::FlashMemoryInterface;
    use arplus_firmware::queue_utils;
    use arplus_firmware::startup_sequence;
    use arplus_firmware::system::System;
    use arplus_firmware::version_indicator::VersionIndicator;

    // Blinks on the PCB's LED signalize the current revision.
    const BLINKS: u8 = 1;

    // 1 kHz granularity for task scheduling.
    #[monotonic(binds = SysTick, default = true)]
    type Mono = Systick<1000>;

    #[shared]
    struct Shared {
        save_cache: Option<Save>,
    }

    #[local]
    struct Local {
        audio_interface: AudioInterface,
        flash_memory_interface: FlashMemoryInterface,
        control_input_interface: ControlInputInterface,
        control_output_interface: ControlOutputInterface,
        instrument: Instrument,
        controller: Controller,
        version_indicator: VersionIndicator,
        instrument_attributes_producer: Producer<'static, InstrumentAttributes, 8>,
        instrument_attributes_consumer: Consumer<'static, InstrumentAttributes, 8>,
        control_input_snapshot_producer: Producer<'static, ControlInputSnapshot, 8>,
        control_input_snapshot_consumer: Consumer<'static, ControlInputSnapshot, 8>,
        save_producer: Producer<'static, Save, 8>,
        save_consumer: Consumer<'static, Save, 8>,
    }

    #[init(
        local = [
            instrument_attributes_queue: Queue<InstrumentAttributes, 8> = Queue::new(),
            input_snapshot_queue: Queue<ControlInputSnapshot, 8> = Queue::new(),
            save_queue: Queue<Save, 8> = Queue::new(),
        ]
    )]
    fn init(cx: init::Context) -> (Shared, Local, init::Monotonics) {
        defmt::info!("Starting the firmware, initializing resources");

        let (instrument_attributes_producer, instrument_attributes_consumer) =
            cx.local.instrument_attributes_queue.split();
        let (control_input_snapshot_producer, control_input_snapshot_consumer) =
            cx.local.input_snapshot_queue.split();
        let (save_producer, save_consumer) = cx.local.save_queue.split();

        let system = System::init(cx.core, cx.device);
        let mono = system.mono;
        let mut audio_interface = system.audio_interface;
        let mut flash_memory_interface = system.flash_memory_interface;
        let mut control_input_interface = system.control_input_interface;
        let control_output_interface = system.control_output_interface;

        startup_sequence::warm_up_control_input(&mut control_input_interface);
        let save = startup_sequence::retrieve_save(
            &mut control_input_interface,
            &mut flash_memory_interface,
        );
        let controller = Controller::from(save);
        let instrument = Instrument::new(SAMPLE_RATE as f32);
        let version_indicator = VersionIndicator::new(BLINKS, system.status_led);

        defmt::info!("Spawning tasks");

        audio_interface.spawn();
        version_indicator_loop::spawn().unwrap();
        control_loop::spawn().unwrap();
        input_collection_loop::spawn().unwrap();
        save_caching_loop::spawn().unwrap();
        save_pacing_loop::spawn().unwrap();

        (
            Shared { save_cache: None },
            Local {
                audio_interface,
                flash_memory_interface,
                control_input_interface,
                control_output_interface,
                instrument,
                controller,
                version_indicator,
                instrument_attributes_producer,
                instrument_attributes_consumer,
                control_input_snapshot_producer,
                control_input_snapshot_consumer,
                save_producer,
                save_consumer,
            },
            init::Monotonics(mono),
        )
    }

    #[task(
        local = [
            control_input_interface,
            control_input_snapshot_producer,
        ],
        priority = 2,
    )]
    fn input_collection_loop(cx: input_collection_loop::Context) {
        input_collection_loop::spawn_after(1.millis()).ok().unwrap();

        let control_input_interface = cx.local.control_input_interface;
        let control_input_snapshot_producer = cx.local.control_input_snapshot_producer;

        control_input_interface.sample();

        let _ = control_input_snapshot_producer.enqueue(control_input_interface.snapshot());
    }

    #[task(
        local = [
            controller,
            control_output_interface,
            instrument_attributes_producer,
            control_input_snapshot_consumer,
            save_producer,
        ],
        priority = 3,
    )]
    fn control_loop(cx: control_loop::Context) {
        control_loop::spawn_after(1.millis()).ok().unwrap();

        let controller = cx.local.controller;
        let control_output_interface = cx.local.control_output_interface;
        let instrument_attributes_producer = cx.local.instrument_attributes_producer;
        let control_input_snapshot_consumer = cx.local.control_input_snapshot_consumer;
        let save_producer = cx.local.save_producer;

        queue_utils::warn_about_capacity("input_snapshot", control_input_snapshot_consumer);

        if let Some(snapshot) = queue_utils::dequeue_last(control_input_snapshot_consumer) {
            let result = controller.apply_input_snapshot(snapshot);
            if let Some(save) = result.save {
                let _ = save_producer.enqueue(save);
            }
            let _ = instrument_attributes_producer.enqueue(result.instrument_attributes);
        }

        controller.tick();
        control_output_interface.set_state(&controller.desired_output_state());
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

    #[task(
        local = [
            save_consumer,
        ],
        shared = [
            save_cache,
        ],
        priority = 3,
    )]
    fn save_caching_loop(cx: save_caching_loop::Context) {
        save_caching_loop::spawn_after(1.millis()).ok().unwrap();

        let save_consumer = cx.local.save_consumer;
        let mut save_cache = cx.shared.save_cache;

        queue_utils::warn_about_capacity("save_consumer", save_consumer);

        if let Some(save) = queue_utils::dequeue_last(save_consumer) {
            save_cache.lock(|save_cache| {
                *save_cache = Some(save);
            });
        }
    }

    #[task(
        shared = [
            save_cache,
        ],
        priority = 3,
    )]
    fn save_pacing_loop(mut cx: save_pacing_loop::Context) {
        save_pacing_loop::spawn_after(1.secs()).ok().unwrap();

        cx.shared.save_cache.lock(|save_cache| {
            if let Some(save) = save_cache.take() {
                save_coroutine::spawn(save)
                    .unwrap_or_else(|_| defmt::warn!("Failed issuing store request"));
            }
        });
    }

    #[task(
        local = [
            flash_memory_interface
        ]
    )]
    fn save_coroutine(cx: save_coroutine::Context, save: Save) {
        let flash_memory_interface = cx.local.flash_memory_interface;
        flash_memory_interface.save(save);
    }

    #[task(
        local = [
            version_indicator
        ]
    )]
    fn version_indicator_loop(cx: version_indicator_loop::Context) {
        let version_indicator = cx.local.version_indicator;
        let required_sleep = version_indicator.cycle();
        version_indicator_loop::spawn_after(required_sleep).unwrap();
    }
}
