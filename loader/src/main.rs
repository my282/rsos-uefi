#![no_main]
#![no_std]

// When using 'no_std', alloc won't be linked by cargo.
extern crate alloc;

use log::info;
use uefi::CString16;
use uefi::allocator::Allocator;
use uefi::boot::MemoryType;
use uefi::fs::FileSystem;
use uefi::mem::memory_map::MemoryMap;
use uefi::prelude::*;

use alloc::string::String;
use core::fmt::Write;

#[entry]
fn main() -> Status {
    // init logging
    uefi::helpers::init().unwrap();

    // open a file system of the disk in which this 'loader' is
    let esp_sys =
        boot::get_image_file_system(boot::image_handle()).expect("Failed to get file system");
    let mut fs = FileSystem::new(esp_sys);
    let path = CString16::try_from("\\mmap.txt").expect("Failed try_from for CString16");
    match fs.try_exists(path.as_ref()) {
        Ok(true) => match fs.remove_file(path.as_ref()) {
            Ok(()) => info!("Succeed to remove file"),
            Err(_) => panic!("Failed to remove file"),
        },
        _ => (),
    };

    let mut mmap_contents = String::new();

    let memmap = boot::memory_map(MemoryType::LOADER_DATA).expect("Failed to get memorymap");
    for m in memmap.entries() {
        let mut s = String::new();
        info!("{:?}", m);
        writeln!(&mut s, "{:?}", m).expect("write!() failed");
        mmap_contents.push_str(s.as_ref());
    }

    match fs.write(path.as_ref(), mmap_contents) {
        Ok(_) => info!("Wrote into file"),
        Err(e) => panic!("{e}"),
    }

    info!("Get into loop...");

    loop {}
}

#[global_allocator]
static ALLOCATOR: Allocator = Allocator;
