#!/bin/sh
set -eu

TARGET=${XR819_TARGET:-root@192.168.0.105}
DEVICE=${XR819_SDIO_DEVICE:-mmc1:0001:1}
BOOT_IMAGE=${1:?usage: reload-target.sh BOOT_IMAGE MAIN_IMAGE}
MAIN_IMAGE=${2:?usage: reload-target.sh BOOT_IMAGE MAIN_IMAGE}

scp "$BOOT_IMAGE" "$MAIN_IMAGE" "$TARGET:/tmp/"
boot_name=$(basename "$BOOT_IMAGE")
main_name=$(basename "$MAIN_IMAGE")

ssh "$TARGET" "
set -eu
install -m0644 /tmp/$boot_name /lib/firmware/xr819/boot_xr819.bin
install -m0644 /tmp/$main_name /lib/firmware/xr819/fw_xr819.bin
if [ -L /sys/bus/sdio/devices/$DEVICE/driver ]; then
    echo $DEVICE > /sys/bus/sdio/drivers/cw1200_wlan_sdio/unbind
fi
# A startup timeout is returned by the bind write for experimental images.
echo $DEVICE > /sys/bus/sdio/drivers/cw1200_wlan_sdio/bind || true
"
