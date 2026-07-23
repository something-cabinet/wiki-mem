# Docker OOM Stress Test Harness

Validates that `wm-server` stays within a 768 MB memory limit under concurrent load.

## What it tests

- Memory usage during concurrent search queries
- OOM killer resilience at a 768 MB container limit
- Server health after load

## Prerequisites

- Docker & Docker Compose

## How to run

```bash
cd scripts/docker-stress && bash run-stress.sh
```

The script will:
1. Build the `wm-server` Docker image
2. Start a container with a 768 MB memory limit
3. Wait for the health check to pass
4. Fire 10 concurrent search queries
5. Inspect the container for OOM status

## Expected result

```
PASS: Server survived within 768MB
```

If the server is OOM-killed, the script exits with code 1 and prints:
```
FAIL: Server was OOM-killed at 768MB limit
```
