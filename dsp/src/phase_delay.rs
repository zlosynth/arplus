//! Phase delay approximation for state variable filter.
//!
//! When a signal passes through SVF, it is delayed. How much it is delayed
//! depends on the cutoff frequency and the Q factor. This module assists
//! in calculating that delay.

use super::phase_delay_lookup::*;

/// Calculate phase delay.
///
/// The output is relative to the input signal frequency. For example, if the input
/// signal is 440 Hz, and this function returns 0.05, the absolute delay in
/// seconds is then `(1 / 440) * 0.05`.
pub fn phase_delay(cutoff: f32, q: f32) -> f32 {
    if cutoff < TABLE_1_C_MIN {
        defmt::warn!("Phase delay under range for cutoff={:?}", cutoff);
        let x = 0.0;
        let y = (q - TABLE_1_Q_MIN) / (TABLE_1_Q_MAX - TABLE_1_Q_MIN);
        interpolate(TABLE_4, x, y)
    } else if (TABLE_1_C_MIN..=TABLE_1_C_MAX).contains(&cutoff)
        && (TABLE_1_Q_MIN..=TABLE_1_Q_MAX).contains(&q)
    {
        let x = (cutoff - TABLE_1_C_MIN) / (TABLE_1_C_MAX - TABLE_1_C_MIN);
        let y = (q - TABLE_1_Q_MIN) / (TABLE_1_Q_MAX - TABLE_1_Q_MIN);
        interpolate(TABLE_1, x, y)
    } else if (TABLE_2_C_MIN..=TABLE_2_C_MAX).contains(&cutoff)
        && (TABLE_2_Q_MIN..=TABLE_2_Q_MAX).contains(&q)
    {
        let x = (cutoff - TABLE_2_C_MIN) / (TABLE_2_C_MAX - TABLE_2_C_MIN);
        let y = (q - TABLE_2_Q_MIN) / (TABLE_2_Q_MAX - TABLE_2_Q_MIN);
        interpolate(TABLE_2, x, y)
    } else if (TABLE_3_C_MIN..=TABLE_3_C_MAX).contains(&cutoff)
        && (TABLE_3_Q_MIN..=TABLE_3_Q_MAX).contains(&q)
    {
        let x = (cutoff - TABLE_3_C_MIN) / (TABLE_3_C_MAX - TABLE_3_C_MIN);
        let y = (q - TABLE_3_Q_MIN) / (TABLE_3_Q_MAX - TABLE_3_Q_MIN);
        interpolate(TABLE_3, x, y)
    } else if (TABLE_4_C_MIN..=TABLE_4_C_MAX).contains(&cutoff)
        && (TABLE_4_Q_MIN..=TABLE_4_Q_MAX).contains(&q)
    {
        let x = (cutoff - TABLE_4_C_MIN) / (TABLE_4_C_MAX - TABLE_4_C_MIN);
        let y = (q - TABLE_4_Q_MIN) / (TABLE_4_Q_MAX - TABLE_4_Q_MIN);
        interpolate(TABLE_4, x, y)
    } else {
        defmt::warn!("Phase delay above range for cutoff={:?}", cutoff);
        let x = 1.0;
        let y = (q - TABLE_4_Q_MIN) / (TABLE_4_Q_MAX - TABLE_4_Q_MIN);
        interpolate(TABLE_4, x, y)
    }
}

/// Two dimensional linear interpolation.
fn interpolate<const X: usize, const Y: usize>(data: [[f32; X]; Y], x: f32, y: f32) -> f32 {
    let x_left_i = ((x * X as f32) as usize).min(X - 1);
    let x_right_i = (x_left_i + 1).min(X - 1);
    let x_p = libm::modff(x * X as f32).0;
    let y_top_i = ((y * Y as f32) as usize).min(Y - 1);
    let y_bottom_i = (y_top_i + 1).min(Y - 1);
    let y_top_p = libm::modff(y * Y as f32).0;

    let top_left = data[y_top_i][x_left_i];
    let top_right = data[y_top_i][x_right_i];
    let top = top_left + x_p * (top_right - top_left);

    let bottom_left = data[y_bottom_i][x_left_i];
    let bottom_right = data[y_bottom_i][x_right_i];
    let bottom = bottom_left + x_p * (bottom_right - bottom_left);

    top + (bottom - top) * y_top_p
}
