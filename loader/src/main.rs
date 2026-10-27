#![no_main]
#![no_std]

use log::info;
use uefi::CString16;
use uefi::allocator::Allocator;
use uefi::boot::MemoryType;
use uefi::fs::FileSystem;
use uefi::mem::memory_map::MemoryMap;
use uefi::prelude::*;

#[entry]
fn main() -> Status {
    // init logging
    uefi::helpers::init().unwrap();

    info!("Hello!");

    let memmap = boot::memory_map(MemoryType::LOADER_DATA).expect("Failed to get memorymap");
    for m in memmap.entries() {
        match m.ty {
            MemoryType::RESERVED => info!("{:?}", m),
            _ => (),
        }
    }

    // open a file system of the disk in which this 'loader' is
    let esp_sys =
        boot::get_image_file_system(boot::image_handle()).expect("Failed to get file system");

    let mut fs = FileSystem::new(esp_sys);
    let path = CString16::try_from("\\test.txt").expect("Failed try_from for CString16");
    // let contents =
    //     CString16::try_from("This is file writing test!").expect("Failed try_from for CString16");
    let contents: &str = "Hello! This is file writing test!";

    info!("{:?}", contents);

    fs.write(path.as_ref(), contents).expect("Failed to write");

    info!("Wrote to file");

    loop {}
}

#[global_allocator]
static ALLOCATOR: Allocator = Allocator;
