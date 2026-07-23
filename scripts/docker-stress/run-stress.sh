#!/usr/bin/env bash
set -euo pipefail

echo "=== Building wm-server image ==="
docker compose build

echo "=== Starting wm-server (768MB limit) ==="
docker compose up -d
trap "docker compose down" EXIT

echo "=== Waiting for health check ==="
for i in $(seq 1 12); do
  if curl -sf http://localhost:4090/api/health > /dev/null 2>&1; then
    echo "Server ready"
    break
  fi
  sleep 2
done

echo "=== Running stress: 10 concurrent search queries ==="
for i in $(seq 1 10); do
  curl -s -X POST http://localhost:4090/api/search/query \
    -H 'Content-Type: application/json' \
    -d '{"q":"test","limit":5}' > /dev/null &
done
wait

echo "=== Checking OOM status ==="
CONTAINER_ID=$(docker compose ps -q wm-server)
OOM_KILLED=$(docker inspect "$CONTAINER_ID" --format '{{.State.OOMKilled}}')
EXIT_CODE=$(docker inspect "$CONTAINER_ID" --format '{{.State.ExitCode}}')

echo "OOMKilled: $OOM_KILLED"
echo "Exit code: $EXIT_CODE"

if [ "$OOM_KILLED" = "true" ]; then
  echo "FAIL: Server was OOM-killed at 768MB limit"
  exit 1
else
  echo "PASS: Server survived within 768MB"
fi
