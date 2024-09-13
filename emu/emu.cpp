#include <stddef.h>
#include <stdint.h>

#include "slib.h"

static int *arr() {
    int *x = (int *)malloc(sizeof(int) * 10);
    for (int i = 0; i < 10; i++) {
        x[i] = i + 1;
    }
    return x;
}

struct Foo {
    int x;
    int y;
};

int main() {
    Vec v = new_vec(sizeof(uint8_t), 4);
    uint8_t val = 25;
    push_vec(&v, &val);
    uint8_t aa = 69;
    push_vec(&v, &aa);
    free(v.data);

    // Vec a = new_vec(sizeof(uint8_t), 1);
    // uint8_t vvv = 4;
    // push_vec(&a, &vvv);
    // return (int)((uint8_t*)a.data)[0];
}
