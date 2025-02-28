use alloc::vec::Vec;
use spin::Mutex;
use x86_64::{structures::paging::PageTable, VirtAddr};

pub mod scheduler;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct ProcessId(u64);

#[derive(Debug, PartialEq, Eq)]
pub enum ProcessState {
    Ready,
    Ruuing,
    Blocked,
    Terminated,
}

pub struct Process {
    id: ProcessId,
    state: Mutex<ProcessState>,
    parent_id: Option<ProcessId>,
    page_table: Mutex<Option<&'static mut PageTable>>,
    stack_top: VirtAddr,
    context: Mutex<ProcessContext>,
    children: Mutex<Vec<ProcessId>>,
}

#[derive(Debug, Default)]
pub struct ProcessContext {
    // 汎用レジスタ
    pub rax: u64,
    pub rbx: u64,
    pub rcx: u64,
    pub rdx: u64,
    pub rsi: u64,
    pub rdi: u64,
    pub rbp: u64,
    pub rsp: u64,
    pub r8: u64,
    pub r9: u64,
    pub r10: u64,
    pub r11: u64,
    pub r12: u64,
    pub r13: u64,
    pub r14: u64,
    pub r15: u64,
    // 命令ポインタ
    pub rip: u64,
    // フラグレジスタ
    pub rflags: u64,
    // セグメントレジスタ
    pub cs: u64,
    pub ss: u64,
    pub ds: u64,
    pub es: u64,
    pub fs: u64,
    pub gs: u64,
    // CR3 (ページテーブルベースアドレス)
    pub cr3: u64,
}
