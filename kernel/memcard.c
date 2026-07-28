#include "cd32.h"
#include "cd32_memcard.h"

int cd32_memcard_init(void)
{
    return 0;
}

int cd32_memcard_present(int slot)
{
    if (slot < 0 || slot >= CD32_MEMCARD_SLOTS) return 0;
    return (CD32_MEMCARD_STAT(slot) & CD32_MEMCARD_STAT_INSERT) != 0;
}

int cd32_memcard_read(int slot, uint32_t block, void *buf)
{
    if (slot < 0 || slot >= CD32_MEMCARD_SLOTS) return -1;
    if (block >= CD32_MEMCARD_BLOCKS) return -1;
    if (!cd32_memcard_present(slot)) return -1;

    CD32_MEMCARD_BLOCKNO(slot) = block;
    CD32_MEMCARD_CTRL(slot) = CD32_MEMCARD_CTRL_READ;
    uint32_t base = CD32_MEMCARD_BASE + slot * CD32_MEMCARD_SLOT_SIZE;
    uint32_t off = block * CD32_MEMCARD_BLOCK;
    for (uint32_t i = 0; i < CD32_MEMCARD_BLOCK; i++)
        ((volatile uint8_t*)buf)[i] = *(volatile uint8_t*)(base + off + i);

    CD32_MEMCARD_CTRL(slot) = 0;
    return 0;
}

int cd32_memcard_write(int slot, uint32_t block, const void *buf)
{
    if (slot < 0 || slot >= CD32_MEMCARD_SLOTS) return -1;
    if (block >= CD32_MEMCARD_BLOCKS) return -1;
    if (!cd32_memcard_present(slot)) return -1;

    CD32_MEMCARD_BLOCKNO(slot) = block;
    CD32_MEMCARD_CTRL(slot) = CD32_MEMCARD_CTRL_WRITE;
    uint32_t base = CD32_MEMCARD_BASE + slot * CD32_MEMCARD_SLOT_SIZE;
    uint32_t off = block * CD32_MEMCARD_BLOCK;
    for (uint32_t i = 0; i < CD32_MEMCARD_BLOCK; i++)
        *(volatile uint8_t*)(base + off + i) = ((const volatile uint8_t*)buf)[i];

    CD32_MEMCARD_CTRL(slot) = 0;
    return 0;
}
