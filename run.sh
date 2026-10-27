#!/bin/sh
set -e

(cd loader && cargo build)
(cd kernel && cargo build)

mkdir -p esp/efi/boot
cp loader/target/x86_64-unknown-uefi/debug/loader.efi esp/efi/boot/bootx64.efi

qemu-system-x86_64 -enable-kvm -serial stdio\
  -drive if=pflash,format=raw,readonly=on,file=OVMF_CODE_4M.fd \
  -drive if=pflash,format=raw,file=OVMF_VARS_4M.fd \
  -drive format=raw,file=fat:rw:esp
