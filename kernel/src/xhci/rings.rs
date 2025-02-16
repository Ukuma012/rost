use core::{
    marker::PhantomPinned,
    ptr::{read_volatile, write_volatile},
};

use crate::memory::IoBox;

use super::trb::TrbBase;

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
