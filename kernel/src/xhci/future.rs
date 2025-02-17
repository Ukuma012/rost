use alloc::collections::vec_deque::VecDeque;
use spin::Mutex;

use super::trb::{TrbBase, TrbType};
pub struct EventWaitCond {
    trb_type: Option<TrbType>,
    trb_addr: Option<u64>,
    slot: Option<u8>,
}

pub struct EventWaitInfo {
    cond: EventWaitCond,
    trbs: Mutex<VecDeque<TrbBase>>,
}

impl EventWaitInfo {
    pub fn matches(&self, trb: &TrbBase) -> bool {
        if let Some(trb_type) = self.cond.trb_type {
            if trb.trb_type() != trb_type as u32 {
                return false;
            }
        }
        if let Some(slot) = self.cond.slot {
            if trb.slot_id() != slot {
                return false;
            }
        }
        if let Some(trb_addr) = self.cond.trb_addr {
            if trb.data() != trb_addr {
                return false;
            }
        }

        true
    }
}
