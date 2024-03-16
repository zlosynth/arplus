use arplus_control::Save;

use crate::control_input::ControlInputInterface;
use crate::flash_memory::FlashMemoryInterface;

const RESET_BUTTON: usize = 3;

pub fn warm_up_control_input(control_input_interface: &mut ControlInputInterface) {
    for _ in 0..100 {
        control_input_interface.sample();
    }
}

pub fn retrieve_save(
    control_input_interface: &mut ControlInputInterface,
    flash_memory_interface: &mut FlashMemoryInterface,
) -> Save {
    if is_reset_requested(control_input_interface) {
        defmt::info!("Reset was initiated");
        let save = Save::default();
        flash_memory_interface.save(save);
        save
    } else {
        flash_memory_interface.load()
    }
}

fn is_reset_requested(control_input_interface: &mut ControlInputInterface) -> bool {
    if is_button_held(control_input_interface) {
        wait_until_button_is_released(control_input_interface);
        true
    } else {
        false
    }
}

fn is_button_held(control_input_interface: &mut ControlInputInterface) -> bool {
    control_input_interface.snapshot().buttons[RESET_BUTTON]
}

fn wait_until_button_is_released(control_input_interface: &mut ControlInputInterface) {
    loop {
        control_input_interface.sample();
        if !control_input_interface.snapshot().buttons[RESET_BUTTON] {
            break;
        }
    }
}
