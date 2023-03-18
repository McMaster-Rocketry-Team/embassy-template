#!/usr/bin/env python3

import subprocess
import re
import math

FLASH_SIZE_KIB = 512
RAM_SIZE_KIB = 256

result = subprocess.run('cargo bloat --release --crates --split-std --no-relative-size'.split(' '), capture_output=True)
if result.stderr.decode("utf-8").find("error: could not compile")>=0:
    print("Compilation failed.")
    exit(1)
print(result.stdout.decode("utf-8")[:-93])

result = subprocess.run('cargo size --release -- -A'.split(' '), capture_output=True)
lines = result.stdout.decode("utf-8").split('\n')

ram_bytes = 0
rom_bytes = 0

for line in lines:
    match_result = re.match('^\.([a-zA-Z\._]*)\s*(\d*)\s*0x[1-9][0-9a-f]*$',line)
    if match_result:
        name, size = match_result.groups()
        if name == "bss" or name == "data":
            ram_bytes += int(size)
        else:
            rom_bytes += int(size)

print(f"Total size in release mode:")
print(f"Flash: {math.ceil(rom_bytes / 1024)}KiB / {FLASH_SIZE_KIB}KiB ({round(rom_bytes / (FLASH_SIZE_KIB * 1024) * 100)}%)")
print(f"RAM: {math.ceil(ram_bytes / 1024)}KiB / {RAM_SIZE_KIB}KiB ({round(ram_bytes / (RAM_SIZE_KIB * 1024) * 100)}%)")