use core::{
    mem::MaybeUninit,
    ops::{BitAnd, BitOr, Not, Shl, Shr, Sub},
    ptr::{read_volatile, write_volatile},
};

#[repr(transparent)]
#[derive(Debug)]
pub struct Volatile<T> {
    value: T,
}
impl<T: Default> Default for Volatile<T> {
    fn default() -> Self {
        Self {
            value: T::default(),
        }
    }
}

impl<T: Clone> Clone for Volatile<T> {
    fn clone(&self) -> Self {
        let this = MaybeUninit::uninit();
        let mut this: Self = unsafe { this.assume_init() };
        this.write(self.read());
        this
    }
}

impl<T> Volatile<T> {
    pub fn read(&self) -> T {
        unsafe { read_volatile(&self.value) }
    }

    pub fn write(&mut self, new_value: T) {
        unsafe {
            write_volatile(&mut self.value, new_value);
        }
    }
}

impl<
        T: Shl<usize, Output = T>
            + Shr<usize, Output = T>
            + BitOr<Output = T>
            + BitAnd<Output = T>
            + Not<Output = T>
            + From<u8>
            + Sub<T, Output = T>
            + PartialEq<T>
            + Copy,
    > Volatile<T>
{
    pub fn write_bits(&mut self, shift: usize, width: usize, value: T) {
        let mask = (T::from(1) << width) - T::from(1);
        let mask = mask << shift;
        self.write((value << shift) | (self.read() & !mask));
    }

    pub fn read_bits(&self, shift: usize, width: usize) -> T {
        let mask = (T::from(1) << width) - T::from(1);
        (self.read() >> shift) & mask
    }
}
