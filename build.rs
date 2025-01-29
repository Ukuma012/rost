use bootloader::DiskImageBuilder;
use std::{
    env,
    path::{Path, PathBuf},
    process::Command,
};

fn main() {
    let kernel_path = env::var("CARGO_BIN_FILE_KERNEL").unwrap();
    let disk_builder = DiskImageBuilder::new(PathBuf::from(kernel_path));

    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    let current_dir = env::current_dir().unwrap();
    let uefi_path = out_dir.join("rost-uefi.img");
    let bios_path = out_dir.join("rost-bios.img");

    disk_builder.create_uefi_image(&uefi_path).unwrap();
    disk_builder.create_bios_image(&bios_path).unwrap();

    println!("cargo:rustc-env=UEFI_IMAGE={}", uefi_path.display());
    println!("cargo:rustc-env=BIOS_IMAGE={}", bios_path.display());
}
