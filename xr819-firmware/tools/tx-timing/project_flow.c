/* Numeric trace projection only. No libc, allocation, syscalls or raw-line output.
 * ABI: complete newline-delimited input, first sequence, bounded 64-byte records.
 * On error, *records still identifies the valid prefix. Caller must fail capture.
 */
#include <stddef.h>
#include <stdint.h>

struct span { const unsigned char *p; size_t n; };
static int equal(struct span s, const char *word)
{
    size_t i = 0;
    while (i < s.n && word[i] && s.p[i] == (unsigned char)word[i]) i++;
    return i == s.n && word[i] == 0;
}
static int digit(unsigned char c) { return c >= '0' && c <= '9'; }
static int wordchar(unsigned char c)
{
    return digit(c) || (c >= 'a' && c <= 'z') ||
        (c >= 'A' && c <= 'Z') || c == '_';
}
static int decimal(const unsigned char **cursor, const unsigned char *end, uint64_t *out)
{
    const unsigned char *p = *cursor;
    uint64_t value = 0;
    if (p == end || !digit(*p)) return 0;
    while (p < end && digit(*p)) {
        unsigned d = *p++ - '0';
        if (value > UINT64_MAX / 10 || (value == UINT64_MAX / 10 && d > UINT64_MAX % 10)) return 0;
        value = value * 10 + d;
    }
    *cursor = p; *out = value;
    return 1;
}
static int number(struct span s, int is_signed, uint32_t *out)
{
    size_t i = 0;
    unsigned base = 10, negative = 0;
    uint64_t value = 0, limit = is_signed ? INT32_MAX : UINT32_MAX;
    if (i < s.n && s.p[i] == '-') { negative = 1; i++; }
    if (negative) { if (!is_signed) return 0; limit = (uint64_t)INT32_MAX + 1; }
    if (i + 1 < s.n && s.p[i] == '0' && s.p[i+1] == 'x') { base = 16; i += 2; }
    if (i == s.n) return 0;
    for (; i < s.n; i++) {
        unsigned char c = s.p[i];
        unsigned d;
        if (digit(c)) d = c - '0';
        else if (c >= 'a' && c <= 'f') d = c - 'a' + 10;
        else if (c >= 'A' && c <= 'F') d = c - 'A' + 10;
        else return 0;
        if (d >= base) return 0;
        value = value * base + d; /* Prior iteration bounds value to 32 bits. */
        if (value > limit) return 0;
    }
    *out = negative ? 0u - (uint32_t)value : (uint32_t)value;
    return 1;
}
static void put(unsigned char *p, uint64_t value, unsigned count)
{
    for (unsigned i = 0; i < count; i++) { p[i] = (unsigned char)value; value >>= 8; }
}
static const char *const events[] = {
    "cw1200_queued", "cw1200_transport", "cw1200_confirmed", "net_dev_queue", "mmc_request_done"
};
static const char *const fields[5][7] = {
    {"id", "previous", "len", "queue", "requeue", 0, 0},
    {"id", "msgid", "len", "data", "done", "result", 0},
    {"id", "status", "rate", "ack", "flags", "media", "queued"},
    {"len", 0, 0, 0, 0, 0, 0},
    {"cmd_opcode", "cmd_err", "data_err", "stop_err", "bytes_xfered", 0, 0}
};
static const unsigned counts[] = {5, 6, 7, 1, 5};

static int line(const unsigned char *p, const unsigned char *end, uint64_t sequence, unsigned char *out)
{
    uint64_t cpu = 0, seconds = 0, fraction = 0;
    unsigned decimals = 0, kind, seen = 0;
    int found = 0;
    if (p == end || *p == '#') return 1; /* ignored */
    const unsigned char *blank = p;
    while (blank < end && (*blank == ' ' || *blank == '\t' || *blank == '\r')) blank++;
    if (blank == end) return 1;
    if ((size_t)(end-p) > 8192) return -1;
    while (p < end) {
        if (*p++ != '[') continue;
        const unsigned char *q = p;
        if (decimal(&q, end, &cpu) && q < end && *q == ']' && cpu <= UINT16_MAX) {
            p = q + 1; found = 1; break;
        }
    }
    if (!found) return -2;
    found = 0;
    while (p < end) {
        const unsigned char *q = p;
        if (digit(*p) && decimal(&q, end, &seconds) && q < end && *q == '.') {
            q++; const unsigned char *begin = q;
            if (decimal(&q, end, &fraction) && q < end && *q == ':' && q-begin <= 9) {
                decimals = (unsigned)(q-begin); p = q+1; found = 1; break;
            }
        }
        p++;
    }
    if (!found || seconds > UINT64_MAX / 1000000000u) return -2;
    while (decimals++ < 9) fraction *= 10;
    uint64_t stamp = seconds * 1000000000u;
    if (UINT64_MAX - stamp < fraction) return -2;
    stamp += fraction;
    while (p < end && *p == ' ') p++;
    const unsigned char *begin = p;
    while (p < end && wordchar(*p)) p++;
    struct span name = {begin, (size_t)(p-begin)};
    if (p == end || *p++ != ':') return -3;
    for (kind = 0; kind < 5 && !equal(name, events[kind]); kind++);
    if (kind == 5 || !sequence) return -3;
    for (unsigned i = 0; i < 64; i++) out[i] = 0;
    put(out, sequence, 8); put(out+8, stamp, 8);
    put(out+16, cpu, 2); put(out+18, kind+1, 2);
    while (p < end) {
        if (!wordchar(*p)) { p++; continue; }
        begin = p;
        while (p < end && wordchar(*p)) p++;
        name = (struct span){begin, (size_t)(p-begin)};
        if (p == end || *p != '=') continue;
        p++; begin = p;
        while (p < end && *p != ' ' && *p != '\t' && *p != '\r') p++;
        struct span value = {begin, (size_t)(p-begin)};
        for (unsigned field = 0; field < counts[kind]; field++) {
            if (!equal(name, fields[kind][field])) continue;
            uint32_t v;
            int is_signed = (kind == 1 && field == 5) || (kind == 4 && field >= 1 && field <= 3);
            if ((seen & (1u << field)) || !number(value, is_signed, &v)) return -4;
            seen |= 1u << field;
            put(out+20+field*4, v, 4);
        }
    }
    return seen == (1u << counts[kind])-1 ? 0 : -4;
}

unsigned flow_project_abi(void) { return 1; }
int flow_project(const unsigned char *input, size_t length, uint64_t first,
                 unsigned char *output, size_t capacity, size_t *records)
{
    size_t position = 0;
    *records = 0;
    if (length > 65536 || !first) return -1;
    while (position < length) {
        size_t start = position;
        while (position < length && input[position] != '\n') position++;
        if (position == length || capacity / 64 <= *records || UINT64_MAX - first < *records) return -1;
        int status = line(input+start, input+position, first+*records, output+*records*64);
        position++;
        if (status < 0) return status;
        if (status == 0) (*records)++;
    }
    return 0;
}
