use conquer_once::spin::OnceCell;
use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame};

use crate::println;

pub static IDT: OnceCell<InterruptDescriptorTable> = OnceCell::uninit();

pub fn init_idt() {
    IDT.init_once(|| {
        let mut idt = InterruptDescriptorTable::new();
        idt.breakpoint.set_handler_fn(breakpoint_handler);
        idt
    });

    IDT.get().unwrap().load();
}

extern "x86-interrupt" fn breakpoint_handler(stack_frame: InterruptStackFrame) {
    println!("Exception: BREAKOINT\n{:#?}", stack_frame);
}
