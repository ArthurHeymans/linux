#!/bin/bash
# Soak one class-0 image over a real over-the-air path and capture the pipe
# cursor invariant.
#
# Phase 1: ping flood (sustained back-to-back TX, no server needed)
# Phase 2: iperf v2 uplink, client in the wifi netns -> server on the board's
#          wired end0. The wifi leg is XR819 -> AP -> switch -> end0, so this is
#          real over-the-air traffic (the historical harnesses used 127.0.0.1).
#
#   $1 = local packed firmware image
#   $2 = label used for the remote copy
set -u
IMG=$1
LABEL=$2
SP=/nix/store/nkkj35yh0rmj71bwyz8wn7jg6mkm15bx-sshpass-1.10/bin/sshpass
SSH="$SP -p 1234 ssh -o StrictHostKeyChecking=no -o ConnectTimeout=3 -o ServerAliveInterval=5 -o ServerAliveCountMax=2 root@192.168.0.104"
SCP="$SP -p 1234 scp -o StrictHostKeyChecking=no -o ConnectTimeout=5 -o ServerAliveInterval=5 -o ServerAliveCountMax=2"

# The exit trap installs a recovery image and reboots the board, so a run
# started too soon after a previous one races that reboot: ssh times out, the
# trap fires again, and a genuine result is masked by a phantom failure. Wait
# for the board to answer ssh before touching it.
wait_board_ready() {
  for i in $(seq 1 100); do
    if timeout 20 $SSH 'exit 0' 2>/dev/null; then
      [ "$i" -gt 1 ] && echo "BOARD_READY_AFTER=${i}"
      return 0
    fi
    sleep 3
  done
  echo BOARD_UNREACHABLE
  return 1
}

wait_new_boot() {
  old=$1
  for i in $(seq 1 180); do
    sleep 3
    current=$($SSH 'cat /proc/sys/kernel/random/boot_id' 2>/dev/null || true)
    if [ -n "$current" ] && [ "$current" != "$old" ]; then
      echo "NEW_BOOT_ID=$current WAIT_TICKS=$i"
      return 0
    fi
  done
  return 1
}

recover() {
  echo ===EXIT-BH-RX-TRACE===
  $SSH 'find /sys/kernel/debug/ieee80211 -path "*/cw1200/bh_rx_trace" -exec cat {} \;' 2>/dev/null || true
  echo ===EXIT-STATUS===
  $SSH 'find /sys/kernel/debug/ieee80211 -path "*/cw1200/status" -exec cat {} \;' 2>/dev/null || true
  echo INSTALLING_RECOVERY
  $SSH 'cp /root/xr819-vendor-host-tx-event-drain.bin /lib/firmware/xr819/fw_xr819.bin; sync; systemctl reboot' 2>/dev/null || true
}
# Only arm the recovery trap once the board is known reachable, so a failure to
# come up is reported plainly instead of triggering another reboot.
wait_board_ready || exit 3
trap recover EXIT

$SCP "$IMG" "root@192.168.0.104:/root/$LABEL.bin" || exit 2
old=$($SSH 'cat /proc/sys/kernel/random/boot_id') || exit 2
$SSH "set -e; cp /root/$LABEL.bin /lib/firmware/xr819/fw_xr819.bin; sync; sha256sum /lib/firmware/xr819/fw_xr819.bin; systemctl reboot" 2>/dev/null || true
wait_new_boot "$old" || exit 2

$SSH 'bash -s' <<'REMOTE'
set -u
NS=wifi-test
SERVER=192.168.0.104
systemctl stop netplan-wpa-wlan0.service 2>/dev/null || true
[ ! -f /run/xr819-glz.pid ] || kill "$(cat /run/xr819-glz.pid)" 2>/dev/null || true
pkill -f 'iperf -s' 2>/dev/null || true
ip netns del "$NS" 2>/dev/null || true
XRIF=$(for n in /sys/class/net/*; do [ "$(cat "$n/address" 2>/dev/null)" = 12:42:2a:37:70:07 ] && basename "$n" && break; done)
[ -n "$XRIF" ] || exit 3
xrphy=$(basename "$(readlink -f "/sys/class/net/$XRIF/phy80211")")
dir=/sys/kernel/debug/ieee80211/$xrphy/cw1200
ip netns add "$NS"
ip netns exec "$NS" sysctl -qw net.ipv6.conf.all.disable_ipv6=1
ip netns exec "$NS" sysctl -qw net.ipv6.conf.default.disable_ipv6=1
iw phy "$xrphy" set netns name "$NS"
ip netns exec "$NS" ip link set lo up
ip netns exec "$NS" ip link set "$XRIF" up
rm -rf /run/wpa-glz
awk '/^}/ { print "    disable_ht=1" } { print }' /etc/xr819-glz.conf > /run/xr819-glz-noht.conf
ip netns exec "$NS" wpa_supplicant -B -P /run/xr819-glz.pid -f /run/xr819-cursor-wpa.log -i "$XRIF" -D nl80211 -c /run/xr819-glz-noht.conf
state=UNKNOWN
for i in $(seq 1 35); do
  state=$(ip netns exec "$NS" wpa_cli -p /run/wpa-glz -i "$XRIF" status 2>/dev/null | sed -n 's/^wpa_state=//p')
  echo "T=$i STATE=${state:-UNKNOWN}"
  [ "$state" = COMPLETED ] && break
  sleep 1
done
[ "$state" = COMPLETED ] || exit 10
ip netns exec "$NS" ip addr add 192.168.0.105/24 dev "$XRIF"
ip netns exec "$NS" ip route add default via 192.168.0.1 dev "$XRIF"
ip netns exec "$NS" ip neigh replace 192.168.0.1 lladdr 6a:02:b8:0d:89:70 dev "$XRIF" nud permanent
srvmac=$(cat /sys/class/net/end0/address)
ip netns exec "$NS" ip neigh replace "$SERVER" lladdr "$srvmac" dev "$XRIF" nud permanent
dmesg -C

sample() {
  echo "---SAMPLE-$1"
  echo "COUNTERS:"; cat "$dir/counters" 2>/dev/null || echo "  UNAVAILABLE"
  echo "BH-RX-TRACE:"; cat "$dir/bh_rx_trace" 2>/dev/null | tail -12 || echo "  UNAVAILABLE"
  sed -n '/^BH status/,/^RXed/p' "$dir/status" 2>/dev/null || true
}

sample pre

echo "===PHASE1-PING-FLOOD==="
ip netns exec "$NS" timeout 40 ping -f -s 512 -w 25 "$SERVER" 2>&1 | tail -4
sample after-flood

echo "===PHASE2-IPERF-OTA-UPLINK==="
iperf -s -D -B "$SERVER" >/run/iperf-server.log 2>&1 || echo "SERVER_START_FAILED"
sleep 1
ip netns exec "$NS" timeout 45 iperf -c "$SERVER" -t 25 -i 5 2>&1 | tail -10
echo "IPERF_RC=$?"
pkill -f 'iperf -s' 2>/dev/null || true
sample after-iperf

echo "===PHASE3-PING-CHECK==="
ip netns exec "$NS" timeout 20 ping -c 20 -i 0.2 -W 2 "$SERVER" 2>&1 | tail -3

echo ===STATUS===
cat "$dir/status" || true
echo ===DMESG===
dmesg | grep -E '\[BH\]|Fatal error|WARNING:|rx blew up|exception|Timeout|timeout|Missed interrupt|wsm|WSM|cw1200|bh_error|outstanding' | tail -80
REMOTE
