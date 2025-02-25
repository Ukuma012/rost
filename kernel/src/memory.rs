use core::marker::PhantomPinned;
use core::mem::MaybeUninit;
use core::pin::Pin;

use alloc::boxed::Box;
use bootloader_api::info::{MemoryRegionKind, MemoryRegions};
use x86_64::structures::paging::{FrameAllocator, OffsetPageTable, PageTable, PhysFrame, Size4KiB};
use x86_64::{PhysAddr, VirtAddr};

use crate::println;

pub struct BootInfoFrameAllocator {
    memory_regions: &'static MemoryRegions,
    next: usize,
}

impl BootInfoFrameAllocator {
    pub unsafe fn init(memory_regions: &'static MemoryRegions) -> Self {
        BootInfoFrameAllocator {
            memory_regions,
            next: 0,
        }
    }

    fn usable_frames(&self) -> impl Iterator<Item = PhysFrame> {
        let regions = self.memory_regions.iter();
        let usable_regions = regions.filter(|region| region.kind == MemoryRegionKind::Usable);
        let addr_ranges = usable_regions.map(|region| region.start..region.end);
        let frame_addresses = addr_ranges.flat_map(|r| r.step_by(4096));
        frame_addresses.map(|addr| PhysFrame::containing_address(PhysAddr::new(addr)))
    }
}

unsafe impl FrameAllocator<Size4KiB> for BootInfoFrameAllocator {
    fn allocate_frame(&mut self) -> Option<PhysFrame<Size4KiB>> {
        let frame = self.usable_frames().nth(self.next);
        self.next += 1;
        frame
    }
}

unsafe fn active_level_4_table(physical_memory_offset: VirtAddr) -> &'static mut PageTable {
    use x86_64::registers::control::Cr3;

    let (level_4_page_frame, _) = Cr3::read();

    let phys = level_4_page_frame.start_address();
    let virt = physical_memory_offset + phys.as_u64();
    let page_table_ptr: *mut PageTable = virt.as_mut_ptr();

    &mut *page_table_ptr // deref of raw pointer, scary unsafe
}

pub unsafe fn init(physical_memory_offset: VirtAddr) -> OffsetPageTable<'static> {
    let level_4_table = active_level_4_table(physical_memory_offset);
    OffsetPageTable::new(level_4_table, physical_memory_offset)
}

#[repr(align(4096))]
pub struct IoBoxInner<T: Sized> {
    data: T,
    _pinned: PhantomPinned,
}

impl<T: Sized> IoBoxInner<T> {
    pub fn new(data: T) -> Self {
        Self {
            data,
            _pinned: PhantomPinned,
        }
    }
}

pub struct IoBox<T: Sized> {
    inner: Pin<Box<IoBoxInner<T>>>,
}

impl<T: Sized> IoBox<T> {
    pub fn new() -> Self {
        let inner = Box::pin(IoBoxInner::new(unsafe {
            MaybeUninit::<T>::zeroed().assume_init()
        }));
        let this = Self { inner };
        // disable_cache(&this)
        this
    }

    pub unsafe fn get_unchecked_mut(&mut self) -> &mut T {
        &mut self.inner.as_mut().get_unchecked_mut().data
    }
}

impl<T> AsRef<T> for IoBox<T> {
    fn as_ref(&self) -> &T {
        &self.inner.as_ref().get_ref().data
    }
}

impl<T: Sized> Default for IoBox<T> {
    fn default() -> Self {
        Self::new()
    }
}
