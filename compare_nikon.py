#!/usr/bin/env python3
"""Compare fpexif and ExifTool JSON outputs for Nikon D300"""

import json
import sys

# Read ExifTool JSON
with open('/home/user/fpexif/test-data/RAW_NIKON_D300.json') as f:
    exiftool_data = json.load(f)[0]

# Read fpexif JSON
import subprocess
result = subprocess.run(
    ['./target/debug/fpexif', 'list', 'test-data/RAW_NIKON_D300.NEF', '--exiftool-json'],
    capture_output=True, text=True, cwd='/home/user/fpexif'
)
fpexif_data = json.loads(result.stdout)[0]

# Ignore fields
IGNORE = {'SourceFile', 'ExifToolVersion', 'FileName', 'Directory',
          'FileSize', 'FileModifyDate', 'FileAccessDate',
          'FileInodeChangeDate', 'FilePermissions', 'FileType',
          'FileTypeExtension', 'MIMEType', 'MakerNote'}

# Compare
missing = []
mismatched = []
binary_tags = []

for key, exiftool_value in exiftool_data.items():
    if key in IGNORE:
        continue

    fpexif_value = fpexif_data.get(key)

    if fpexif_value is None:
        missing.append((key, exiftool_value))
    elif isinstance(fpexif_value, str) and (' ' in fpexif_value and all(c in '0123456789abcdef ' for c in fpexif_value.lower())):
        # Looks like hex string
        binary_tags.append((key, exiftool_value, fpexif_value))
    elif str(fpexif_value) != str(exiftool_value):
        mismatched.append((key, exiftool_value, fpexif_value))

print("=== MISSING TAGS ===")
for key, value in sorted(missing)[:20]:
    print(f"{key}: {value}")
print(f"\nTotal missing: {len(missing)}\n")

print("=== BINARY/HEX TAGS (need decoding) ===")
for key, expected, actual in sorted(binary_tags)[:20]:
    print(f"{key}:")
    print(f"  Expected: {expected}")
    print(f"  Got (hex): {actual[:80]}...")
print(f"\nTotal binary: {len(binary_tags)}\n")

print("=== MISMATCHED TAGS ===")
for key, expected, actual in sorted(mismatched)[:20]:
    print(f"{key}:")
    print(f"  Expected: {expected}")
    print(f"  Got: {actual}")
print(f"\nTotal mismatched: {len(mismatched)}\n")

print(f"=== SUMMARY ===")
print(f"Total ExifTool tags (excl. ignored): {len([k for k in exiftool_data if k not in IGNORE])}")
print(f"Missing: {len(missing)}")
print(f"Binary (need decode): {len(binary_tags)}")
print(f"Mismatched: {len(mismatched)}")
print(f"Matching: {len([k for k in exiftool_data if k not in IGNORE and k in fpexif_data and str(fpexif_data[k]) == str(exiftool_data[k])])}")
