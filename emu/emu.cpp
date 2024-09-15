#include <stddef.h>
#include <stdint.h>

#include "slib.h"

static void clear_screen()
{
    memset((void*)SCREEN, 0, SCREEN_BYTES);
}

/// Sets the byte at byte position (`x`, `y`) to `v`.
///
/// Bounds checking is the resposibility of the caller.
static void draw_byte(int x, int y, int v)
{
    int byte_index = (y * SCREEN_WIDTH + x) / 8;

#if DEBUG
    if (byte_index >= SCREEN_BYTES || byte_index < 0) {
        printf(
            "draw_byte: Invalid byte location: { x: %i, y: %i }, byte_index: "
            "%i\n",
            x,
            y,
            byte_index);
        PANIC;
    }
#endif

    SCREEN[byte_index] = v;
}

/// Sets the bit at pixel position (`x`, `y`) to `v`.
///
/// Bounds checking is the resposibility of the caller.
static void draw_pixel(int x, int y, int v)
{
    int byte_index = (y * SCREEN_WIDTH + x) / 8;
    int bit_index = (y * SCREEN_WIDTH + x) % 8;
    int mask = 1 << (7 - bit_index);

#if DEBUG

    if (v != 1 && v != 0) {
        printf(
            "draw_pixel: Invalid value for pixel: %i. Should be either 1 or "
            "0.",
            v);
        PANIC;
    }

    if (byte_index >= SCREEN_BYTES || byte_index < 0) {
        printf(
            "draw_pixel: Invalid pixel location: { x: %i, y: %i }, byte_index: "
            "%i\n",
            x,
            y,
            byte_index);
        PANIC;
    }
#endif

    SCREEN[byte_index] = (SCREEN[byte_index] & ~mask) | (-v & mask);
}

static void draw_checkerboard(int x, int y)
{
    if ((x / 8 + y / 8) % 2 == 0) {
        draw_pixel(x, y, 1);
    }
}

struct BitMap {
    /// Pixels are packed into bytes.
    ///
    /// 1 byte = 8 pixels.
    uint8_t* pixels;
    int width;
    int height;
};

static BitMap bitmap(int width, int height, const uint8_t* pixels)
{
    return BitMap{.pixels = (uint8_t*)pixels, .width = width, .height = height};
}

static void draw_bitmap(BitMap* bmp, int pixel_x, int pixel_y)
{
    for (int y = 0; y < bmp->height; y++) {
        for (int x = 0; x < bmp->width; x++) {
            int px = pixel_x + x;
            int py = pixel_y + y;
            if (px < SCREEN_WIDTH && py < SCREEN_HEIGHT && px >= 0 && py >= 0) {
                int p = (y * bmp->height + x);
                int v = (bmp->pixels[p / 8] >> (7 - (p % 8))) & 1;
                draw_pixel(px, py, v);
            }
        }
    }
}

struct Vec2 {
    uint32_t x;
    uint32_t y;
};

static Vec2 vec2(uint32_t x, uint32_t y)
{
    return Vec2{
        .x = x,
        .y = y,
    };
}

struct Rect {
    int width;
    int height;
    Vec2 position;
};

static void draw_rect(const Rect* rect)
{
    for (int y = 0; y < rect->height; y++) {
        for (int x = 0; x < rect->width; x++) {
            int px = rect->position.x + x;
            int py = rect->position.y + y;
            if (px < SCREEN_WIDTH && py < SCREEN_HEIGHT && px >= 0 && py >= 0)
                draw_pixel(px, py, 1);
        }
    }
}

#define COLLIDERS_LEN 20
struct Colliders {
    uint32_t width[COLLIDERS_LEN];
    uint32_t height[COLLIDERS_LEN];
    Vec2 tl[COLLIDERS_LEN];
    int len;
};

typedef int ColliderId;

static bool colliding(const Colliders* c, ColliderId c1, ColliderId c2)
{
#if DEBUG
    if (c1 >= c->len || c2 >= c->len) {
        printf(
            "colliding: ColliderId outside len of Colliders. c1: %i, c2: %i, "
            "len: %i",
            c1,
            c2,
            c->len);
        PANIC;
    }
#endif

    if (c->tl[c1].x < c->tl[c2].x + c->width[c2] &&
        c->tl[c2].x < c->tl[c1].x + c->width[c1]) {
        if (c->tl[c1].y < c->tl[c2].y + c->height[c2] &&
            c->tl[c2].y < c->tl[c1].y + c->height[c1]) {
            return true;
        }
    }

    return false;
}

