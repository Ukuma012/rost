# Build the UEFI application
cargo build --target x86_64-unknown-uefi --release
qemu-img create -f raw ./build/app.img 200M
mkfs.fat -F 32 -s 2 ./build/app.img
sudo mount -o loop ./build/app.img ./mnt
sudo mkdir -p ./mnt/EFI/BOOT
sudo cp ../target/x86_64-unknown-uefi/release/bootloader.efi ./mnt/EFI/BOOT/BOOTX64.EFI
sudo cp ../kernel.elf ./mnt/kernel.elf
sleep 1
sudo umount ./mnt
cd .. && qemu-system-x86_64 \
         -bios OVMF_CODE.fd \
         -device ahci,id=ahci \
         -device ide-cd,drive=disk,bus=ahci.0 \
         -drive id=disk,if=none,format=raw,file=bootloader/build/app.img \
         -device nec-usb-xhci,id=xhci \
         -device usb-mouse 