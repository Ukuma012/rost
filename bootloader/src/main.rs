#![no_main]
#![no_std]

#[macro_use]
extern crate alloc;

use core::{panic, usize};

use goblin::elf::Elf;
use goblin::elf64::program_header;
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

#[entry]
fn efi_main() -> Status {
    uefi::helpers::init().unwrap();

    let _kernel_entry_point_addr = load_elf("kernel.elf");
    boot::stall(10_000_000);
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

fn load_elf(path: &str) {
    let mut file = read_file(path);
    let file_info = file.get_boxed_info::<FileInfo>().unwrap();
    let file_size = file_info.file_size() as usize;
    let mut buf = vec![0; file_size];

    file.read(&mut buf).unwrap();

    // load elf
    let elf = Elf::parse(&buf).unwrap();

    let mut dest_start = usize::MAX;
    let mut dest_end = 0;
    for ph in elf.program_headers.iter() {
        println!(
            "Program header: {} {} {} {} {}",
            program_header::pt_to_str(ph.p_type),
            ph.p_offset,
            ph.p_vaddr,
            ph.p_paddr,
            ph.p_memsz
        );
        if ph.p_type != program_header::PT_LOAD {
            continue;
        }
        dest_start = dest_start.min(ph.p_paddr as usize);
        dest_end = dest_end.max(ph.p_paddr + ph.p_memsz);
    }
}
