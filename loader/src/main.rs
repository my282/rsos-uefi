#![no_main]
#![no_std]

// When using 'no_std', 'alloc' won't be linked by cargo.
extern crate alloc;

use alloc::string::String;
use alloc::string::ToString;
use core::fmt::Write;
use log::info;
use uefi::CString16;
use uefi::Identify;
use uefi::allocator::Allocator;
use uefi::boot::{MemoryType, SearchType};
use uefi::fs::FileSystem;
use uefi::mem::memory_map::MemoryMap;
use uefi::prelude::*;
use uefi::proto::console::gop;
use uefi::proto::console::gop::GraphicsOutput;
use uefi::proto::console::gop::PixelFormat;
use uefi::proto::device_path::DevicePath;

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
    let gop_handle = boot::locate_handle_buffer(SearchType::ByProtocol(&gop::GraphicsOutput::GUID))
        .expect("Failed to obtain gop handle");
    let mut selected_h: Option<Handle> = None;
    for h in gop_handle.iter() {
        // h is a handle
        // info!("Handle: {h:?}");

        // parametars to open protocol
        let params = boot::OpenProtocolParams {
            handle: *h,
            agent: boot::image_handle(),
            controller: None,
        };

        match boot::test_protocol::<DevicePath>(params) {
            Ok(true) => selected_h = Some(*h),
            Ok(false) => (),
            Err(e) => info!("'test_protocol' failed {}", e),
        }
    }
    let selected_h = selected_h.expect("No GOP found");
    let mut go = boot::open_protocol_exclusive::<GraphicsOutput>(selected_h)
        .expect("Failed to open protocol");

    info!("framebuffer pointer: {:?}", go.frame_buffer().as_mut_ptr());
    info!("framebuffer size: {:?}", go.frame_buffer().size());
    info!("resolution: {:?}", go.current_mode_info().resolution());
    let format = go.current_mode_info().pixel_format();
    info!("pixel format: {:?}", format);
    info!(
        "pixel bitmask: {:?}",
        go.current_mode_info().pixel_bitmask()
    );
    let stride = go.current_mode_info().stride();
    info!("stride: {:?}", stride);

    let fb_size = go.frame_buffer().size();

    let mut fb = go.frame_buffer();
    for i in 0..fb_size {
        unsafe {
            fb.write_byte(i, 0xff);
        }
    }

    let bgr = Bgr {
        blue: 0xf0,
        green: 0xed,
        red: 0x59,
    };

    for x in 0..100 {
        for y in 0..100 {
            unsafe {
                write_fb(&mut fb, x, y, bgr, stride, format);
            }
        }
    }

    // let op = gop::BltOp::VideoFill {
    //     color: BltPixel::new(0xff, 0xff, 0xff),
    //     dest: (0, 0),
    //     dims: go.current_mode_info().resolution(),
    // };
    // go.blt(op).expect("'blt' failed");

    info!("Get into loop...");

    loop {}
}

#[derive(Copy, Clone)]
struct Bgr {
    blue: u8,
    green: u8,
    red: u8,
}

impl Bgr {
    fn to_bytes(self) -> [u8; 4] {
        [self.blue, self.green, self.red, 0x00]
    }
}

// [ INFO]:  src/main.rs@078: framebuffer pointer: 0x80000000
// [ INFO]:  src/main.rs@079: framebuffer size: 4096000 (= 1280 * 800 * 4)
// [ INFO]:  src/main.rs@080: resolution: (1280, 800)
// [ INFO]:  src/main.rs@081: pixel format: Bgr
// [ INFO]:  src/main.rs@082: pixel bitmask: None
// [ INFO]:  src/main.rs@086: stride: 1280

unsafe fn write_fb(
    fb: &mut gop::FrameBuffer,
    x: usize,
    y: usize,
    bgr: Bgr,
    stride: usize,
    format: PixelFormat,
) {
    let offset = 4 * (stride * y + x);
    unsafe {
        match write_rgb(fb, offset, bgr, format) {
            Ok(()) => (),
            Err(error) => info!("Write_rgb failed: {error}"),
        };
    }
}

unsafe fn write_rgb(
    fb: &mut gop::FrameBuffer,
    offset: usize,
    bgr: Bgr,
    format: PixelFormat,
) -> Result<(), String> {
    match format {
        PixelFormat::Rgb => return Err("RGB".to_string()),
        PixelFormat::Bgr => unsafe {
            fb.write_value(offset, bgr.to_bytes());
        },
        PixelFormat::Bitmask => return Err("Bitmask".to_string()),
        PixelFormat::BltOnly => return Err("BitOnly".to_string()),
    }

    // check format
    Ok(())
}

#[global_allocator]
static ALLOCATOR: Allocator = Allocator;
