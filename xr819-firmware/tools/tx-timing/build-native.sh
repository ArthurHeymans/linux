#!/usr/bin/env bash
# Run inside a shell with the ARM hard-float cross compiler available.
set -euo pipefail
umask 077
here=$(cd -- "$(dirname -- "$0")" && pwd)
out=/tmp/xr819-flow-native
mkdir -p "$out"
chmod 700 "$out"
flags=(-std=c11 -O2 -Wall -Wextra -Werror -fPIC -shared -nostdlib
       -fno-builtin -fno-stack-protector -fno-tree-loop-distribute-patterns)
cc "${flags[@]}" "$here/project_flow.c" -o "$out/project_flow-host.so"
armv7l-unknown-linux-gnueabihf-gcc "${flags[@]}" "$here/project_flow.c" -o "$out/project_flow-arm.so"
for file in "$out"/*.so; do
  test -z "$(nm -u "$file")"
  if readelf -d "$file" | grep -q NEEDED; then
    echo 'Unexpected native dependency' >&2; exit 1
  fi
done
cc -std=c11 -O1 -g -Wall -Wextra -Werror -fsanitize=address,undefined \
  -fno-omit-frame-pointer "$here/project_flow.c" "$here/test_project_flow.c" -o "$out/projector-sanitize"
"$out/projector-sanitize"
export XR819_FLOW_NATIVE_TEST_LIB="$out/project_flow-host.so"
python3 -m unittest discover -s "$here" -p 'test_*.py' -v
sha256sum "$here/project_flow.c" | cut -d ' ' -f 1 > "$out/project_flow.source-sha256"
sha256sum "$out/project_flow-arm.so"
