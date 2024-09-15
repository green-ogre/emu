#include "slib.h"

#include <stdarg.h>
#include <stddef.h>
#include <stdint.h>
#pragma GCC diagnostic ignored "-Warray-bounds"

#pragma GCC push_options
#pragma GCC optimize("O0")
void *memset(void *s, int c, size_t n)
{
    unsigned char *p = (unsigned char *)s;
    while (n--) {
        *p++ = (unsigned char)c;
    }
    return s;
}
#pragma GCC pop_options

void memcpy(void *dst, const void *src, int bytes)
{
    uint8_t *d = (uint8_t *)dst;
    const uint8_t *s = (uint8_t *)src;

    for (int i = 0; i < bytes; i++) {
        d[i] = s[i];
    }
}

void print_vec(const Vec *v)
{
    printf("Vec { cap: %i, len: %i, item_size: %i, data: %i }\n",
           v->cap,
           v->len,
           v->item_size,
           v->data);
}

Vec new_vec(uint32_t item_size, uint32_t init_capacity)
{
    Vec vec = {};
    vec.cap = init_capacity;
    vec.item_size = item_size;
    vec.data = malloc(item_size * init_capacity);
    return vec;
}

void push_vec(Vec *vec, const void *item)
{
    if (vec->len < vec->cap) {
        memcpy((uint8_t *)vec->data + (vec->len * vec->item_size),
               item,
               vec->item_size);
        vec->len++;
    }
}

// 64 kb
#define MEMORY_SIZE 64000

// Memory-mapped heap.
#define HEAP (volatile uint8_t *)HEAP_OFFSET
#define HEAP_OFFSET 0x8

struct Alloc {
    uint16_t size;
    uint16_t is_free;
    uint32_t offset;
    Alloc *next;
};

static Alloc *alloc = (Alloc *)(HEAP);

static void allocate(Alloc *a, int bytes)
{
    printf("Allocating %i bytes...\n", bytes);

    Alloc new_alloc = {};
    int aligned_bytes = bytes + (4 - bytes % 4);
    new_alloc.size = sizeof(Alloc) + aligned_bytes;
    new_alloc.offset = a->offset + a->size;
    new_alloc.is_free = 0;

    printf("Alloc header size: %i, Allocation size: %i, Offset: %i\n",
           sizeof(Alloc),
           new_alloc.size - sizeof(Alloc),
           new_alloc.offset + HEAP_OFFSET);

    *((Alloc *)(HEAP + new_alloc.offset)) = new_alloc;

#if DEBUG
    memset((void *)((uint8_t *)a + sizeof(Alloc)), 0, a->size - sizeof(Alloc));
#endif

    a->next = (Alloc *)(HEAP + new_alloc.offset);
}

static void reuse_allocation(Alloc *a, int bytes)
{
    assert((uint8_t)bytes <= a->size - sizeof(Alloc));

    printf("Allocating %i bytes...\n", bytes);
    printf("Resuing header\n");

    a->is_free = 0;

#if DEBUG
    memset((void *)((uint8_t *)a + sizeof(Alloc)), 0, a->size - sizeof(Alloc));
#endif

    printf("Alloc header size: %i, Allocation size: %i, Offset: %i\n",
           sizeof(Alloc),
           a->size - sizeof(Alloc),
           a->offset + HEAP_OFFSET);
}

void *malloc(int bytes)
{
    Alloc *head = alloc;

    if (!head->next) {
        alloc->size = 16;
        alloc->offset = 0;
        alloc->is_free = 0;
    }

    while (head->next) {
        if (head->is_free == 1) {
            break;
        }
        head = head->next;
    }

    if (head->is_free == 1) {
        reuse_allocation(head, bytes);
        return (Alloc *)(HEAP + head->offset + sizeof(Alloc));
    } else {
        allocate(head, bytes);
        return (Alloc *)(HEAP + head->next->offset + sizeof(Alloc));
    }
}

void free(void *block)
{
    Alloc *a = (Alloc *)((uint8_t *)block - sizeof(Alloc));
    printf("Freeing allocation...\n");
    printf("Alloc header size: %i, Allocation size: %i, Offset: %i\n",
           sizeof(Alloc),
           a->size - sizeof(Alloc),
           a->offset + HEAP_OFFSET);
    a->is_free = 1;

#if DEBUG
    uint8_t *mem = (uint8_t *)a + sizeof(Alloc);
    for (uint8_t i = 0; i < a->size - sizeof(Alloc); i++) {
        switch (i % 4) {
            case 0:
                mem[i] = 0xDE;
                break;
            case 1:
                mem[i] = 0xAD;
                break;
            case 2:
                mem[i] = 0xBE;
                break;
            case 3:
                mem[i] = 0xEF;
                break;
            default:
                break;
        }
    }
#endif
}

// Memory-mapped console.
#define CONSOLE_OUT (*((volatile uint8_t *)CONSOLE_OFFSET))
#define CONSOLE_OFFSET 0x4

static void print_char(char c)
{
    CONSOLE_OUT = c;
}

static void print_str(const char *str)
{
    while (*str != '\0') {
        print_char(*str++);
    }
}

static void print_int(int i)
{
    if (i == 0) {
        char str[2];
        str[0] = '0';
        str[1] = '\0';
        print_str(str);
    }

    int len = 0;
    int t = i;
    while (t > 0) {
        len++;
        t /= 10;
    }

    char str[len];

    str[len] = '\0';
    while (i > 0) {
        str[--len] = (char)(i % 10 + '0');
        i /= 10;
    }

    print_str(str);
}

void print_address(uintptr_t addr)
{
    char hex_chars[] = "0123456789abcdef";
    char buffer[sizeof(void *) * 2 + 3];
    int idx = sizeof(buffer) - 1;

    buffer[idx--] = '\0';

    do {
        buffer[idx--] = hex_chars[addr & 0xf];
        addr >>= 4;
    } while (addr && idx > 1);

    buffer[1] = 'x';
    buffer[0] = '0';

    print_str(buffer);
}

void printf(const char *fmt, ...)
{
    va_list listp;

    va_start(listp, fmt);
    while (*fmt) {
        if (*fmt == '%') {
            fmt++;

            if (*fmt == 's')
                print_str((const char *)va_arg(listp, char *));
            else if (*fmt == 'i')
                print_int((int)va_arg(listp, int));
            else if (*fmt == 'p')
                print_address((uintptr_t)va_arg(listp, void *));
        } else {
            print_char(*fmt);
        }
        fmt++;
    }
    va_end(listp);
}

// extern void* __sdata;
// extern void* __edata;

#define EXIT *((volatile char *)0x1)

extern int main();

void __attribute__((naked, noreturn, section(".text.entry"))) _start()
{
    asm volatile("call main");
    // Emu will exit gracefully upon reading from 0x1
    asm volatile("lw x0, 0x1(x0)");
}
