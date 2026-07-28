use std::path::Path;
use std::fs;

pub const MEMCARD_SLOTS: usize = 2;
pub const MEMCARD_BLOCK: usize = 512;
pub const MEMCARD_BLOCKS: usize = 1024;
pub const MEMCARD_SIZE: usize = MEMCARD_BLOCK * MEMCARD_BLOCKS; // 512KB
pub const MEMCARD_SLOT_SIZE: usize = MEMCARD_SIZE;

pub const MEMCARD_BASE: u32 = 0x06000000;
pub const MEMCARD_CTRL: u32 = 0x06100000;
pub const MEMCARD_STAT: u32 = 0x06100004;
pub const MEMCARD_BLOCKNO: u32 = 0x06100008;

const CTRL_READ: u32 = 1;
const CTRL_WRITE: u32 = 2;
const STAT_INSERT: u32 = 8;

pub struct MemoryCard {
    slots: [Option<Vec<u8>>; MEMCARD_SLOTS],
    dirty: [bool; MEMCARD_SLOTS],
    paths: [Option<String>; MEMCARD_SLOTS],
    ctrl: [u32; MEMCARD_SLOTS],
    stat: [u32; MEMCARD_SLOTS],
    blockno: [u32; MEMCARD_SLOTS],
}

impl MemoryCard {
    pub fn new() -> Self {
        Self {
            slots: [None, None],
            dirty: [false, false],
            paths: [None, None],
            ctrl: [0, 0],
            stat: [0, 0],
            blockno: [0, 0],
        }
    }

    pub fn mount(&mut self, slot: usize, path: &Path) {
        if slot >= MEMCARD_SLOTS { return; }
        let data = fs::read(path).unwrap_or_else(|_| vec![0u8; MEMCARD_SIZE]);
        let mut card = vec![0u8; MEMCARD_SIZE];
        let len = data.len().min(MEMCARD_SIZE);
        card[..len].copy_from_slice(&data[..len]);
        self.slots[slot] = Some(card);
        self.dirty[slot] = false;
        self.paths[slot] = Some(path.to_string_lossy().to_string());
        self.stat[slot] |= STAT_INSERT;
        log::info!("MemCard: slot {} mounted from {}", slot, path.display());
    }

    pub fn unmount(&mut self, slot: usize) {
        if slot >= MEMCARD_SLOTS { return; }
        self.flush(slot);
        self.slots[slot] = None;
        self.stat[slot] &= !STAT_INSERT;
        log::info!("MemCard: slot {} unmounted", slot);
    }

    pub fn flush(&mut self, slot: usize) {
        if slot >= MEMCARD_SLOTS { return; }
        if !self.dirty[slot] { return; }
        if let (Some(ref path), Some(ref data)) = (&self.paths[slot], &self.slots[slot]) {
            let _ = fs::write(Path::new(path), data);
            self.dirty[slot] = false;
            log::info!("MemCard: slot {} flushed to {}", slot, path);
        }
    }

    pub fn flush_all(&mut self) {
        for slot in 0..MEMCARD_SLOTS {
            self.flush(slot);
        }
    }

    pub fn read_byte(&self, addr: u32) -> Option<u8> {
        let ctrl_end = MEMCARD_CTRL + (MEMCARD_SLOTS as u32) * 0x10;
        if addr >= MEMCARD_CTRL && addr < ctrl_end {
            let slot = ((addr - MEMCARD_CTRL) / 0x10) as usize;
            let off = (addr - MEMCARD_CTRL) % 0x10;
            if slot < MEMCARD_SLOTS {
                let val = match off {
                    0x00 => self.ctrl[slot],
                    0x04 => self.stat[slot],
                    0x08 => self.blockno[slot],
                    _ => return None,
                };
                return Some((val >> ((addr & 3) * 8)) as u8);
            }
            return None;
        }
        // Data area
        for slot in 0..MEMCARD_SLOTS {
            let slot_base = MEMCARD_BASE + (slot as u32) * (MEMCARD_SLOT_SIZE as u32);
            let slot_end = slot_base + MEMCARD_SIZE as u32;
            if addr >= slot_base && addr < slot_end {
                if let Some(ref data) = self.slots[slot] {
                    let off = (addr - slot_base) as usize;
                    return data.get(off).copied();
                }
                return Some(0);
            }
        }
        None
    }

    pub fn write_byte(&mut self, addr: u32, val: u8) -> Option<()> {
        // Control registers
        let ctrl_end = MEMCARD_CTRL + (MEMCARD_SLOTS as u32) * 0x10;
        if addr >= MEMCARD_CTRL && addr < ctrl_end {
            let slot = ((addr - MEMCARD_CTRL) / 0x10) as usize;
            let off = (addr - MEMCARD_CTRL) % 0x10;
            if slot < MEMCARD_SLOTS {
                match off {
                    0x00 => {
                        self.ctrl[slot] = val as u32;
                        if val as u32 == CTRL_READ || val as u32 == CTRL_WRITE {
                            // Block transfer happens synchronously in emulator
                            // (real hardware would be async)
                            let _block = self.blockno[slot] as usize;
                            if val as u32 == CTRL_WRITE {
                                self.dirty[slot] = true;
                            }
                            self.ctrl[slot] = 0;
                        }
                    }
                    0x08 => self.blockno[slot] = val as u32,
                    _ => {}
                }
                return Some(());
            }
            return None;
        }
        // Data area
        for slot in 0..MEMCARD_SLOTS {
            let slot_base = MEMCARD_BASE + (slot as u32) * (MEMCARD_SLOT_SIZE as u32);
            let slot_end = slot_base + MEMCARD_SIZE as u32;
            if addr >= slot_base && addr < slot_end {
                if let Some(ref mut data) = self.slots[slot] {
                    let off = (addr - slot_base) as usize;
                    if off < data.len() {
                        data[off] = val;
                        self.dirty[slot] = true;
                        return Some(());
                    }
                }
                return None;
            }
        }
        None
    }

    pub fn present(&self, slot: usize) -> bool {
        self.slots.get(slot).and_then(|s| s.as_ref()).is_some()
    }
}
