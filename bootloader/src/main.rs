#![no_main]
#![no_std]

#[macro_use]
extern crate alloc;

use boot::stall;
use boot::{AllocateType, MemoryType};
use common::graphic_info;
use common::graphic_info::GraphicInfo;
use common::mem_desc;
use core::{panic, slice, usize};
use goblin::elf::Elf;
use goblin::elf64::program_header;
use log::info;
use uefi::mem::memory_map::MemoryAttribute;
use uefi::println;
use uefi::proto::console::gop::GraphicsOutput;
use uefi::proto::console::gop::PixelFormat;
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
    // let graphic_info = init_graphic((800, 600));

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
    let elf = Elf::parse(&buf).unwrap();

    let mut dest_start = usize::MAX;
    let mut dest_end: usize = 0;
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
        dest_end = dest_end.max((ph.p_paddr + ph.p_memsz) as usize);
    }

    let pages = (dest_end - dest_start + UEFI_PAGE_SIZE - 1) / UEFI_PAGE_SIZE;
    boot::allocate_pages(
        AllocateType::Address(dest_start as u64),
        MemoryType::LOADER_DATA,
        pages,
    )
    .unwrap();

    for ph in elf.program_headers.iter() {
        if ph.p_type != program_header::PT_LOAD {
            continue;
        }
        let dest = unsafe { slice::from_raw_parts_mut(ph.p_paddr as *mut u8, ph.p_memsz as usize) };
        dest[..(ph.p_filesz as usize)].copy_from_slice(
            &buf[(ph.p_offset as usize)..(ph.p_offset as usize + ph.p_filesz as usize)],
        );
        dest[(ph.p_filesz as usize)..].fill(0);
    }

    info!("Loaded ELF at: 0x{:x}", dest_start);

    elf.entry
}

fn init_graphic(resolution: (usize, usize)) -> GraphicInfo {
    let gop_handle = boot::get_handle_for_protocol::<GraphicsOutput>().unwrap();
    let mut gop = boot::open_protocol_exclusive::<GraphicsOutput>(gop_handle).unwrap();

    let mode = gop
        .modes()
        .find(|mode| mode.info().resolution() == resolution)
        .unwrap();
    info!("Switching graphic mode...");
    gop.set_mode(&mode).unwrap();

    let mode_info = gop.current_mode_info();
    let (width, height) = gop.current_mode_info().resolution();

    GraphicInfo {
        resolution: (width, height),
        format: convert_pixel_format(mode_info.pixel_format()),
        stride: mode_info.stride(),
        framebuf_addr: gop.frame_buffer().as_mut_ptr() as u64,
        framebuf_size: gop.frame_buffer().size(),
    }
}

fn convert_pixel_format(pixel_format: PixelFormat) -> graphic_info::PixelFormat {
    match pixel_format {
        PixelFormat::Rgb => graphic_info::PixelFormat::Rgb,
        PixelFormat::Bgr => graphic_info::PixelFormat::Bgr,
        _ => panic!("Unsupported pixel format"),
    }
}

fn convert_mem_attr(mem_attr: MemoryAttribute) -> u64 {
    mem_attr.bits()
}

fn convert_mem_type(mem_type: MemoryType) -> mem_desc::MemoryType {
    match mem_type {
        MemoryType::RESERVED => mem_desc::MemoryType::Reserved,
        MemoryType::LOADER_CODE => mem_desc::MemoryType::LoaderCode,
        MemoryType::LOADER_DATA => mem_desc::MemoryType::LoaderData,
        MemoryType::BOOT_SERVICES_CODE => mem_desc::MemoryType::BootServicesCode,
        MemoryType::BOOT_SERVICES_DATA => mem_desc::MemoryType::BootServicesData,
        MemoryType::RUNTIME_SERVICES_CODE => mem_desc::MemoryType::RuntimeServicesCode,
        MemoryType::RUNTIME_SERVICES_DATA => mem_desc::MemoryType::RuntimeServicesData,
        MemoryType::CONVENTIONAL => mem_desc::MemoryType::Conventional,
        MemoryType::UNUSABLE => mem_desc::MemoryType::Unusable,
        MemoryType::ACPI_RECLAIM => mem_desc::MemoryType::AcpiReclaim,
        MemoryType::ACPI_NON_VOLATILE => mem_desc::MemoryType::AcpiNonVolatile,
        MemoryType::MMIO => mem_desc::MemoryType::Mmio,
        MemoryType::MMIO_PORT_SPACE => mem_desc::MemoryType::MmioPortSpace,
        MemoryType::PAL_CODE => mem_desc::MemoryType::PalCode,
        MemoryType::PERSISTENT_MEMORY => mem_desc::MemoryType::PersistentMemory,
        MemoryType(value) => mem_desc::MemoryType::Other(value),
    }
}
