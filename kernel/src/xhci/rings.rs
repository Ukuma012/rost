use core::{
    alloc::{GlobalAlloc, Layout},
    marker::PhantomPinned,
    ptr::{null_mut, read_volatile, write_volatile},
};

use spin::mutex::Mutex;

use crate::{allocator::ALLOCATOR, memory::IoBox};

use super::trb::{NormalTrb, TrbBase, TrbType};

#[repr(C, align(4096))]
pub struct TrbRing {
    trb: [TrbBase; Self::NUM_TRB],
    current_index: usize,
    _pinned: PhantomPinned,
}

impl TrbRing {
    pub const NUM_TRB: usize = 16;
    fn new() -> IoBox<Self> {
        IoBox::new()
    }

    fn reset(&mut self) {
        self.current_index = 0;
        for trb in self.trb.iter_mut() {
            trb.set_cycle_bit_state(false);
        }
    }

    pub fn phys_addr(&self) -> u64 {
        &self.trb[0] as *const TrbBase as u64
    }

    pub const fn num_trbs(&self) -> usize {
        Self::NUM_TRB
    }

    fn current(&self) -> TrbBase {
        self.trb(self.current_index)
    }

    fn trb(&self, index: usize) -> TrbBase {
        unsafe { read_volatile(&self.trb[index]) }
    }

    fn advance_index(&mut self, new_cycle: bool) {
        if self.current().cycle_bit_state() == new_cycle {
            panic!("cycle state does not change")
        }
        self.trb[self.current_index].set_cycle_bit_state(new_cycle);
        self.current_index = (self.current_index + 1) % self.trb.len();
    }

    fn advance_index_notoggle(&mut self, cycle_ours: bool) {
        if self.current().cycle_bit_state() != cycle_ours {
            panic!("cycle state mismatch")
        }
        self.current_index = (self.current_index + 1) % self.trb.len();
    }

    fn current_index(&self) -> usize {
        self.current_index
    }

    fn current_ptr(&self) -> usize {
        &self.trb[self.current_index] as *const TrbBase as usize
    }

    fn trb_ptr(&self, index: usize) -> usize {
        &self.trb[index] as *const TrbBase as usize
    }

    fn write(&mut self, index: usize, trb: TrbBase) {
        if index < self.trb.len() {
            unsafe {
                write_volatile(&mut self.trb[index], trb);
            }
        } else {
            panic!("TrbRing Out of Range")
        }
    }

    fn write_current(&mut self, trb: TrbBase) {
        self.write(self.current_index, trb);
    }
}

/// xHCに指示を出すためのリングバッファ
/// ソフトウェアがTRBを追加し、xHCがそれを読み取る
pub struct CommandRing {
    ring: IoBox<TrbRing>,
    cycle_state_ours: bool,
}

impl Default for CommandRing {
    fn default() -> Self {
        let mut this = Self {
            ring: TrbRing::new(),
            cycle_state_ours: false,
        };
        let link_trb = TrbBase::trb_link(this.ring.as_ref());
        unsafe { this.ring.get_unchecked_mut() }.write(TrbRing::NUM_TRB - 1, link_trb);
        this
    }
}

impl CommandRing {
    pub fn reset(&mut self) {
        self.cycle_state_ours = false;
        let ring = unsafe { self.ring.get_unchecked_mut() };
        ring.reset();
    }

    pub fn ring_phys_addr(&self) -> u64 {
        self.ring.as_ref() as *const TrbRing as u64
    }
}

/// USBデバイスとソフトウェアの間で
/// データを送受信するためのリングバッファ
/// ソフトウェアがTRBを追加し、xHCがそれを読み取る
pub struct TransferRingInner {
    ring: IoBox<TrbRing>,
    cycle_state_ours: bool,
    dequeue_index: usize,
    buffers: [*mut u8; TrbRing::NUM_TRB - 1],
}

impl TransferRingInner {
    const BUF_SIZE: usize = 4096;
    const BUF_ALIGN: usize = 4096;
    pub fn new(transfer_size: usize) -> Self {
        let mut this = Self {
            ring: TrbRing::new(),
            cycle_state_ours: false,
            dequeue_index: 0,
            buffers: [null_mut(); TrbRing::NUM_TRB - 1],
        };

        let link_trb = TrbBase::trb_link(this.ring.as_ref());
        let num_trbs = this.ring.as_ref().num_trbs();
        let mut_ring = unsafe { this.ring.get_unchecked_mut() };
        mut_ring.write(num_trbs - 1, link_trb);
        for (i, v) in this.buffers.iter_mut().enumerate() {
            let layout = Layout::from_size_align(Self::BUF_SIZE, Self::BUF_ALIGN).unwrap();

            *v = unsafe { ALLOCATOR.alloc(layout) };

            if v.is_null() {
                panic!("TransfeRing buffer allocation failed")
            }

            mut_ring.write(i, NormalTrb::new(*v, transfer_size as u16).into());
        }
        this
    }

    pub fn fill_ring(&mut self) {
        loop {
            let next_enqueue_index =
                (self.ring.as_ref().current_index() + 1) % (self.ring.as_ref().num_trbs() - 1);
            if next_enqueue_index == self.ring.as_ref().num_trbs() - 4 {
                // Ring is full
                break;
            }
            let mut_ring = unsafe { self.ring.get_unchecked_mut() };
            mut_ring.advance_index(!self.cycle_state_ours);
        }
    }

    pub fn dequeue_trb(&mut self, trb_ptr: usize) {
        let trb_ptr_expected = self.ring.as_ref().trb_ptr(self.dequeue_index);
        if trb_ptr_expected != trb_ptr {
            panic!("expected ptr does not match!")
        }
        let mut_ring = unsafe { self.ring.get_unchecked_mut() };

        self.dequeue_index += 1;
        if self.dequeue_index == mut_ring.num_trbs() - 1 {
            self.dequeue_index = 0;
        }

        mut_ring.advance_index(!self.cycle_state_ours);
        if mut_ring.current().trb_type() == TrbType::Link as u32 {
            mut_ring.advance_index(!self.cycle_state_ours);
            self.cycle_state_ours = !self.cycle_state_ours;
        }
    }

    pub fn current(&self) -> TrbBase {
        self.ring.as_ref().current()
    }

    pub fn ring_phys_addr(&self) -> u64 {
        self.ring.as_ref() as *const TrbRing as u64
    }
}

pub struct TransferRing {
    inner: Mutex<TransferRingInner>,
}

impl TransferRing {
    pub fn new(transfer_size: usize) -> Self {
        let inner = TransferRingInner::new(transfer_size);
        let inner = Mutex::new(inner);
        Self { inner }
    }

    pub fn fill_ring(&self) {
        self.inner.lock().fill_ring();
    }

    pub fn dequeue_trb(&self, trb_ptr: usize) {
        self.inner.lock().dequeue_trb(trb_ptr);
    }

    pub fn current(&self) -> TrbBase {
        self.inner.lock().current()
    }

    pub fn ring_phys_addr(&self) -> u64 {
        self.inner.lock().ring_phys_addr()
    }
}
