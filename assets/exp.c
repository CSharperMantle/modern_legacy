#include <stdbool.h>
#include <stdint.h>
#include <stdio.h>

static void encipher(size_t num_rounds, uint64_t v[2], uint64_t const k[4]) {
  uint64_t v0 = v[0], v1 = v[1], sum = 0, delta = 0x9e38538a49;
  for (size_t i = 0; i < num_rounds; i++) {
    v0 += (((v1 << 4) ^ ((v1 & 0xffffffffff) >> 5)) + v1) ^ (sum + k[sum & 3]);
    sum = (sum + delta) & 0xffffffffffff;
    v1 += (((v0 << 4) ^ ((v0 & 0xffffffffff) >> 5)) + v0) ^ (sum + k[(sum >> 11) & 3]);
  }
  v[0] = v0 & 0xffffffffff;
  v[1] = v1 & 0xffffffffff;
}

static void decipher(size_t num_rounds, uint64_t v[2], uint64_t const k[4]) {
  uint64_t v0 = v[0], v1 = v[1], delta = 0x9e38538a49, sum = delta * num_rounds & 0xffffffffff;
  for (size_t i = 0; i < num_rounds; i++) {
    v1 -= (((v0 << 4) ^ ((v0 & 0xffffffffff) >> 5)) + v0) ^ (sum + k[(sum >> 11) & 3]);
    sum = (sum - delta) & 0xffffffffffff;
    v0 -= (((v1 << 4) ^ ((v1 & 0xffffffffff) >> 5)) + v1) ^ (sum + k[sum & 3]);
  }
  v[0] = v0 & 0xffffffffff;
  v[1] = v1 & 0xffffffffff;
}

static const uint32_t CHAR_MAP[56] = {
    0x00000020, 0x00000041, 0x00000042, 0x00000043, 0x00000044, 0x00000045, 0x00000046, 0x00000047, 
    0x00000048, 0x00000049, 0x00000027, 0x0000004A, 0x0000004B, 0x0000004C, 0x0000004D, 0x0000004E, 
    0x0000004F, 0x00000050, 0x00000051, 0x00000052, 0x000000B0, 0x00000022, 0x00000053, 0x00000054, 
    0x00000055, 0x00000056, 0x00000057, 0x00000058, 0x00000059, 0x0000005A, 0x00000030, 0x00000031, 
    0x00000032, 0x00000033, 0x00000034, 0x00000035, 0x00000036, 0x00000037, 0x00000038, 0x00000039, 
    0x0000002E, 0x0000002C, 0x00000028, 0x00000029, 0x0000002B, 0x0000002D, 0x0000002A, 0x0000002F, 
    0x0000003D, 0x00000024, 0x0000003C, 0x0000003E, 0x00000040, 0x0000003B, 0x0000003A, 0x0000201A
};

static char rev_map(uint8_t value) {
  if (value >= sizeof(CHAR_MAP) / sizeof(CHAR_MAP[0])) {
    return '?';
  }
  return CHAR_MAP[value];
}

static const uint64_t KEY[4] = {0x0c1d00050f, 0x01000137, 0x0400022f, 0x65000027};

static uint64_t cipher[] = {
    0x000000058b0e5eda,
    0x000000f48afab6bb,
    0x000000f47bfb8cbf,
    0x0000005fb0c2b766,
    0x0000008a6528f759,
    0x0000007acea379b5,
    0x000000c0850d08ce,
};

int main(void) {
  for (int i = sizeof(cipher) / sizeof(cipher[0]) - 2; i >= 0; i--) {
    decipher(32, &cipher[i], KEY);
  }
  for (size_t i = 0; i < sizeof(cipher) / sizeof(cipher[0]); i++) {
    for (int j = 7; j >= 0; j--) {
      const uint8_t ch = ((uint8_t *)cipher)[i * 8 + j];
      if (ch != 0) {
        putchar(rev_map(ch));
      }
    }
  }
  putchar('\n');
  return 0;
}
