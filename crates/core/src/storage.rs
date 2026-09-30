//! Flash-backed save storage on the ESP-IDF `nvs` partition.

const MAGIC: [u8; 4] = *b"SAVE";
const HEADER_LEN: usize = 16;
const SECTOR_SIZE: usize = 4096;
const WORD_SIZE: usize = 4;

pub const MAX_PAYLOAD: usize = SECTOR_SIZE * 5 - HEADER_LEN;

#[cfg(not(feature = "desktop"))]
mod firmware {
    use embedded_storage::nor_flash::{NorFlash, ReadNorFlash};
    use esp_bootloader_esp_idf::partitions::{
        read_partition_table, DataPartitionSubType, PartitionType, PARTITION_TABLE_MAX_LEN,
    };
    use esp_hal::peripherals::FLASH;
    use crate::println;
    use esp_storage::FlashStorage;

    use super::{HEADER_LEN, MAGIC, MAX_PAYLOAD, SECTOR_SIZE, WORD_SIZE};

    static mut FLASH_STORAGE: Option<FlashStorage<'static>> = None;

    pub fn init(flash: FLASH<'static>) {
        unsafe {
            let slot = &mut *core::ptr::addr_of_mut!(FLASH_STORAGE);
            if slot.is_none() {
                *slot = Some(FlashStorage::new(flash));
            }
        }
    }

    fn flash() -> Option<&'static mut FlashStorage<'static>> {
        unsafe { (*core::ptr::addr_of_mut!(FLASH_STORAGE)).as_mut() }
    }

    #[derive(Clone, Copy)]
    struct PartitionInfo {
        offset: u32,
        sectors: usize,
    }

    fn find_nvs_partition(flash: &mut FlashStorage) -> Option<PartitionInfo> {
        let mut table_buf = [0u8; PARTITION_TABLE_MAX_LEN];
        let pt = read_partition_table(flash, &mut table_buf).ok()?;
        let entry = pt
            .find_partition(PartitionType::Data(DataPartitionSubType::Nvs))
            .ok()??;
        Some(PartitionInfo {
            offset: entry.offset(),
            sectors: (entry.len() as usize) / SECTOR_SIZE,
        })
    }

    #[derive(Clone, Copy)]
    struct Record {
        start_sector: usize,
        seq: u32,
        len: u32,
    }

    impl Record {
        fn sectors(self) -> usize {
            sectors_for(self.len as usize)
        }
    }

    fn sectors_for(payload_len: usize) -> usize {
        let total = HEADER_LEN + payload_len;
        total.div_ceil(SECTOR_SIZE).max(1)
    }

    fn read_header(flash: &mut FlashStorage, part: PartitionInfo, sector: usize) -> Option<Record> {
        let mut buf = [0u8; HEADER_LEN];
        let addr = part.offset + (sector * SECTOR_SIZE) as u32;
        if let Err(_) = flash.read(addr, &mut buf) {
            return None;
        }
        if buf[..4] != MAGIC {
            return None;
        }
        Some(Record {
            start_sector: sector,
            seq: u32::from_le_bytes(buf[4..8].try_into().ok()?),
            len: u32::from_le_bytes(buf[8..12].try_into().ok()?),
        })
    }

    fn find_latest(flash: &mut FlashStorage, part: PartitionInfo) -> Option<Record> {
        let mut best: Option<Record> = None;
        for i in 0..part.sectors {
            if let Some(r) = read_header(flash, part, i) {
                match best {
                    Some(b) if r.seq > b.seq => best = Some(r),
                    None => best = Some(r),
                    _ => {}
                }
            }
        }
        best
    }

    pub fn has_save() -> bool {
        let Some(flash) = flash() else { return false; };
        let Some(part) = find_nvs_partition(flash) else { return false; };
        find_latest(flash, part).is_some()
    }

    pub fn read_latest(buf: &mut [u8]) -> Option<usize> {
        let flash = flash()?;
        let part = find_nvs_partition(flash)?;
        let r = find_latest(flash, part)?;
        let len = r.len as usize;
        if len > buf.len() || len > MAX_PAYLOAD {
            return None;
        }

        let mut scratch = [0u8; SECTOR_SIZE];
        let mut read = 0;
        let mut sector_idx = r.start_sector;
        let mut sector_offset = HEADER_LEN;
        while read < len {
            let space = SECTOR_SIZE - sector_offset;
            let chunk = space.min(len - read);
            let aligned_chunk = (chunk + WORD_SIZE - 1) & !(WORD_SIZE - 1);
            let aligned_chunk = aligned_chunk.min(space);
            let addr = part.offset + (sector_idx * SECTOR_SIZE + sector_offset) as u32;
            if flash.read(addr, &mut scratch[..aligned_chunk]).is_err() {
                return None;
            }
            buf[read..read + chunk].copy_from_slice(&scratch[..chunk]);
            read += chunk;
            sector_offset += chunk;
            if sector_offset >= SECTOR_SIZE {
                sector_idx = (sector_idx + 1) % part.sectors;
                sector_offset = 0;
            }
        }
        Some(len)
    }

