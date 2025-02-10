// OSの主な役割の一つに、プログラムを互いに分離するということがある
// この目的を達成するために、OSはハードウェアの機能を利用してある
// あるプロセスのメモリ領域に他のプロセスがアクセスできないようにする
// ARM Cortex-Mプロセッサは、MPUがある
// x86ではセグメンテーションとページングの２つの異なるメモリ保護をサポートしている
// 仮想メモリ空間のブロックはページと呼ばれる
// 物理アドレス空間のブロックはフレームと呼ばれる
// ページとフレームの対応付けの情報は、ページテーブルによって保存される
// x86_64アーキテクチャは4層ページテーブルを使っており、ページサイズは4KiB
// それぞれのページテーブルは層によらず512のエントリを持っている
// それぞれのエントリの大きさは8バイトなので、それぞれのテーブルは512 * 8 = 4Kibでぴったり1ページに収まる
// ページテーブルを修正したときは毎回TLBをflushしないといけない

use x86_64::structures::paging::PageTable;
use x86_64::VirtAddr;

pub unsafe fn active_level_4_table(physical_memory_offset: VirtAddr) -> &'static mut PageTable {
    use x86_64::registers::control::Cr3;
    let (level_4_table_frame, _) = Cr3::read();

    let phys = level_4_table_frame.start_address();
    let virt = physical_memory_offset + phys.as_u64();
    let page_table_ptr: *mut PageTable = virt.as_mut_ptr();

    &mut *page_table_ptr
}
