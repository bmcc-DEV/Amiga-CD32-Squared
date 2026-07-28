const NUM_CHANNELS: usize = 8;
const FIFO_SIZE: usize = 4096;
const SAMPLE_RATE: u32 = 44100;

const DSP_REG_CTRL: usize = 0;     // 0x00
const DSP_REG_VOL: usize = 1;      // 0x04
const DSP_CH_BUF: usize = 4;       // 0x10 + ch*16 + 0
const DSP_CH_LEN: usize = 5;       // 0x10 + ch*16 + 4
const DSP_CH_CTRL: usize = 6;      // 0x10 + ch*16 + 8
const DSP_CH_STAT: usize = 7;      // 0x10 + ch*16 + 12

#[derive(Debug, Clone)]
pub struct AudioChannel {
    fifo: Vec<i16>,
    rd_ptr: usize,
    wr_ptr: usize,
    enabled: bool,
    volume: u16,
    pan: u8,
    sample_pos: u32,
    loop_enabled: bool,
}

impl AudioChannel {
    fn new() -> Self {
        Self {
            fifo: vec![0i16; FIFO_SIZE],
            rd_ptr: 0,
            wr_ptr: 0,
            enabled: false,
            volume: 1024,
            pan: 128,
            sample_pos: 0,
            loop_enabled: false,
        }
    }

    fn push_sample(&mut self, s: i16) {
        self.fifo[self.wr_ptr] = s;
        self.wr_ptr = (self.wr_ptr + 1) % FIFO_SIZE;
    }

    fn read_sample(&mut self) -> i16 {
        let s = self.fifo[self.rd_ptr];
        if self.rd_ptr != self.wr_ptr {
            self.rd_ptr = (self.rd_ptr + 1) % FIFO_SIZE;
        }
        s
    }

    fn samples_avail(&self) -> usize {
        if self.wr_ptr >= self.rd_ptr {
            self.wr_ptr - self.rd_ptr
        } else {
            FIFO_SIZE - self.rd_ptr + self.wr_ptr
        }
    }

    fn reset_fifo(&mut self) {
        self.rd_ptr = 0;
        self.wr_ptr = 0;
    }
}

pub struct AudioSubsystem {
    pub channels: Vec<AudioChannel>,
    master_volume: u16,
    pub output_l: i16,
    pub output_r: i16,
    pub dsp_regs: [u32; 64],
    cycle_accum: u32,
}

impl AudioSubsystem {
    pub fn new() -> Self {
        Self {
            channels: (0..NUM_CHANNELS).map(|_| AudioChannel::new()).collect(),
            master_volume: 1024,
            output_l: 0,
            output_r: 0,
            dsp_regs: [0u8; 64].map(|_| 0u32),
            cycle_accum: 0,
        }
    }

    pub fn tick(&mut self, cycles: u32) {
        self.cycle_accum += cycles;
        let sample_clock = 140_000_000 / SAMPLE_RATE;
        if self.cycle_accum < sample_clock {
            return;
        }
        self.cycle_accum -= sample_clock;

        let mut mix_l: i32 = 0;
        let mut mix_r: i32 = 0;

        for ch in &mut self.channels {
            if !ch.enabled || ch.samples_avail() == 0 {
                continue;
            }
            let s = ch.read_sample() as i32;
            let vol = ch.volume as i32;
            let s_vol = s * vol / 1024;
            let pan_l = (255 - ch.pan as i32) * s_vol / 255;
            let pan_r = (ch.pan as i32) * s_vol / 255;
            mix_l += pan_l;
            mix_r += pan_r;

            if ch.samples_avail() == 0 && ch.loop_enabled {
                ch.reset_fifo();
            }
        }

        mix_l = mix_l * self.master_volume as i32 / 1024;
        mix_r = mix_r * self.master_volume as i32 / 1024;
        self.output_l = mix_l.clamp(i16::MIN as i32, i16::MAX as i32) as i16;
        self.output_r = mix_r.clamp(i16::MIN as i32, i16::MAX as i32) as i16;
    }

    pub fn load_channel_samples(&mut self, ch: usize, data: &[i16]) {
        if ch >= NUM_CHANNELS { return; }
        for &s in data {
            self.channels[ch].push_sample(s);
        }
    }

    pub fn trigger_channel(&mut self, ch: usize, buf_addr: u32, buf_len: u32, flags: u32) -> bool {
        if ch >= NUM_CHANNELS { return false; }
        self.channels[ch].enabled = (flags & 1) != 0;
        self.channels[ch].loop_enabled = (flags & 2) != 0;
        if flags & 1 != 0 {
            self.channels[ch].reset_fifo();
        }
        true
    }

    pub fn channel_enabled(&self, ch: usize) -> bool {
        if ch >= NUM_CHANNELS { false } else { self.channels[ch].enabled }
    }

    pub fn read_byte(&self, addr: u32) -> u8 {
        let idx = ((addr & 0xFF) >> 2) as usize;
        if idx < 64 { (self.dsp_regs[idx] & 0xFF) as u8 } else { 0 }
    }

    pub fn read_half(&self, addr: u32) -> u16 {
        let idx = ((addr & 0xFF) >> 2) as usize;
        if idx < 64 { (self.dsp_regs[idx] & 0xFFFF) as u16 } else { 0 }
    }

    pub fn read_word(&self, addr: u32) -> Option<u32> {
        let idx = ((addr & 0xFF) >> 2) as usize;
        if idx < 64 { Some(self.dsp_regs[idx]) } else { None }
    }

    pub fn write_byte(&mut self, addr: u32, val: u8) {
        if addr & 3 != 0 { return; }
        let idx = ((addr & 0xFF) >> 2) as usize;
        if idx < 64 {
            self.dsp_regs[idx] = (self.dsp_regs[idx] & !0xFF) | val as u32;
        }
    }

    pub fn write_half(&mut self, addr: u32, val: u16) {
        if addr & 1 != 0 { return; }
        let idx = ((addr & 0xFF) >> 2) as usize;
        if idx < 64 {
            self.dsp_regs[idx] = (self.dsp_regs[idx] & !0xFFFF) | val as u32;
        }
    }

    pub fn write_word(&mut self, addr: u32, val: u32) {
        if addr & 3 != 0 { return; }
        let idx = ((addr & 0xFF) >> 2) as usize;
        if idx < 64 {
            self.dsp_regs[idx] = val;
            if idx == DSP_REG_CTRL {
                let mask = val as u8;
                for ch in 0..NUM_CHANNELS.min(8) {
                    let was = self.channels[ch].enabled;
                    self.channels[ch].enabled = (mask >> ch) & 1 != 0;
                }
            }
            if idx == DSP_REG_VOL {
                self.master_volume = (val & 0xFFFF) as u16;
            }
            if idx >= DSP_CH_BUF && (idx - DSP_CH_BUF) % 4 == 2 {
                let ch = (idx - DSP_CH_BUF) / 4;
                let flags = val;
                if ch < NUM_CHANNELS {
                    self.channels[ch].enabled = (flags & 1) != 0;
                    self.channels[ch].loop_enabled = (flags & 2) != 0;
                    if flags & 1 != 0 {
                        self.channels[ch].reset_fifo();
                    }
                }
            }
        }
    }

    pub fn output_stereo(&self) -> (i16, i16) {
        (self.output_l, self.output_r)
    }
}
