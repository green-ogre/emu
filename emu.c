typedef struct Apple {
    int x;
    int y;
} Apple;

int test() {
    int x = 0;
    for (int i = 0; i < 10; i++) {
        x += i;
    }
    return x;
}

int main() { return test(); }
