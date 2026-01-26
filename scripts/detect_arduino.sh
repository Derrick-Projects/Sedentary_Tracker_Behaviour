#!/bin/bash
# detect_arduino.sh: Detects Arduino serial port on Linux

# List possible Arduino serial devices
devices=(/dev/ttyACM* /dev/ttyUSB*)
found=0
for dev in "${devices[@]}"; do
    if [ -e "$dev" ]; then
        echo "Arduino detected on: $dev"
        found=1
    fi
done
if [ $found -eq 0 ]; then
    echo "No Arduino device found (checked /dev/ttyACM* and /dev/ttyUSB*)"
    exit 1
fi
