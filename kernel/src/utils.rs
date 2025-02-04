use core::cmp::min;

pub fn extract_bits<T>(value: T, shift: usize, width: usize) -> T
where
    T: TryFrom<u64> + From<u8>,
    u64: TryInto<T> + From<T>,
{
    let mask = (1u64 << min(63, width)) - 1;
    let value = u64::from(value);
    let value = value.checked_shr(shift as u32).unwrap_or(0) & mask;
    TryInto::try_into(value).unwrap_or_else(|_| T::from(0u8))
}
