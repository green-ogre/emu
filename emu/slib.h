#include <stddef.h>
#include <stdint.h>

#define DEBUG 1
#define assert(X)                                                                                                      \
    if (!(X))                                                                                                          \
    (*((volatile char *)0x0))

// Allocate memory of size `bytes` from memory-mapped heap.
void *malloc(int bytes);
// Marks allocation at `block` as freed.
void free(void *block);

void *memset(void *s, int c, size_t n);

// Write the formatted string `fmt` to the memory-mapped console.
void printf(const char *fmt, ...);

struct Vec
{
    uint32_t cap;
    uint32_t len;
    uint32_t item_size;
    void *data;
};
void print_vec(const Vec *v);
Vec new_vec(uint32_t item_size, uint32_t init_capacity);
void push_vec(Vec *vec, const void *item);
