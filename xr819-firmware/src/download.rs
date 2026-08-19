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
pub const STATUS_BAD_FORMAT: u32 = 6;

const SECTIONED_MAGIC: u32 = u32::from_le_bytes(*b"XR01");
const SECTION_COPY: u32 = 0;
const SECTION_FILL: u32 = 1;
const SECTION_ENTRY: u32 = 4;
const ITCM_LIMIT: usize = 0x0001_c000;
const DTCM_BASE: usize = 0x0400_0000;
const DTCM_LIMIT: usize = 0x0400_a000;
const HIGH_BASE: usize = 0xfff0_0000;
const HIGH_LIMIT: usize = 0xfff1_4000;

pub fn section_destination_allowed(destination: usize, length: usize) -> bool {
    if destination & 3 != 0 || length & 3 != 0 {
        return false;
    }
    let Some(end) = destination.checked_add(length) else {
        return false;
    };
    (destination < ITCM_LIMIT && end <= ITCM_LIMIT)
        || (destination >= DTCM_BASE && end <= DTCM_LIMIT)
        || (destination >= HIGH_BASE && end <= HIGH_LIMIT)
}

pub fn section_entry_allowed(entry: usize) -> bool {
    let address = entry & !1;
    address < ITCM_LIMIT || (HIGH_BASE..HIGH_LIMIT).contains(&address)
}

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

struct DownloadStream<'a> {
    control: &'a mut Control,
    size: u32,
    position: u32,
}

impl DownloadStream<'_> {
    fn read_word(&mut self) -> Result<u32, ()> {
        if self
            .position
            .checked_add(4)
            .is_none_or(|end| end > self.size)
        {
            return Err(());
        }
        loop {
            let put = unsafe { read_volatile(&self.control.put) };
            if put > self.position {
                break;
            }
            spin_loop();
        }
        let fifo_offset = self.position & (FIFO_SIZE - 1);
        let source = (FIFO_BASE + fifo_offset as usize) as *const u32;
        let word = unsafe { read_volatile(source) };
        self.position += 4;
        unsafe {
            write_volatile(&mut self.control.trace_pc, self.position);
            write_volatile(&mut self.control.debug[0], word);
            write_volatile(&mut self.control.get, self.position);
        }
        Ok(word)
    }

    fn copy_words(&mut self, destination: usize, length: usize) -> Result<(), ()> {
        for offset in (0..length).step_by(4) {
            let word = self.read_word()?;
            unsafe { ((destination + offset) as *mut u32).write_volatile(word) };
        }
        Ok(())
    }
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

    /// Load a compact multi-region image using the vendor copy/fill section
    /// vocabulary. Only the observed ITCM, DTCM, and high-support windows are
    /// writable, and the stream must end exactly at a type-4 entry record.
    pub fn load_sectioned_image(&mut self, size: u32) -> Result<usize, ()> {
        if !(16..=0x0010_0000).contains(&size) || size & 3 != 0 {
            unsafe { write_volatile(&mut self.status, STATUS_BAD_SIZE) };
            return Err(());
        }

        let mut stream = DownloadStream {
            control: self,
            size,
            position: 0,
        };
        if stream.read_word()? != SECTIONED_MAGIC {
            unsafe { write_volatile(&mut stream.control.status, STATUS_BAD_FORMAT) };
            return Err(());
        }

        loop {
            let section_type = stream.read_word()?;
            match section_type {
                SECTION_COPY => {
                    let destination = stream.read_word()? as usize;
                    let length = stream.read_word()? as usize;
                    if !section_destination_allowed(destination, length) {
                        unsafe { write_volatile(&mut stream.control.status, STATUS_BAD_FORMAT) };
                        return Err(());
                    }
                    stream.copy_words(destination, length)?;
                }
                SECTION_FILL => {
                    let destination = stream.read_word()? as usize;
                    let value = stream.read_word()?;
                    let length = stream.read_word()? as usize;
                    if value != 0 || !section_destination_allowed(destination, length) {
                        unsafe { write_volatile(&mut stream.control.status, STATUS_BAD_FORMAT) };
                        return Err(());
                    }
                    for offset in (0..length).step_by(4) {
                        unsafe { ((destination + offset) as *mut u32).write_volatile(0) };
                    }
                }
                SECTION_ENTRY => {
                    let entry = stream.read_word()? as usize;
                    let _trailer = stream.read_word()?;
                    if stream.position != size || !section_entry_allowed(entry) {
                        unsafe { write_volatile(&mut stream.control.status, STATUS_BAD_FORMAT) };
                        return Err(());
                    }
                    unsafe { write_volatile(&mut stream.control.status, STATUS_SUCCESS) };
                    return Ok(entry);
                }
                _ => {
                    unsafe { write_volatile(&mut stream.control.status, STATUS_BAD_FORMAT) };
                    return Err(());
                }
            }
        }
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

    #[test]
    fn section_ranges_match_observed_tcm_windows() {
        assert!(section_destination_allowed(0, 0x1c000));
        assert!(section_destination_allowed(0x0400_0000, 0xa000));
        assert!(section_destination_allowed(0xfff0_0000, 0x14000));
        assert!(!section_destination_allowed(0x0001_bffc, 8));
        assert!(!section_destination_allowed(0x0400_9ffc, 8));
        assert!(!section_destination_allowed(0x0901_0000, 4));
        assert!(!section_destination_allowed(1, 4));
    }

    #[test]
    fn section_entries_accept_thumb_itcm_and_arm_high() {
        assert!(section_entry_allowed(1));
        assert!(section_entry_allowed(0x0001_bfff));
        assert!(section_entry_allowed(0xfff0_0000));
        assert!(!section_entry_allowed(0x0400_0000));
        assert!(!section_entry_allowed(0x0901_0000));
    }
}
