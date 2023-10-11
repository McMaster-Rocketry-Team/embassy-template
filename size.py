#!/usr/bin/env python3

import subprocess
import re
import math
import os

FLASH_SIZE_KIB = 512
RAM_SIZE_KIB = 80

build_env = os.environ.copy()
build_env['DEFMT_LOG'] = 'off'

subprocess.run('cargo clean'.split(' '), capture_output=True) # or cargo won't recompile with the new env
subprocess.run('cargo build --release'.split(' '))
result = subprocess.run(
    'cargo bloat --release --crates --split-std --no-relative-size -n 20'.split(' '), capture_output=True)
if result.stderr.decode("utf-8").find("error: could not compile") >= 0:
    exit(1)
print()
print(result.stdout.decode("utf-8")[:-93])

result = subprocess.run(
    'cargo bloat --release --split-std --no-relative-size -n 20'.split(' '), capture_output=True)
print(result.stdout.decode("utf-8")[:-93])

result = subprocess.run(
    'cargo size --release -- -A'.split(' '), capture_output=True)
lines = result.stdout.decode("utf-8").split('\n')

global_bytes = 0
rom_bytes = 0

for line in lines:
    match_result = re.match(
        '^\.([a-zA-Z\._]*)\s*(\d*)\s*0x[1-9][0-9a-f]*$', line)
    if match_result:
        name, size = match_result.groups()
        if name == "bss" or name == "data":
            global_bytes += int(size)
        else:
            rom_bytes += int(size)

subprocess.run('cargo clean'.split(' '), capture_output=True)

print()
print(f"Total size in release mode:")
print(f"Flash: {math.ceil(rom_bytes / 1024)}KiB / {FLASH_SIZE_KIB}KiB ({round(rom_bytes / (FLASH_SIZE_KIB * 1024) * 100)}%)")
print(
    f"Global variables: {math.ceil(global_bytes / 1024)}KiB / {RAM_SIZE_KIB}KiB ({round(global_bytes / (RAM_SIZE_KIB * 1024) * 100)}%)")
print(f"Avaliable RAM: {math.floor(RAM_SIZE_KIB - global_bytes / 1024)}KiB")
