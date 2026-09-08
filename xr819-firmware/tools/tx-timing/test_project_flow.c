/* Host-only sanitizer exercise of the native projector's pointer/size boundary. */
#include <assert.h>
#include <stdint.h>
#include <stddef.h>
#include <string.h>
int flow_project(const unsigned char *, size_t, uint64_t, unsigned char *, size_t, size_t *);
static uint32_t state = 0x819;
static uint32_t next(void) { state = state * 1664525u + 1013904223u; return state; }
int main(void)
{
    unsigned char input[8192], output[65536];
    const char valid[] = "worker [003] ... 184.123456789: cw1200_confirmed: id=7 status=6 rate=19 ack=4 flags=0 media=345 queued=179\n";
    for (unsigned iteration = 0; iteration < 20000; iteration++) {
        size_t length = next() % sizeof(input), records = 999;
        size_t capacity = next() % (sizeof(output)+1);
        for (size_t i = 0; i < length; i++) input[i] = next() >> 24;
        if (iteration % 3 == 0) {
            length = sizeof(valid)-1;
            memcpy(input, valid, length);
            if (iteration % 2) input[next()%length] = next() >> 24;
        }
        (void)flow_project(input, length, iteration % 7 ? 1 : UINT64_MAX,
                           output, capacity, &records);
        assert(records <= capacity/64);
    }
    return 0;
}
