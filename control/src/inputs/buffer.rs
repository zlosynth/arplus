//! Smoothening of control data and travel distance measurement.

/// Buffer meant for smoothening and history tracking.
///
/// This is not optimized for large buffers, but should be ok for smoothening
/// and travel distance measuring on input control voltage to up to 32 samples.
#[derive(Debug, defmt::Format)]
pub struct Buffer<const N: usize> {
    buffer: [f32; N],
    pointer: usize,
    mask: usize,
}

impl<const N: usize> Default for Buffer<N> {
    fn default() -> Self {
        Self::new()
    }
}

impl<const N: usize> Buffer<N> {
    /// # Panics
    ///
    /// The size of the buffer `N` must be a power of 2. Initialization
    /// fails otherwise.
    #[must_use]
    pub fn new() -> Self {
        assert!(is_power_of_2(N));
        let mask = N - 1;
        Self {
            buffer: [0.0; N],
            pointer: 0,
            mask,
        }
    }

    pub fn write(&mut self, value: f32) {
        self.buffer[self.pointer] = value;
        self.pointer = self.pointer.wrapping_add(1) & self.mask;
    }

    #[must_use]
    pub fn read_raw(&self) -> f32 {
        let newest = self.pointer.wrapping_sub(1) & self.mask;
        self.buffer[newest]
    }

    #[must_use]
    pub fn traveled(&self) -> f32 {
        let newest = self.pointer.wrapping_sub(1) & self.mask;
        let oldest = self.pointer;
        self.buffer[newest] - self.buffer[oldest]
    }
}

fn is_power_of_2(n: usize) -> bool {
    if n == 1 {
        return true;
    } else if n % 2 != 0 || n == 0 {
        return false;
    }

    is_power_of_2(n / 2)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn when_reads_raw_it_returns_last() {
        let mut buffer: Buffer<4> = Buffer::new();
        buffer.write(1.0);
        buffer.write(2.0);
        assert_relative_eq!(buffer.read_raw(), 2.0);
    }

    #[test]
    fn when_measures_traveled_it_returns_distance_from_oldest() {
        let mut buffer: Buffer<4> = Buffer::new();

        buffer.write(1.0);
        buffer.write(2.0);
        buffer.write(3.0);
        buffer.write(4.0);
        assert_relative_eq!(buffer.traveled(), 3.0);

        buffer.write(4.0);
        buffer.write(3.0);
        buffer.write(2.0);
        buffer.write(1.0);
        assert_relative_eq!(buffer.traveled(), -3.0);
    }
}
