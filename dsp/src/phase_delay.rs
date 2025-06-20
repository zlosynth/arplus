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
        interpolate(TABLE_1, x, y)
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
    // Clamp x and y to [0, 1] range
    let x = x.clamp(0.0, 1.0);
    let y = y.clamp(0.0, 1.0);

    let x_scaled = x * (X - 1) as f32;
    let x_left_i = libm::floorf(x_scaled) as usize;
    let x_right_i = (x_left_i + 1).min(X - 1);
    let x_p = x_scaled - libm::floorf(x_scaled);

    let y_scaled = y * (Y - 1) as f32;
    let y_top_i = libm::floorf(y_scaled) as usize;
    let y_bottom_i = (y_top_i + 1).min(Y - 1);
    let y_top_p = y_scaled - libm::floorf(y_scaled);

    let top_left = data[y_top_i][x_left_i];
    let top_right = data[y_top_i][x_right_i];
    let top = top_left + x_p * (top_right - top_left);

    let bottom_left = data[y_bottom_i][x_left_i];
    let bottom_right = data[y_bottom_i][x_right_i];
    let bottom = bottom_left + x_p * (bottom_right - bottom_left);

    top + (bottom - top) * y_top_p
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_phase_delay_below_minimum_cutoff() {
        let q = (TABLE_1_Q_MIN + TABLE_1_Q_MAX) / 2.0; // Midpoint of Q range
        let below_min_result = phase_delay(TABLE_1_C_MIN / 2.0, q);
        let min_result = phase_delay(TABLE_1_C_MIN, q);

        assert!(
            approx::relative_eq!(below_min_result, min_result),
            "Phase delay below minimum cutoff should match value at minimum cutoff (got {}, expected {})",
            below_min_result,
            min_result
        );
    }

    #[test]
    fn test_phase_delay_above_maximum_cutoff() {
        let q = (TABLE_4_Q_MIN + TABLE_4_Q_MAX) / 2.0; // Midpoint of Q range for TABLE_4
        let above_max_result = phase_delay(TABLE_4_C_MAX * 2.0, q);
        let max_result = phase_delay(TABLE_4_C_MAX, q);

        assert!(
            approx::relative_eq!(above_max_result, max_result),
            "Phase delay above maximum cutoff should match value at maximum cutoff (got {}, expected {})",
            above_max_result,
            max_result
        );
    }

    #[test]
    fn test_phase_delay_continuous_cutoff() {
        // Test that phase delay is continuous across cutoff for a fixed Q
        let cutoff_start = TABLE_1_C_MIN;
        let cutoff_end = TABLE_4_C_MAX;
        let q = (TABLE_1_Q_MIN + TABLE_1_Q_MAX) / 2.0;

        let steps = 10000;
        let step = (cutoff_end - cutoff_start) / steps as f32;

        let max_jump = 0.02;

        let mut prev_cutoff: Option<f32> = None;
        let mut prev_delay: Option<f32> = None;
        for i in 0..=steps {
            let cutoff = cutoff_start + i as f32 * step;
            let delay = phase_delay(cutoff, q);

            if let Some(prev_delay) = prev_delay {
                let jump = (delay - prev_delay).abs();

                assert!(
                    jump <= max_jump,
                    "Discontinuity detected at cutoff={}: jump of {} from {} to {}, from previous cutoff={}",
                    cutoff,
                    jump,
                    prev_delay,
                    delay,
                    prev_cutoff.unwrap_or(0.0),
                );
            }

            prev_cutoff = Some(cutoff);
            prev_delay = Some(delay);
        }
    }

    #[test]
    fn test_phase_delay_continuous_q() {
        // Test that phase delay is continuous across Q for a fixed cutoff
        let q_start = TABLE_1_Q_MIN;
        let q_end = TABLE_4_Q_MAX;
        let cutoff = (TABLE_1_C_MIN + TABLE_4_C_MAX) / 2.0;

        let steps = 100;
        let step = (q_end - q_start) / steps as f32;

        let max_jump = 0.1;

        let mut prev_q: Option<f32> = None;
        let mut prev_delay: Option<f32> = None;
        for i in 0..=steps {
            let q = q_start + i as f32 * step;
            let delay = phase_delay(cutoff, q);

            if let Some(prev_delay) = prev_delay {
                let jump = (delay - prev_delay).abs();

                assert!(
                    jump <= max_jump,
                    "Discontinuity detected at q={}: jump of {} from {} to {}, from previous q={}",
                    q,
                    jump,
                    prev_delay,
                    delay,
                    prev_q.unwrap_or(0.0),
                );
            }

            prev_q = Some(q);
            prev_delay = Some(delay);
        }
    }

    #[test]
    fn test_interpolate_between_values() {
        #[rustfmt::skip]
        let data = [
            [1.0, 3.0],
            [5.0, 7.0],
        ];

        let result = interpolate(data, 0.5, 0.5);
        approx::assert_relative_eq!(result, 4.0);
    }

    #[test]
    fn test_interpolate_edge_cases() {
        #[rustfmt::skip]
        let data = [
            [1.0, 2.0],
            [3.0, 4.0],
        ];

        let test_cases = [
            // (x, y, expected_value)
            (0.0, 0.0, 1.0),
            (1.0, 0.0, 2.0),
            (0.0, 1.0, 3.0),
            (1.0, 1.0, 4.0),
        ];

        for (x, y, expected) in test_cases {
            approx::assert_relative_eq!(interpolate(data, x, y), expected,);
        }
    }

    #[test]
    fn test_interpolate_out_of_bounds_clamping() {
        #[rustfmt::skip]
        let data = [
            [1.0, 2.0],
            [3.0, 4.0],
        ];

        let test_cases = [
            // (x, y, expected_value)
            (-0.5, 0.0, 1.0),
            (0.0, -0.5, 1.0),
            (1.5, 0.0, 2.0),
            (1.0, -0.5, 2.0),
            (-0.5, 1.0, 3.0),
            (0.0, 1.5, 3.0),
            (1.5, 1.0, 4.0),
            (1.0, 1.5, 4.0),
        ];

        for (x, y, expected) in test_cases {
            approx::assert_relative_eq!(interpolate(data, x, y), expected,);
        }
    }

    #[test]
    fn test_interpolate_single_point() {
        let data = [[1.0]];
        let result = interpolate(data, 0.5, 0.5);
        assert_eq!(
            result, 1.0,
            "Single point interpolation should return the point value"
        );
    }
}