static void draw_line(Vec2 start, Vec2 end)
{
#if DEBUG
    if (start.x != end.x && start.y != end.y) {
        printf(
            "draw_line: expected straight line. start { x: %i, y: %i }, end { "
            "x: %i, y: %i }\n",
            start.x,
            start.y,
            end.x,
            end.y);
        PANIC;
    }
#endif

    if (start.x == end.x) {
        int sy;
        int by;

        if (start.y > end.y) {
            sy = end.y;
            by = start.y;
        } else {
            sy = start.y;
            by = end.y;
        }

        for (int y = sy; y <= by; y++) {
            if (y < SCREEN_HEIGHT && y >= 0) {
                draw_pixel(start.x, y, 1);
            }
        }
    } else {
        int sx;
        int bx;

        if (start.x > end.x) {
            sx = end.x;
            bx = start.x;
        } else {
            sx = start.x;
            bx = end.x;
        }

        for (int x = sx; x <= bx; x++) {
            if (x < SCREEN_WIDTH && x >= 0) {
                draw_pixel(x, start.y, 1);
            }
        }
    }
}

static void draw_frame()
{
    draw_line(vec2(1, 1), vec2(SCREEN_WIDTH - 2, 1));
    draw_line(vec2(1, SCREEN_HEIGHT - 2),
              vec2(SCREEN_WIDTH - 2, SCREEN_HEIGHT - 2));
    draw_line(vec2(1, 1), vec2(1, SCREEN_HEIGHT - 2));
    draw_line(vec2(SCREEN_WIDTH - 2, 1),
              vec2(SCREEN_WIDTH - 2, SCREEN_HEIGHT - 2));
}

/// Determines if button `b` is currently pressed.
///
/// Caller ensures that `b` is 0, 1, 2, or 3.
static bool read_button(int b)
{
#if DEBUG
    if (b < 0 || b > 3) {
        printf("Invalid button: %i\n", b);
        PANIC;
    }
#endif
    return (*BUTTON >> b) & 1;
}

struct Player {
    Vec2 position;
    bool grounded;
};

static void move_player(Colliders* colliders,
                        Player* player,
                        Vec2 dt,
                        int player_width,
                        int player_height)
{
    Vec2 result = vec2(player->position.x + dt.x, player->position.y + dt.y);
    colliders->tl[0] = result;

    for (int i = 1; i < colliders->len; i++) {
        if (colliding(colliders, 0, i)) {
            // TODO: Might catch edges here.
            if (colliders->tl[i].y == player->position.y + player_height) {
                player->grounded = true;
            }

            colliders->tl[0] = player->position;
            return;
        }
    }

    player->position = result;
}

static uint32_t get_current_tick()
{
    return *((uint32_t*)TICK);
}

static int secsf_to_ticks(float secs)
{
    return secs * TICK_RATE;
}

static int secs_to_ticks(int secs)
{
    return secs * TICK_RATE;
}

typedef void (*TimerCallback)();

struct Timer {
    uint32_t interval;
    uint32_t tick;
    bool finished;
};

// #define TIMER_COUNT 10
// static Timer timers[TIMER_COUNT];

// static void update_timers()
// {
//     uint32_t current_tick = get_current_tick();
//     for (int i = 0; i < TIMER_COUNT; i++) {
//         if (timers[i].active) {
//             if (timers[i].tick + current_tick > timers[i].interval) {
//                 timers[i].callback();
//                 timers[i].tick = current_tick;
//             }
//         }
//     }
// }
//
// static int set_timer(uint32_t interval, TimerCallback callback)
// {
//     uint32_t current_tick = get_current_tick();
//     for (int i = 0; i < TIMER_COUNT; i++) {
//         if (!timers[i].active) {
//             timers[i].tick = current_tick;
//             timers[i].interval = interval;
//             timers[i].callback = callback;
//             timers[i].active = true;
//
//             return i;
//         }
//     }
//
//     return -1;
// }

static void update_timer(Timer* timer)
{
    if (!timer->finished) {
        uint32_t current_tick = get_current_tick();
        if (current_tick >= timer->interval) {
            timer->finished = true;
        }

        timer->tick = current_tick;
    }
}

