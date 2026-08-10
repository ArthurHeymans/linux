//! XR819 host download-control protocol used before WSM starts.

use core::hint::spin_loop;
use core::ptr::{read_volatile, write_volatile};

pub const FIFO_BASE: usize = 0x0900_4000;
pub const FIFO_SIZE: u32 = 0x8000;
pub const CONTROL_BASE: usize = 0x0900_ff80;
pub const MAIN_IMAGE_BASE: usize = 0xfff0_0000;

pub fn split_destination(
    offset: u32,
    first_size: u32,
    first_base: usize,
    second_base: usize,
) -> usize {
    if offset < first_size {
        first_base + offset as usize
    } else {
        second_base + offset.wrapping_sub(first_size) as usize
    }
}

pub const ARE_YOU_HERE: u32 = 0x8765_4321;
pub const I_AM_HERE: u32 = 0x1234_5678;
pub const STATUS_PENDING: u32 = u32::MAX;
pub const STATUS_SUCCESS: u32 = 0;
pub const STATUS_BAD_SIZE: u32 = 5;

#[repr(C)]
pub struct Control {
    image_size: u32,
    flags: u32,
    put: u32,
    trace_pc: u32,
    get: u32,
    status: u32,
    debug: [u32; 26],
}

impl Control {
    /// # Safety
    ///
    /// The XR819 shared-memory window must be enabled and mapped normally.
    pub unsafe fn get() -> &'static mut Self {
        unsafe { &mut *(CONTROL_BASE as *mut Self) }
    }

    pub fn advertise(&mut self) {
        unsafe {
            write_volatile(&mut self.flags, 0);
            write_volatile(&mut self.put, 0);
            write_volatile(&mut self.get, 0);
            write_volatile(&mut self.status, STATUS_PENDING);
            write_volatile(&mut self.image_size, I_AM_HERE);
        }
    }

    pub fn wait_for_image_size(&self) -> u32 {
        loop {
            let size = unsafe { read_volatile(&self.image_size) };
            if size != ARE_YOU_HERE && size != I_AM_HERE {
                return size;
            }
            spin_loop();
        }
    }

    pub fn copy_image(&mut self, size: u32) -> Result<(), ()> {
        self.copy_image_to(size, MAIN_IMAGE_BASE)
    }

    pub fn copy_image_to(&mut self, size: u32, destination_base: usize) -> Result<(), ()> {
        if size == 0 || size > 0x0010_0000 {
            unsafe { write_volatile(&mut self.status, STATUS_BAD_SIZE) };
            return Err(());
        }

        let mut copied = 0;
        while copied < size {
            let put = loop {
                let put = unsafe { read_volatile(&self.put) };
                if put != copied {
                    break put;
                }
                spin_loop();
            };

            while copied < put && copied < size {
                let fifo_offset = copied & (FIFO_SIZE - 1);
                let source = (FIFO_BASE + fifo_offset as usize) as *const u32;
                let destination = (destination_base + copied as usize) as *mut u32;
                let word = unsafe { read_volatile(source) };
                unsafe {
                    write_volatile(&mut self.trace_pc, copied);
                    write_volatile(&mut self.debug[0], word);
                    write_volatile(destination, word);
                }
                copied = copied.wrapping_add(4);
            }
            unsafe { write_volatile(&mut self.get, copied.min(size)) };
        }

        unsafe { write_volatile(&mut self.status, STATUS_SUCCESS) };
        Ok(())
    }

    /// Stream a flat host payload into two non-contiguous executable windows.
    pub fn copy_image_split_to(
        &mut self,
        size: u32,
        first_size: u32,
        first_base: usize,
        second_base: usize,
    ) -> Result<(), ()> {
        if size == 0 || size > 0x0010_0000 || first_size == 0 || first_size & 3 != 0 {
            unsafe { write_volatile(&mut self.status, STATUS_BAD_SIZE) };
            return Err(());
        }

        let mut copied = 0;
        while copied < size {
            let put = loop {
                let put = unsafe { read_volatile(&self.put) };
                if put != copied {
                    break put;
                }
                spin_loop();
            };

            while copied < put && copied < size {
                let fifo_offset = copied & (FIFO_SIZE - 1);
                let source = (FIFO_BASE + fifo_offset as usize) as *const u32;
                let destination =
                    split_destination(copied, first_size, first_base, second_base) as *mut u32;
                let word = unsafe { read_volatile(source) };
                unsafe {
                    write_volatile(&mut self.trace_pc, copied);
                    write_volatile(&mut self.debug[0], word);
                    write_volatile(destination, word);
                }
                copied = copied.wrapping_add(4);
            }
            unsafe { write_volatile(&mut self.get, copied.min(size)) };
        }

        unsafe { write_volatile(&mut self.status, STATUS_SUCCESS) };
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn split_destination_preserves_each_region_offset() {
        assert_eq!(split_destination(0, 0x7500, 0, 0xfff0_0000), 0);
        assert_eq!(split_destination(0x74fc, 0x7500, 0, 0xfff0_0000), 0x74fc);
        assert_eq!(
            split_destination(0x7500, 0x7500, 0, 0xfff0_0000),
            0xfff0_0000,
        );
        assert_eq!(
            split_destination(0x7520, 0x7500, 0, 0xfff0_0000),
            0xfff0_0020,
        );
    }
}
