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
#define BI_VALUE 0x08
#define BI_END 0xFF

#define COMPONENT_ID_INVOKE 0x00
#define COMPONENT_ID_LIST 0x01
#define COMPONENT_ID_DESTROY_VALUE 0x02

#define UUID_LEN 38

volatile uint8_t * const component_fifo = (volatile uint8_t *)0x10001000;
volatile uint8_t * const component_fifo_write_ready = (volatile uint8_t *)0x10001002;
volatile uint8_t * const panic_fifo = (volatile uint8_t *)0x10002000;
volatile uint8_t * const panic_fifo_write_ready = (volatile uint8_t *)0x10002002;
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

void write_panic(const char *ptr) {
    for (const char *data = ptr; *data != 0; data++) {
        *panic_fifo = *data;
    }
}

void panic(int line, const char *msg) {
    while (1) {
        write_panic("EEPROM PANIC line ");
        int digit = 1000000;
        bool started = false;
        while (digit > 0) {
            char p = ((line / digit) % 10) + '0';
            if (p != '0') started = true;
            if (started) *panic_fifo = p;
            digit /= 10;
        }
        write_panic(" - ");
        write_panic(msg);
        *panic_fifo_write_ready = 1;
        asm("ebreak");
    }
}

#define PANIC() panic(__LINE__, "(unknown)");
#define PANIC_MSG(msg) panic(__LINE__, msg);

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
void write_fifo_value(uint32_t value) {
    write_fifo(BI_VALUE, 1);
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


void read_fifo(void *dest, uint32_t size) {
    if (size == 0) return;
    char *data = (char *)dest;
    for (uint32_t i = 0; i < size; i++) {
        data[i] = *component_fifo;
    }
}

uint8_t read_fifo_int8() {
    uint8_t res;
    read_fifo(&res, 1);
    return res;
}
uint32_t read_fifo_int32() {
    uint32_t res;
    read_fifo(&res, 4);
    return res;
}
uint32_t read_fifo_value() {
    uint32_t res;
    read_fifo(&res, 4);
    return res;
}
void read_fifo_string(uint32_t maxLen, char *dest) {
    uint32_t len;
    read_fifo(&len, 4);
    if (maxLen == 0) {
        for (uint32_t i = 0; i < len; i++) {
            read_fifo_int8();
        }
    } else if (len > maxLen - 1) {
        read_fifo(dest, maxLen - 1);
        for (uint32_t i = len - maxLen - 1; i < len; i++) {
            read_fifo_int8();
        }
        dest[maxLen] = '\0';
    } else {
        read_fifo(dest, len);
        dest[len] = '\0';
    }
}

void memcpy(void *dest, const void *src, uint32_t size) {
    char *d = (char*)dest;
    const char *s = (const char*)src;
    for (uint32_t i = 0; i < size; i++)  {
        d[i] = s[i];
    }
}


void dispose_value(uint32_t value) {
    write_fifo_int8(COMPONENT_ID_DESTROY_VALUE);
    write_fifo_value(value);
    write_fifo_end();
    write_fifo_ready(); 

    if (read_fifo_int8() != BI_INT8) PANIC();
    read_fifo_int8(); // ignore success/errorerror
    if (read_fifo_int8() != BI_END) PANIC();
}
uint32_t fopen(const char *target_fs, const char *path) {
    // Invoke
    write_fifo_int8(COMPONENT_ID_INVOKE);
    write_fifo_string(target_fs);
    write_fifo_string("open\0");
    write_fifo_string(path);
    write_fifo_end();
    write_fifo_ready(); 

    if (read_fifo_int8() != BI_INT8) PANIC();
    bool error = read_fifo_int8();
    if (error) {
        if (read_fifo_int8() != BI_BYTES) PANIC();
        read_fifo_string(0, nullptr); // ignore string
        if (int r = read_fifo_int8() != BI_END) PANIC();
        return -1;
    } else {
        if (read_fifo_int8() != BI_VALUE) PANIC();
        uint32_t handle = read_fifo_value();
        if (read_fifo_int8() != BI_END) PANIC();
        return handle;
    }
}
void fclose(const char *target_fs, uint32_t handle) {
    // Invoke
    write_fifo_int8(COMPONENT_ID_INVOKE);
    write_fifo_string(target_fs);
    write_fifo_string("close\0");
    write_fifo_value(handle);
    write_fifo_end();
    write_fifo_ready(); 

    if (read_fifo_int8() != BI_INT8) PANIC();
    bool error = read_fifo_int8();
    if (error) {
        if (read_fifo_int8() != BI_BYTES) PANIC();
        read_fifo_string(0, nullptr); // ignore string
        if (int r = read_fifo_int8() != BI_END) PANIC();
    } else {
        if (read_fifo_int8() != BI_END) PANIC();
    }

    dispose_value(handle);
}


int find_kernel(char *uuidOut) {
    // call the list api
    write_fifo_int8(COMPONENT_ID_LIST);
    write_fifo_string("filesystem");
    write_fifo_end();
    write_fifo_ready();

    constexpr int max_uuid = 8;
    char uuids[max_uuid][UUID_LEN];
    int current_uuid = 0;
    while (true) {
        uint8_t type = read_fifo_int8();
        if (type == BI_END) break;

        if (type != BI_BYTES) PANIC();
        read_fifo_string(0, nullptr); // Read out the type and ignore it

        if (read_fifo_int8() != BI_BYTES) PANIC();
        if (current_uuid < max_uuid) {
            read_fifo_string(sizeof(uuids[current_uuid]), uuids[current_uuid]); // read the UUID
            current_uuid++;
        } else {
            read_fifo_string(0, nullptr); // ignore extra UUIDs (we just must parse the ENTIRE response)
        }
    }

    for (int i = 0; i < current_uuid; i++) {
        int handle = fopen(uuids[i], KERNEL_PATH);
        if (handle > 0) {
            memcpy(uuidOut, uuids[i], UUID_LEN);
            return handle;
        }
    }

    return -1;
}

bool read_kernel() {
    // the EEPROM is mapped directly to memory; read out the boot partition ID (up to 31 chars)
    char bootID[UUID_LEN];
    for (int i = 0; i < sizeof(bootID); i++) {
        bootID[i] = eeprom_data[i];
    }

    int handle = fopen(bootID, KERNEL_PATH);

    if (handle < 0) {
        // Try to find the right one
        handle = find_kernel(bootID);
    }

    if (handle < 0) {
        PANIC_MSG("No boot medium found");
    }

    // TODO: Load kernel into memory

    // Cleanup
    fclose(bootID, handle);

    return handle > 0;
}

extern "C" {
    int main() {
        // Attempt to read /kernel from any FS
        bool res = read_kernel();

        // Hang
        PANIC_MSG("Did not jump to kernel");
        return 1;
    }
}