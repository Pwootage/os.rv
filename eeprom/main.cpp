#include <cstdint>
#include "elf.h"

#define BI_EMPTY 0x00
#define BI_INT8 0x01
#define BI_INT16 0x02
#define BI_INT32 0x03
#define BI_INT64 0x04
#define BI_INT128 0x05
#define BI_BYTES 0x06
#define BI_OBJECT 0x07
#define BI_END 0xFF

#define COMPONENT_ID_INVOKE 0x00
#define COMPONENT_ID_LIST 0x01

volatile uint8_t * const component_fifo = (volatile uint8_t *)0x10001000;
volatile uint8_t * const component_fifo_write_ready = (volatile uint8_t *)0x10001002;
volatile uint8_t * const eeprom = (volatile uint8_t *)0x20000000;
volatile uint8_t * const eeprom_data = (volatile uint8_t *)0x20010000;
const char *KERNEL_PATH = "/kernel\0";

struct UUID {
    union {
        struct {
            uint64_t hi;
            uint64_t lo;
        };
        char value[16];
    };
};

void write_fifo(int value, uint32_t size) {
    // Little endian makes this simple
    const char *data = (const char *)&value;
    for (uint32_t i = 0; i < size; i++) {
        *component_fifo = data[i];
    }
}
void write_fifo(const void *ptr, uint32_t size) {
    const char *data = (const char *)ptr;
    for (uint32_t i = 0; i < size; i++) {
        *component_fifo = data[i];
    }
}

void write_fifo_int8(uint8_t value) {
    write_fifo(BI_INT8, 1);
    write_fifo(value, 1);
}
void write_fifo_int32(uint32_t value) {
    write_fifo(BI_INT32, 1);
    write_fifo(value, 4);
}
void write_fifo_string(const char *str) {
    // Figure out length
    int len = 0;
    while (str[len] != 0) len++;
    write_fifo(BI_BYTES, 1);
    write_fifo(len, 4);
    write_fifo(str, len);
}
void write_fifo_end() {
    write_fifo(BI_END, 1);
}
void write_fifo_ready() {
    *component_fifo_write_ready = 1;
    asm("ebreak");
}

void memcpy(void *dest, const void *src, uint32_t size) {
    char *d = (char*)dest;
    const char *s = (const char*)src;
    for (uint32_t i = 0; i < size; i++)  {
        d[i] = s[i];
    }
}

int fopen(const char *target_fs, const char *path) {
    // Invoke

    write_fifo_int8(COMPONENT_ID_INVOKE);
    write_fifo_string(target_fs);
    write_fifo_string("open\0");
    write_fifo_string(path);
    write_fifo_end();
    write_fifo_ready(); 
}

bool read_kernel(const char *target_fs) {
    int handle = fopen(target_fs, KERNEL_PATH);

    return handle > 0;
}

extern "C" {
    int main() {
        // the EEPROM is mapped directly to memory; read out the boot partition ID (up to 31 chars)
        char bootID[32];
        for (int i = 0; i < sizeof(bootID); i++) {
            bootID[i] = eeprom_data[i];
        }
        // Attempt to read /kernel
        bool res = read_kernel(bootID);

        // Hang
        while (1) {}
        return 1;
    }
}