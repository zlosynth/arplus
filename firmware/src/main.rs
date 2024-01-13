#![no_main]
#![no_std]

use arplus_firmware as _; // Global logger and panicking behavior.

#[rtic::app(device = stm32h7xx_hal::pac, peripherals = true, dispatchers = [EXTI0, EXTI1, EXTI2])]
mod app {
    use systick_monotonic::Systick;

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
    }

    #[init]
    fn init(cx: init::Context) -> (Shared, Local, init::Monotonics) {
        defmt::info!("Starting the firmware, initializing resources");

        let system = System::init(cx.core, cx.device);
        let mono = system.mono;

        let version_indicator = VersionIndicator::new(BLINKS, system.status_led);

        defmt::info!("Initialization was completed, starting tasks");

        version_indicator_alarm::spawn().unwrap();

        (
            Shared {},
            Local { version_indicator },
            init::Monotonics(mono),
        )
    }

    #[task(local = [version_indicator])]
    fn version_indicator_alarm(cx: version_indicator_alarm::Context) {
        let version_indicator = cx.local.version_indicator;
        let required_sleep = version_indicator.cycle();
        version_indicator_alarm::spawn_after(required_sleep).unwrap();
    }
}
