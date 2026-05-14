#!/usr/bin/env bash

# Run a long-lived command with live terminal output, a log file, and periodic
# heartbeats so stalled runs are still observable.
#
# Usage:
#   ./scripts/observe.sh cargo test
#   ./scripts/observe.sh cargo build --verbose
#
# Log files are written under ${RRCAD_LOG_DIR:-${TMPDIR:-/tmp}/rrcad-logs}.

set -euo pipefail

if [ "$#" -eq 0 ]; then
  echo "usage: $0 <command> [args...]" >&2
  exit 2
fi

log_root="${RRCAD_LOG_DIR:-${TMPDIR:-/tmp}/rrcad-logs}"
heartbeat_seconds="${RRCAD_HEARTBEAT_SECS:-30}"

mkdir -p "$log_root"

command_name="$(basename "$1")"
safe_name="$(printf '%s' "$command_name" | tr '/ ' '__' | tr -cd '[:alnum:]_.-')"
stamp="$(date -u +%Y%m%dT%H%M%SZ)"
log_file="$log_root/${safe_name}-${stamp}-$$.log"

fifo="$(mktemp "$log_root/.observe.XXXXXX")"
rm -f "$fifo"
mkfifo "$fifo"

cleanup() {
  rm -f "$fifo"
}
trap cleanup EXIT

printf '==> logging %s to %s\n' "$command_name" "$log_file" | tee -a "$log_file"
printf '==> heartbeat every %ss\n' "$heartbeat_seconds" | tee -a "$log_file"

tee -a "$log_file" <"$fifo" &
tee_pid=$!

{
  printf '==> %s started at %s\n' "$command_name" "$(date -u +%Y-%m-%dT%H:%M:%SZ)"
  "$@"
} >"$fifo" 2>&1 &
cmd_pid=$!

last_size=0
while kill -0 "$cmd_pid" 2>/dev/null; do
  sleep "$heartbeat_seconds"
  if kill -0 "$cmd_pid" 2>/dev/null; then
    size=$(wc -c <"$log_file" 2>/dev/null || printf '0')
    if [ "$size" -gt "$last_size" ]; then
      printf '==> %s still running; %s bytes logged so far; log: %s\n' \
        "$command_name" "$size" "$log_file" | tee -a "$log_file"
    else
      printf '==> %s still running; no new output; log: %s\n' \
        "$command_name" "$log_file" | tee -a "$log_file"
    fi
    last_size=$size
  fi
done

set +e
wait "$cmd_pid"
status=$?
set -e

wait "$tee_pid" || true

if [ "$status" -eq 0 ]; then
  printf '==> %s completed successfully; log: %s\n' "$command_name" "$log_file" | tee -a "$log_file"
else
  printf '==> %s exited with status %d; log: %s\n' "$command_name" "$status" "$log_file" | tee -a "$log_file"
fi

exit "$status"
