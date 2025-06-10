#!/bin/bash
V=$(podman run --timeout=300 -tid localhost/shell2:latest)
sleep 1
URL=$(podman logs "$V"|tr -cd '[:print:]\n'|grep -oP 'https://on.tty-share[^ ]+')
if [[ -z "$URL" ]]; then
  podman kill "$V"
  exit 1
fi
echo "$URL"
