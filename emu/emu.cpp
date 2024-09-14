#include <stddef.h>
#include <stdint.h>

#include "slib.h"

static int* arr()
{
    int* x = (int*)malloc(sizeof(int) * 10);
    for (int i = 0; i < 10; i++) {
        x[i] = i + 1;
    }
    return x;
}

struct Foo {
    int x;
    int y;
};

int main()
{
    Vec v = new_vec(sizeof(uint8_t), 4);
    // uint8_t val = 25;
    // push_vec(&v, &val);
    // uint8_t aa = 69;
    // push_vec(&v, &aa);
    // free(v.data);

    // *SCREEN = 0x80;

    // for (int i = 0; i < SCREEN_WIDTH * SCREEN_HEIGHT; i++) {
    //     SCREEN[i] = 0xFF;
    // }

    for (;;) {
        for (int y = 0; y < SCREEN_HEIGHT; y++) {
            for (int x = 0; x < SCREEN_WIDTH; x++) {
                // printf("y: %i, x: %i\n", y, x);
                SCREEN[y * SCREEN_WIDTH + x % SCREEN_WIDTH] = 0x00;
            }
        }

        RENDER;
    }

    return 1;

    // Vec a = new_vec(sizeof(uint8_t), 1);
    // uint8_t vvv = 4;
    // push_vec(&a, &vvv);
    // return (int)((uint8_t*)a.data)[0];
}
