use core::mem::MaybeUninit;

use arplus_dsp::{Attributes, Dsp, MemoryManager, Random, StereoMode, TriggerAttributes};
use proptest::prelude::*;

struct TestRandom;

impl Random for TestRandom {
    fn normal(&mut self) -> f32 {
        use rand::prelude::*;
        let mut rng = rand::rng();
        rng.random()
    }
}

fn check_stability(
    resonance: f32,
    cutoff: f32,
    frequency: f32,
    contour: f32,
    pluck: f32,
    is_root: bool,
    strings: usize,
    chord_size: usize,
    trigger_interval: f32,
    width: f32,
) {
    const SECONDS: f32 = 30.0;
    const SAMPLE_RATE: f32 = 96_000.0;
    const CONTROL_RATE: f32 = 1_000.0;

    static mut MEMORY: [MaybeUninit<u32>; 96 * 1024] =
        unsafe { MaybeUninit::uninit().assume_init() };
    let mut memory_manager = MemoryManager::from(unsafe { &mut MEMORY[..] });

    let mut dsp = Dsp::new(SAMPLE_RATE, &mut memory_manager);
    let trigger = TriggerAttributes {
        frequency,
        contour,
        pluck,
        is_root,
    };
    let attributes = Attributes {
        resonance,
        cutoff,
        trigger: None,
        gain: 1.0,
        chord_size,
        width,
        stereo_mode: StereoMode::Haas,
        strings,
    };

    let mut buffer = [(0.0, 0.0); 64];

    let buffers = ((SAMPLE_RATE * SECONDS) / buffer.len() as f32) as usize;
    let control_every_x_buffers = ((SAMPLE_RATE / buffer.len() as f32) / CONTROL_RATE) as usize;
    let trigger_every_x_controls = (CONTROL_RATE * trigger_interval) as usize;

    for i in 0..buffers {
        if i % control_every_x_buffers == 0 {
            let mut attributes = attributes.clone();
            if i % trigger_every_x_controls == 0 {
                attributes.trigger = Some(trigger);
            }
            dsp.set_attributes(attributes, &mut TestRandom);
        }

        let input_connected = false;
        dsp.process(&mut buffer, input_connected, &mut TestRandom);

        for (x1, x2) in buffer.iter() {
            if x1.is_nan() || x2.is_nan() {
                panic!("The output buffer contains NaN");
            }
        }
    }
}

#[test]
#[ignore]
fn stability_with_high_cutoff() {
    let resonance = 0.5;
    let cutoff = 0.99;
    let frequency = 2224.2874;
    let contour = 0.4;
    let pluck = 0.99;
    let is_root = false;
    let chord_size = 6;
    let trigger_interval = 2.965139;
    let width = 0.0;
    let strings = 8;
    check_stability(
        resonance,
        cutoff,
        frequency,
        contour,
        pluck,
        is_root,
        strings,
        chord_size,
        trigger_interval,
        width,
    );
}

proptest! {
    #[test]
    #[ignore]
    fn stability_proptest(
        resonance in 0.0..=1.0f32,
        cutoff in 0.0..=1.0f32,
        frequency in 16.0..3951.0f32,
        contour in 0.0..=1.0f32,
        pluck in 0.0..=1.0f32,
        is_root: bool,
        strings in 1..=8usize,
        chord_size in 0..=7usize,
        trigger_interval in 0.1..3.0f32,
        width in 0.0..=1.0f32,
    ) {
        check_stability(
            resonance,
            cutoff,
            frequency,
            contour,
            pluck,
            is_root,
            strings,
            chord_size,
            trigger_interval,
            width,
        );
    }
}