static bool timer_finished(Timer* timer)
{
    if (timer->finished) {
        timer->finished = false;
        return true;
    } else {
        return false;
    }
}

const uint8_t smiley_bmp[8] = {0b00111100,
                               0b01000010,
                               0b10100101,
                               0b10000001,
                               0b10100101,
                               0b10011001,
                               0b01000010,
                               0b00111100};

struct Obstacle {
    Rect rect;
    ColliderId collider;
};

static ColliderId push_collider(Colliders* colliders,
                                int width,
                                int height,
                                Vec2 position)
{
    int i = colliders->len++;

#if DEBUG
    if (i >= COLLIDERS_LEN) {
        printf("push_collider: Colliders out of space. len: %i", i);
        PANIC;
    }
#endif

    colliders->height[i] = height;
    colliders->width[i] = width;
    colliders->tl[i] = position;

    return i;
}

static Obstacle obstable(Colliders* colliders,
                         int width,
                         int height,
                         Vec2 position)
{
    Rect rect;
    rect.height = height;
    rect.width = width;
    rect.position = position;

    Obstacle obstacle;
    obstacle.rect = rect;
    obstacle.collider = push_collider(colliders, width, height, position);
    return obstacle;
}

int main()
{
    BitMap bmp = {
        .pixels = (uint8_t*)smiley_bmp,
        .width = 8,
        .height = 8,
    };

    Vec2 pos = {
        .x = 10,
        .y = 10,
    };
    Player player = {.position = pos, .grounded = false};

    Timer jump_timer = {};
    jump_timer.interval = secsf_to_ticks(0.25);
    jump_timer.tick = get_current_tick();
    jump_timer.finished = true;

    bool apply_jump_force;

    bool button_held;

    Colliders colliders;

    // PLAYER
    colliders.height[0] = 8;
    colliders.width[0] = 8;
    colliders.tl[0] = player.position;

    // FLOOR
    colliders.height[1] = 2;
    colliders.width[1] = SCREEN_WIDTH;
    colliders.tl[1] = vec2(0, SCREEN_HEIGHT - 2);

    // LEFT WALL
    colliders.height[2] = SCREEN_HEIGHT;
    colliders.width[2] = 2;
    colliders.tl[2] = vec2(0, 0);

    // RIGHT WALL
    colliders.height[3] = SCREEN_HEIGHT;
    colliders.width[3] = 2;
    colliders.tl[3] = vec2(SCREEN_WIDTH - 2, 0);

    colliders.len = 4;

#define OBSTACLE_LEN 3
    Obstacle obstacles[OBSTACLE_LEN];
    obstacles[0] =
        obstable(&colliders, 50, 10, vec2(42, SCREEN_HEIGHT - 12 - 20));
    obstacles[1] =
        obstable(&colliders, 50, 10, vec2(120, SCREEN_HEIGHT - 12 - 50));
    obstacles[2] =
        obstable(&colliders, 50, 10, vec2(180, SCREEN_HEIGHT - 12 - 90));

#define PLAYER_SPEED 2

    for (;;) {
        clear_screen();

        update_timer(&jump_timer);

        if (read_button(0) && !player.grounded && apply_jump_force) {
            if (timer_finished(&jump_timer)) {
                apply_jump_force = false;
            }

            move_player(&colliders, &player, vec2(0, -1 * PLAYER_SPEED), 8, 8);
        } else {
            apply_jump_force = false;
            move_player(&colliders, &player, vec2(0, 1), 8, 8);
        }

        if (read_button(0) && player.grounded && !button_held) {
            player.grounded = false;
            jump_timer.finished = false;
            jump_timer.interval = get_current_tick() + secsf_to_ticks(0.75);
            apply_jump_force = true;
        }

        button_held = read_button(0);

        if (read_button(1)) {
            move_player(&colliders, &player, vec2(-1 * PLAYER_SPEED, 0), 8, 8);
        } else if (read_button(3)) {
            move_player(&colliders, &player, vec2(1 * PLAYER_SPEED, 0), 8, 8);
        }

        draw_bitmap(&bmp, player.position.x, player.position.y);

        for (int i = 0; i < OBSTACLE_LEN; i++) {
            draw_rect(&obstacles[i].rect);
        }

        draw_frame();

        RENDER;
    }

    return 0;
}