    fn write_chunk(
        flash: &mut FlashStorage,
        addr: u32,
        chunk: &[u8],
        scratch: &mut [u8; SECTOR_SIZE],
    ) -> bool {
        let padded = chunk.len().div_ceil(WORD_SIZE) * WORD_SIZE;
        scratch[..chunk.len()].copy_from_slice(chunk);
        for b in &mut scratch[chunk.len()..padded] {
            *b = 0xFF;
        }
        flash.write(addr, &scratch[..padded]).is_ok()
    }

    pub fn write_next(payload: &[u8]) -> bool {
        if payload.len() > MAX_PAYLOAD { return false; }
        let Some(flash) = flash() else { return false; };
        let Some(part) = find_nvs_partition(flash) else { return false; };
        let sectors_total = part.sectors;
        if sectors_total == 0 { return false; }
        let needed = sectors_for(payload.len());
        if needed > sectors_total { return false; }

        let latest = find_latest(flash, part);
        let (start, next_seq) = match latest {
            Some(r) => ((r.start_sector + r.sectors()) % sectors_total, r.seq.wrapping_add(1)),
            None => (0, 1),
        };

        for i in 0..needed {
            let sector = (start + i) % sectors_total;
            let addr = part.offset + (sector * SECTOR_SIZE) as u32;
            if flash.erase(addr, addr + SECTOR_SIZE as u32).is_err() {
                return false;
            }
        }

        let mut scratch = [0u8; SECTOR_SIZE];
        let mut written = 0;
        let mut sector_idx = start;
        let mut sector_offset = HEADER_LEN;
        while written < payload.len() {
            let space = SECTOR_SIZE - sector_offset;
            let chunk_len = space.min(payload.len() - written);
            let addr = part.offset + (sector_idx * SECTOR_SIZE + sector_offset) as u32;
            let chunk = &payload[written..written + chunk_len];
            if !write_chunk(flash, addr, chunk, &mut scratch) {
                return false;
            }
            written += chunk_len;
            sector_offset += chunk_len;
            if sector_offset >= SECTOR_SIZE {
                sector_idx = (sector_idx + 1) % sectors_total;
                sector_offset = 0;
            }
        }

        let mut header = [0u8; HEADER_LEN];
        header[..4].copy_from_slice(&MAGIC);
        header[4..8].copy_from_slice(&next_seq.to_le_bytes());
        header[8..12].copy_from_slice(&(payload.len() as u32).to_le_bytes());
        let start_addr = part.offset + (start * SECTOR_SIZE) as u32;
        if flash.write(start_addr, &header).is_err() { return false; }

        let mut readback = [0u8; HEADER_LEN];
        if flash.read(start_addr, &mut readback).is_err() { return false; }
        readback == header
    }

    pub fn erase_all() -> bool {
        let Some(flash) = flash() else { return false; };
        let Some(part) = find_nvs_partition(flash) else { return false; };
        for i in 0..part.sectors {
            let addr = part.offset + (i * SECTOR_SIZE) as u32;
            if flash.erase(addr, addr + SECTOR_SIZE as u32).is_err() {
                return false;
            }
        }
        true
    }
} // mod firmware

#[cfg(not(feature = "desktop"))]
pub use firmware::{erase_all, has_save, init, read_latest, write_next};

#[cfg(feature = "desktop")]
mod desktop {
    use std::fs;
    use std::io::Read;
    use std::path::Path;
    const SAVE_PATH: &str = "./catode32-save.json";
    pub fn init() {}
    pub fn has_save() -> bool { Path::new(SAVE_PATH).is_file() }
    pub fn read_latest(buf: &mut [u8]) -> Option<usize> {
        let mut f = fs::File::open(SAVE_PATH).ok()?;
        let mut n = 0;
        loop {
            match f.read(&mut buf[n..]) {
                Ok(0) => break,
                Ok(k) => n += k,
                _ => return None,
            }
            if n == buf.len() { break; }
        }
        Some(n)
    }
    pub fn write_next(payload: &[u8]) -> bool {
        fs::write(SAVE_PATH, payload).is_ok()
    }
    pub fn erase_all() -> bool {
        if !has_save() { return true; }
        fs::remove_file(SAVE_PATH).is_ok()
    }
}

#[cfg(feature = "desktop")]
pub use desktop::{erase_all, has_save, init, read_latest, write_next};
