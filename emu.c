#include <stdarg.h>
#include <stddef.h>
#include <stdint.h>

#define assert(X) \
    if (!(X)) (*((volatile char*)0x0))

// 12 kb
#define MEMORY_SIZE 12000

// Memory-mapped heap.
#define HEAP (volatile uint8_t*)0x8
static int next_memory = 0;

void* malloc(int bytes) {
    if (next_memory + bytes > MEMORY_SIZE) {
        return NULL;
    }

    void* ptr = (void*)(HEAP + next_memory);
    next_memory += bytes;

    return ptr;
}

// Memory-mapped console.
#define CONSOLE_OUT (*((volatile uint8_t*)0x4))

static void print_char(char c) { CONSOLE_OUT = c; }

static void print_str(const char* str) {
    while (*str != '\0') {
        print_char(*str++);
    }
}

static void print_int(int i) {
    // Handle the case where i is 0
    if (i == 0) {
        char* str = (char*)malloc(sizeof(char) * 2);
        if (str) {
            str[0] = '0';
            str[1] = '\0';
        }
        print_str(str);
    }

    int len = 0;
    int t = i;
    while (t > 0) {
        len++;
        t /= 10;
    }

    char* str = (char*)malloc(sizeof(char) * (len + 1));
    if (!str) {
        print_str("ON NO");
    }

    // *(HEAP + 10) = len;

    str[len] = '\0';
    while (i > 0) {
        str[--len] = (char)(i % 10 + '0');
        i /= 10;
    }

    print_str(str);
}

void printf(const char* fmt, ...) {
    va_list listp;

    va_start(listp, fmt);
    while (*fmt) {
        if (*fmt == '%') {
            fmt++;

            if (*fmt == 's')
                print_str((char*)va_arg(listp, char*));
            else if (*fmt == 'i')
                print_int((int)va_arg(listp, int));
        } else {
            print_char(*fmt);
        }
        fmt++;
    }
    va_end(listp);
}

static int* arr() {
    int* x = malloc(sizeof(int) * 10);
    for (int i = 0; i < 10; i++) {
        x[i] = i + 1;
    }
    // printf("hello world");
    return x;
}

int main() {
    int* a = arr();
    for (int i = 0; i < 10; i++) {
        printf("%i\n", a[i]);
    }

    printf("int: %i, + %s", 69, "Hello, World!");
    assert(6 % 4 == 2);
}

// extern void* __sdata;
// extern void* __edata;

#define EXIT *((volatile char*)0x1)

void __attribute__((naked, noreturn, section(".text.entry"))) _start() {
    (void)main();
    EXIT;
}
