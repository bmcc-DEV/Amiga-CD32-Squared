#ifndef CD32_MEMCARD_H
#define CD32_MEMCARD_H

#include <stdint.h>

#define CD32_MEMCARD_SLOTS     2
#define CD32_MEMCARD_BLOCK     512
#define CD32_MEMCARD_BLOCKS    1024
#define CD32_MEMCARD_SIZE      (CD32_MEMCARD_BLOCK * CD32_MEMCARD_BLOCKS)

#define CD32_MEMCARD_BASE      0x06000000UL
#define CD32_MEMCARD_SLOT_SIZE 0x00080000UL

#define CD32_MEMCARD_CTRL(s)   (*(volatile uint32_t*)(CD32_MEMCARD_BASE + 0x100000 + (s) * 0x10))
#define CD32_MEMCARD_STAT(s)   (*(volatile uint32_t*)(CD32_MEMCARD_BASE + 0x100004 + (s) * 0x10))
#define CD32_MEMCARD_BLOCKNO(s) (*(volatile uint32_t*)(CD32_MEMCARD_BASE + 0x100008 + (s) * 0x10))

#define CD32_MEMCARD_CTRL_READ   1
#define CD32_MEMCARD_CTRL_WRITE  2
#define CD32_MEMCARD_STAT_READY  1
#define CD32_MEMCARD_STAT_BUSY   2
#define CD32_MEMCARD_STAT_ERROR  4
#define CD32_MEMCARD_STAT_INSERT 8

int cd32_memcard_init(void);
int cd32_memcard_read(int slot, uint32_t block, void *buf);
int cd32_memcard_write(int slot, uint32_t block, const void *buf);
int cd32_memcard_present(int slot);

#endif
