#![no_main]
#![no_std]

#[macro_use]
extern crate alloc;

use core::panic;

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

    println!("{:?}", buf);
}
