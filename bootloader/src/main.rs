#![no_main]
#![no_std]

use log::info;
use uefi::Identify;
use uefi::Result;
use uefi::prelude::*;
use uefi::proto::device_path::text::AllowShortcuts;
use uefi::proto::device_path::text::DevicePathToText;
use uefi::proto::device_path::text::DisplayOnly;
use uefi::proto::loaded_image::LoadedImage;

#[entry]
fn efi_main() -> Status {
    uefi::helpers::init().unwrap();

    print_image_path().unwrap();
    boot::stall(10_000_000);
    Status::SUCCESS
}

fn print_image_path() -> Result {
    let loaded_image = boot::open_protocol_exclusive::<LoadedImage>(boot::image_handle())?;
    let device_path_to_text_handle =
        *boot::locate_handle_buffer(boot::SearchType::ByProtocol(&DevicePathToText::GUID))?
            .first()
            .expect("DevicePathToText is missing");

    let device_path_to_text =
        boot::open_protocol_exclusive::<DevicePathToText>(device_path_to_text_handle)?;

    let image_device_path = loaded_image.file_path().expect("File path is not set");
    let image_device_path_text = device_path_to_text
        .convert_device_path_to_text(image_device_path, DisplayOnly(true), AllowShortcuts(false))
        .expect("convert_device_path_to_text failed");

    info!("Image Path: {}", &*image_device_path_text);
    Ok(())
}
