#!/bin/sh
set -eu

TARGET=${XR819_TARGET:-root@192.168.0.104}
DEVICE=${XR819_SDIO_DEVICE:-mmc1:0001:1}
BOOT_IMAGE=${1:?usage: reload-target.sh BOOT_IMAGE MAIN_IMAGE}
MAIN_IMAGE=${2:?usage: reload-target.sh BOOT_IMAGE MAIN_IMAGE}

scp "$BOOT_IMAGE" "$MAIN_IMAGE" "$TARGET:/tmp/"
boot_name=$(basename "$BOOT_IMAGE")
main_name=$(basename "$MAIN_IMAGE")

ssh "$TARGET" sh -s -- "$boot_name" "$main_name" "$DEVICE" <<'REMOTE'
set -eu

boot_name=$1
main_name=$2
device=$3

install -m0644 "/tmp/$boot_name" /lib/firmware/xr819/boot_xr819.bin
install -m0644 "/tmp/$main_name" /lib/firmware/xr819/fw_xr819.bin

# Driver removal waits for mac80211 teardown. If association userspace remains
# active, those teardown commands can race the disappearing firmware and leave
# the unbind writer in uninterruptible sleep. Stop association owners first.
killall -TERM wpa_supplicant 2>/dev/null || true
killall -TERM hostapd 2>/dev/null || true
remaining=20
while [ "$remaining" -gt 0 ]; do
    if ! pidof wpa_supplicant >/dev/null 2>&1 &&
       ! pidof hostapd >/dev/null 2>&1; then
        break
    fi
    remaining=$((remaining - 1))
    sleep 1
done
if pidof wpa_supplicant >/dev/null 2>&1 ||
   pidof hostapd >/dev/null 2>&1; then
    echo 'association process did not stop; refusing unsafe SDIO unbind' >&2
    exit 1
fi

list_device_interfaces='device=$1
for path in /sys/class/net/*; do
    [ -L "$path/device" ] || continue
    [ "$(basename "$(readlink -f "$path/device")")" = "$device" ] || continue
    basename "$path"
done'

quiesce_interfaces() {
    namespace=$1
    if [ "$namespace" = root ]; then
        interfaces=$(sh -c "$list_device_interfaces" sh "$device")
        for interface in $interfaces; do
            ip link set "$interface" down
        done
    else
        interfaces=$(ip netns exec "$namespace" \
            sh -c "$list_device_interfaces" sh "$device")
        for interface in $interfaces; do
            ip netns exec "$namespace" ip link set "$interface" down
        done
    fi
}

quiesce_interfaces root
ip netns list | while read -r namespace _; do
    [ -n "$namespace" ] && quiesce_interfaces "$namespace"
done
sleep 1

if [ -L "/sys/bus/sdio/devices/$device/driver" ]; then
    echo "$device" > /sys/bus/sdio/drivers/cw1200_wlan_sdio/unbind
fi
# A startup timeout is returned by the bind write for experimental images.
echo "$device" > /sys/bus/sdio/drivers/cw1200_wlan_sdio/bind || true
REMOTE
