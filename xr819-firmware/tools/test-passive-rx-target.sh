#!/bin/sh
set -eu

TARGET=${XR819_TARGET:-root@192.168.0.105}
MMC_DRIVER=${XR819_MMC_DRIVER:-sunxi-mmc}
MMC_DEVICE=${XR819_MMC_DEVICE:-1c10000.mmc}
BOOT_IMAGE=${1:?usage: test-passive-rx-target.sh BOOT_IMAGE MAIN_IMAGE}
MAIN_IMAGE=${2:?usage: test-passive-rx-target.sh BOOT_IMAGE MAIN_IMAGE}

scp "$BOOT_IMAGE" "$MAIN_IMAGE" "$TARGET:/tmp/"
boot_name=$(basename "$BOOT_IMAGE")
main_name=$(basename "$MAIN_IMAGE")

ssh "$TARGET" "
set -eu
ip link set wlan0 down 2>/dev/null || true
install -m0644 /tmp/$boot_name /lib/firmware/xr819/boot_xr819.bin
install -m0644 /tmp/$main_name /lib/firmware/xr819/fw_xr819.bin
sync

echo $MMC_DEVICE > /sys/bus/platform/drivers/$MMC_DRIVER/unbind
sleep 1
echo $MMC_DEVICE > /sys/bus/platform/drivers/$MMC_DRIVER/bind
sleep 4

ip link set wlan0 up
sleep 1
start=\$(date +%s%N)
set +e
timeout 30 iw dev wlan0 scan > /tmp/xr819-passive-scan.out 2> /tmp/xr819-passive-scan.err
scan_rc=\$?
set -e
end=\$(date +%s%N)

printf 'scan_rc=%s elapsed_ns=%s lines=%s\\n' \
    \"\$scan_rc\" \"\$((end - start))\" \"\$(wc -l < /tmp/xr819-passive-scan.out)\"
cat /tmp/xr819-passive-scan.err
head -120 /tmp/xr819-passive-scan.out
phy=\$(basename /sys/class/ieee80211/phy*)
grep -E 'BH status|RXed|AGG RXed|Used bufs|WSM status|WSM retval|Scan:' \
    /sys/kernel/debug/ieee80211/\$phy/cw1200/status
sha256sum /lib/firmware/xr819/boot_xr819.bin /lib/firmware/xr819/fw_xr819.bin
dmesg | tail -35
"
