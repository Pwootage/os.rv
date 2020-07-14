#include <cstdint>
#include "elf.h"

#define BI_EMPTY 0x00
#define BI_INT8 0x01
#define BI_INT16 0x02
#define BI_INT32 0x03
#define BI_INT64 0x04
#define BI_INT128 0x05
#define BI_ARRAY 0x06
#define BI_OBJECT 0x07
#define BI_END 0xFF

#define COMPONENT_ID_INVOKE 0x01
#define COMPONENT_ID_LIST 0x01

constexpr volatile uint8_t * const component_fifo = (volatile uint8_t *)0x10001000;
constexpr volatile uint8_t * const eeprom = (volatile uint8_t *)0x20000000;
constexpr volatile uint8_t * const eeprom_data = (volatile uint8_t *)0x20010000;
const char *KERNEL_PATH = "/kernel";

struct UUID {
    union {
        struct {
            uint64_t hi;
            uint64_t lo;
        };
        char value[16];
    }
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
    write_fifo(COMPONENT_ID_INVOKE, 1);
}
void write_fifo_int32(uint8_t value) {
    write_fifo(BI_INT8, 1);
    write_fifo(COMPONENT_ID_INVOKE, 1);
}
void write_fifo_uuid(const UUID &value) {
    write_fifo(BI_INT128, 1);
    write_fifo(value.value, 16);
}

void memcpy(void *dest, const void *src, uint32_t size) {
    char *d = (char*)dest;
    const char *s = (const char*)src;
    for (uint32_t i = 0; i < size; i++)  {
        d[i] = s[i];
    }
}

int fopen(const UUID &target_fs, const char *path) {
    // Invoke

    write_fifo()
}

bool read_kernel(char *boot_id) {
    
} 

extern "C" {
    int main() {
        // the EEPROM is mapped directly to memory; read out the boot partition ID
        char boot_id[16];
        for (int i = 0; i < sizeof(boot_id); i++) {
            boot_id[i] = eeprom_data[i];
        }
        // Attempt to read /kernel
        bool res = read_kernel(boot_id);

        // Hang
        while (1) {}
    }
}