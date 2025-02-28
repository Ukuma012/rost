use alloc::collections::{btree_map::BTreeMap, vec_deque::VecDeque};
use conquer_once::spin::OnceCell;
use spin::Mutex;
use spinning_top::Spinlock;

use crate::println;

use super::{Process, ProcessId};

pub static SCHEDULER: OnceCell<Spinlock<Scheduler>> = OnceCell::uninit();

pub struct Scheduler {
    processes: Mutex<BTreeMap<ProcessId, Process>>,
    ready_queue: Mutex<VecDeque<ProcessId>>,
    current: Mutex<Option<ProcessId>>,
}

impl Scheduler {
    pub fn new() -> Self {
        Self {
            processes: Mutex::new(BTreeMap::new()),
            ready_queue: Mutex::new(VecDeque::new()),
            current: Mutex::new(None),
        }
    }

    // 新しいプロセスを作成し、ReadyQueueに追加
    pub fn create_process(&self, entry_point: u64, parent_id: Option<ProcessId>) -> ProcessId {
        // プロセス作成ロジック
        todo!()
    }

    // 次のプロセスを選択
    pub fn schedule(&self) -> Option<&Process> {
        // スケジューリングロジック
        todo!()
    }

    // Context Switchを実行
    pub fn context_switch(&self) {
        // 現在のプロセスのコンテキストを保存
        // 次のプロセスを選択
        // 新しいプロセスのコンテキストを復元
        println!("hi");
    }
}
