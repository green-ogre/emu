#include <stddef.h>
#include <stdint.h>

#define TICK_RATE 30
// Memory-mapped system tick.
#define TICK ((volatile uint8_t *)TICK_OFFSET)
#define TICK_OFFSET 0x1194C

#define DEBUG 1
#define assert(X) \
    if (!(X)) (*((volatile char *)0x0))
#define PANIC (*((volatile char *)0x0))

// Memory-mapped screen.
#define SCREEN ((volatile uint8_t *)SCREEN_OFFSET)
#define SCREEN_WIDTH 320
#define SCREEN_HEIGHT 200
#define SCREEN_BYTES ((SCREEN_WIDTH * SCREEN_HEIGHT) / 8)
#define SCREEN_OFFSET 0xFA08

#define _RENDER 0x2
#define RENDER (*((volatile uint8_t *)_RENDER))

// Memory-mapped button.
#define BUTTON ((volatile uint8_t *)BUTTON_OFFSET)
#define BUTTON_OFFSET 0x11948

// Allocate memory of size `bytes` from memory-mapped heap.
void *malloc(int bytes);
// Marks allocation at `block` as freed.
void free(void *block);
void *memset(void *s, int c, size_t n);
// Write the formatted string `fmt` to the memory-mapped console.
void printf(const char *fmt, ...);

struct Vec {
    uint32_t cap;
    uint32_t len;
    uint32_t item_size;
    void *data;
};
void print_vec(const Vec *v);
Vec new_vec(uint32_t item_size, uint32_t init_capacity);
void push_vec(Vec *vec, const void *item);
