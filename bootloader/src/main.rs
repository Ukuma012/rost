#![no_main]
#![no_std]

#[macro_use]
extern crate alloc;

use boot::stall;
use boot::{AllocateType, MemoryType};
use common::elf::{Elf64, SegmentType};
use core::slice::from_raw_parts_mut;
use core::{panic, usize};
use log::info;
use uefi::println;
use uefi::proto::media::file::{File, FileInfo};
use uefi::{
    CStr16,
    prelude::*,
    proto::media::{
        file::{FileAttribute, FileMode, FileType, RegularFile},
        fs::SimpleFileSystem,
    },
};

const UEFI_PAGE_SIZE: usize = 4096;

#[entry]
fn efi_main() -> Status {
    uefi::helpers::init().unwrap();

    // load kernel
    let kernel_entry_point_addr = load_elf("kernel.elf");
    println!("entry_point: {}", kernel_entry_point_addr);
    stall(10_000_000);

    Status::SUCCESS
}

fn read_file(path: &str) -> RegularFile {
    let sfs_handle = boot::get_handle_for_protocol::<SimpleFileSystem>().unwrap();
    let mut root = boot::open_protocol_exclusive::<SimpleFileSystem>(sfs_handle)
        .unwrap()
        .open_volume()
        .unwrap();
    let mut buf = [0; 256];
    let path = CStr16::from_str_with_buf(path, &mut buf).unwrap();
    let file = root
        .open(path, FileMode::Read, FileAttribute::empty())
        .unwrap()
        .into_type()
        .unwrap();

    match file {
        FileType::Regular(file) => file,
        FileType::Dir(_) => panic!("Not a file: {}", path),
    }
}

fn load_elf(path: &str) -> u64 {
    let mut file = read_file(path);
    let file_info = file.get_boxed_info::<FileInfo>().unwrap();
    let file_size = file_info.file_size() as usize;
    let mut buf = vec![0; file_size];

    file.read(&mut buf).unwrap();

    // load elf
    let elf = Elf64::new(&buf).unwrap();

    let mut dest_start = usize::MAX;
    let mut dest_end: usize = 0;
    let phs = elf.program_headers();

    for p in &phs {
        if p.segment_type() != SegmentType::Load {
            continue;
        }

        dest_start = dest_start.min(p.virt_addr as usize);
        dest_end = dest_end.max((p.virt_addr + p.mem_size) as usize);
    }

    let pages = (dest_end - dest_start + UEFI_PAGE_SIZE - 1) / UEFI_PAGE_SIZE;

    boot::allocate_pages(
        AllocateType::Address(dest_start as u64),
        MemoryType::LOADER_DATA,
        pages,
    )
    .unwrap();

    for p in &phs {
        if p.segment_type() != SegmentType::Load {
            continue;
        }

        let offset = p.offset as usize;
        let file_size = p.file_size as usize;
        let mem_size = p.mem_size as usize;
        let dest = unsafe { from_raw_parts_mut(p.virt_addr as *mut u8, mem_size) };
        dest[..file_size].copy_from_slice(&buf[offset..offset + file_size]);
        dest[file_size..].fill(0);
    }

    info!("Loaded ELF at: 0x{:x}", dest_start);

    elf.header().entry_point
}
