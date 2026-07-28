#include "cd32.h"

#define DSP_CTRL      (*(volatile uint32_t*)(CD32_DSP_BASE + 0x00))
#define DSP_VOL       (*(volatile uint32_t*)(CD32_DSP_BASE + 0x04))
#define DSP_CH_BUF(n) (*(volatile uint32_t*)(CD32_DSP_BASE + 0x10 + (n) * 16 + 0))
#define DSP_CH_LEN(n) (*(volatile uint32_t*)(CD32_DSP_BASE + 0x10 + (n) * 16 + 4))
#define DSP_CH_CTRL(n)(*(volatile uint32_t*)(CD32_DSP_BASE + 0x10 + (n) * 16 + 8))
#define DSP_CH_STAT(n)(*(volatile uint32_t*)(CD32_DSP_BASE + 0x10 + (n) * 16 + 12))

#define CH_BUF_BASE    0x01100000UL
#define CH_BUF_STRIDE  0x00001000UL

void cd32_audio_init(void)
{
    DSP_CTRL = 0xFF;
    DSP_VOL  = 1024;
}

void cd32_audio_play(int ch, int16_t *samples, int count, int loop)
{
    if (ch < 0 || ch >= CD32_AUDIO_CHANNELS) return;
    if (count <= 0) return;

    uint32_t buf = CH_BUF_BASE + (uint32_t)ch * CH_BUF_STRIDE;
    uint32_t len = (uint32_t)count * 2;
    if (len > CH_BUF_STRIDE) len = CH_BUF_STRIDE;

    for (uint32_t i = 0; i < len / 2; i++)
        *(volatile int16_t*)(buf + i * 2) = samples[i];

    DSP_CH_BUF(ch)  = buf;
    DSP_CH_LEN(ch)  = len;
    DSP_CH_CTRL(ch) = 1 | (loop ? 2 : 0);
}

void cd32_audio_stop(int ch)
{
    if (ch < 0 || ch >= CD32_AUDIO_CHANNELS) return;
    DSP_CH_CTRL(ch) = 0;
    DSP_CTRL &= ~(1u << (uint32_t)ch);
}

void cd32_audio_volume(int ch, int vol)
{
    if (ch < 0 || ch >= CD32_AUDIO_CHANNELS || vol < 0) return;
    if (vol > 1024) vol = 1024;
    DSP_CH_BUF(ch) = (DSP_CH_BUF(ch) & 0xFFFF) | ((uint32_t)vol << 16);
}

void cd32_audio_pan(int ch, int pan)
{
    if (ch < 0 || ch >= CD32_AUDIO_CHANNELS || pan > 255) return;
    DSP_CH_LEN(ch) = (DSP_CH_LEN(ch) & 0xFFFF0000) | (uint32_t)pan;
}
