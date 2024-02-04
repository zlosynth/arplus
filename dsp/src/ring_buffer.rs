//! Write to and read from a ring buffer, keeping data in a static slice.

use core::fmt;
use core::ptr;
use core::sync::atomic;

use crate::math;
use crate::memory_manager::MemoryManager;

pub struct RingBuffer {
    buffer: &'static mut [f32],
    mask: usize,
    write_index: usize,
}

impl fmt::Debug for RingBuffer {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "RingBuffer(write_index: {})", self.write_index,)
    }
}

impl defmt::Format for RingBuffer {
    fn format(&self, fmt: defmt::Formatter) {
        defmt::write!(fmt, "RingBuffer(write_index: {})", self.write_index,);
    }
}

impl From<&'static mut [f32]> for RingBuffer {
    fn from(buffer: &'static mut [f32]) -> Self {
        assert!(is_power_of_2(buffer.len()));
        let mask = buffer.len() - 1;
        Self {
            buffer,
            mask,
            write_index: 0,
        }
    }
}

impl RingBuffer {
    pub fn new_from_memory_manager(memory_manager: &mut MemoryManager, size: usize) -> Self {
        let slice = memory_manager
            .allocate(math::upper_power_of_two(size))
            .unwrap();
        RingBuffer::from(slice)
    }

    pub fn write(&mut self, value: f32) {
        self.write_index = (self.write_index + 1) & self.mask;
        self.buffer[self.write_index] = value;
    }

    pub fn peek(&self, relative_index: usize) -> f32 {
        let index = self.write_index.wrapping_sub(relative_index) & self.mask;
        self.buffer[index]
    }

    pub fn peek_interpolated(&self, relative_index: f32) -> f32 {
        let index_a = libm::floorf(relative_index) as usize;
        let a = self.peek(index_a);

        let index_b = libm::ceilf(relative_index) as usize;
        let b = self.peek(index_b);

        let diff = b - a;
        let root = if relative_index < 0.0 { b } else { a };

        let fract = libm::modff(relative_index).0;
        root + diff * fract
    }

    pub fn reset(&mut self) {
        for x in self.buffer.iter_mut() {
            unsafe {
                ptr::write_volatile(x, 0.0);
            }
            atomic::compiler_fence(atomic::Ordering::SeqCst);
        }
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
    use core::mem::MaybeUninit;

    #[test]
    fn check_power_of_2() {
        assert!(is_power_of_2(1));
        assert!(is_power_of_2(2));
        assert!(is_power_of_2(8));
        assert!(is_power_of_2(1024));

        assert!(!is_power_of_2(3));
        assert!(!is_power_of_2(10));
    }

    #[test]
    #[should_panic]
    fn initialize_buffer_with_invalid_size() {
        static mut MEMORY: [MaybeUninit<u32>; 16] = unsafe { MaybeUninit::uninit().assume_init() };
        let mut memory_manager = MemoryManager::from(unsafe { &mut MEMORY[..] });
        let slice = memory_manager.allocate(3).unwrap();
        let _buffer = RingBuffer::from(slice);
    }

    #[test]
    fn initialize_buffer() {
        static mut MEMORY: [MaybeUninit<u32>; 16] = unsafe { MaybeUninit::uninit().assume_init() };
        let mut memory_manager = MemoryManager::from(unsafe { &mut MEMORY[..] });

        let slice = memory_manager.allocate(8).unwrap();
        let _buffer = RingBuffer::from(slice);
    }

    #[test]
    fn write_to_buffer() {
        static mut MEMORY: [MaybeUninit<u32>; 16] = unsafe { MaybeUninit::uninit().assume_init() };
        let mut memory_manager = MemoryManager::from(unsafe { &mut MEMORY[..] });

        let slice = memory_manager.allocate(8).unwrap();
        let mut buffer = RingBuffer::from(slice);

        buffer.write(1.0);
    }

    #[test]
    fn read_from_buffer() {
        static mut MEMORY: [MaybeUninit<u32>; 16] = unsafe { MaybeUninit::uninit().assume_init() };
        let mut memory_manager = MemoryManager::from(unsafe { &mut MEMORY[..] });

        let slice = memory_manager.allocate(8).unwrap();
        let mut buffer = RingBuffer::from(slice);

        buffer.write(1.0);
        buffer.write(2.0);
        buffer.write(3.0);

        assert_relative_eq!(buffer.peek(0), 3.0);
        assert_relative_eq!(buffer.peek(1), 2.0);
        assert_relative_eq!(buffer.peek(2), 1.0);
    }

    #[test]
    fn follow_reads_and_writes_throughout_the_buffer() {
        static mut MEMORY: [MaybeUninit<u32>; 4] = unsafe { MaybeUninit::uninit().assume_init() };
        let mut memory_manager = MemoryManager::from(unsafe { &mut MEMORY[..] });

        let slice = memory_manager.allocate(4).unwrap();
        let mut buffer = RingBuffer::from(slice);

        buffer.write(1.0);
        assert_eq!(buffer.peek(0) as usize, 1);

        buffer.write(2.0);
        assert_eq!(buffer.peek(0) as usize, 2);
        assert_eq!(buffer.peek(1) as usize, 1);

        buffer.write(3.0);
        assert_eq!(buffer.peek(0) as usize, 3);
        assert_eq!(buffer.peek(1) as usize, 2);
        assert_eq!(buffer.peek(2) as usize, 1);

        buffer.write(4.0);
        assert_eq!(buffer.peek(0) as usize, 4);
        assert_eq!(buffer.peek(1) as usize, 3);
        assert_eq!(buffer.peek(2) as usize, 2);
        assert_eq!(buffer.peek(3) as usize, 1);

        buffer.write(5.0);
        assert_eq!(buffer.peek(0) as usize, 5);
        assert_eq!(buffer.peek(1) as usize, 4);
        assert_eq!(buffer.peek(2) as usize, 3);
        assert_eq!(buffer.peek(3) as usize, 2);

        buffer.write(6.0);
        assert_eq!(buffer.peek(0) as usize, 6);
        assert_eq!(buffer.peek(1) as usize, 5);
        assert_eq!(buffer.peek(2) as usize, 4);
        assert_eq!(buffer.peek(3) as usize, 3);
    }
}
