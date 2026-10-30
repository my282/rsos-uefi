#!/bin/sh
set -e

IMG=disk.img
LOADER=loader/target/x86_64-unknown-uefi/debug/loader.efi

(cd loader && cargo build)
(cd kernel && cargo build)

# 起動ディスク（パーティションなしの FAT32、64MiB）を毎回作り直す
rm -f "$IMG"
mkfs.fat -C -F 32 -n RSOS "$IMG" 65536 > /dev/null
mmd -i "$IMG" ::/EFI ::/EFI/BOOT
mcopy -i "$IMG" "$LOADER" ::/EFI/BOOT/BOOTX64.EFI
# TODO: カーネルを x86_64-unknown-none でビルドするようにしたらコピーする
# mcopy -i "$IMG" kernel/target/x86_64-unknown-none/debug/kernel ::/kernel.elf

qemu-system-x86_64 -enable-kvm -serial stdio \
  -drive if=pflash,format=raw,readonly=on,file=OVMF_CODE_4M.fd \
  -drive if=pflash,format=raw,file=OVMF_VARS_4M.fd \
  -drive format=raw,file="$IMG"

# 終了後に中身を見る例:
#   mdir -i disk.img ::/
#   mtype -i disk.img ::/test.txt
