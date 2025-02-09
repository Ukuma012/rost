/// x86_64アーキテクチャは例外発生時に予め定義されている
/// 既知の正常なスタックに切り替えることができる
use conquer_once::spin::OnceCell;
use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame};

use crate::{gdt, println};

pub static IDT: OnceCell<InterruptDescriptorTable> = OnceCell::uninit();

pub fn init_idt() {
    IDT.init_once(|| {
        let mut idt = InterruptDescriptorTable::new();
        idt.breakpoint.set_handler_fn(breakpoint_handler);
        unsafe {
            idt.double_fault
                .set_handler_fn(double_fault_handler)
                .set_stack_index(gdt::DOUBLE_FAULT_IST_INDEX);
        }
        idt
    });

    IDT.get().unwrap().load();
}

extern "x86-interrupt" fn breakpoint_handler(stack_frame: InterruptStackFrame) {
    println!("Exception: BREAKOINT\n{:#?}", stack_frame);
}

/// ダブルフォルト例外は直前の(1度目の)例外ハンドラの処理中に
/// ２度目の例外が発生したとき起きうる
extern "x86-interrupt" fn double_fault_handler(
    stack_frame: InterruptStackFrame,
    _error_code: u64,
) -> ! {
    panic!("EXCEPTION: DOUBLE FAULT\n{:#?}", stack_frame);
}
