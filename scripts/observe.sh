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
command_line="$(printf '%q ' "$@")"

fifo="$(mktemp "$log_root/.observe.XXXXXX")"
rm -f "$fifo"
mkfifo "$fifo"

cleanup() {
  rm -f "$fifo"
}
trap cleanup EXIT

printf '==> logging %s to %s\n' "$command_name" "$log_file" | tee -a "$log_file"
printf '==> command: %s\n' "$command_line" | tee -a "$log_file"
printf '==> heartbeat every %ss\n' "$heartbeat_seconds" | tee -a "$log_file"

tee -a "$log_file" <"$fifo" &
tee_pid=$!

{
  printf '==> %s started at %s\n' "$command_name" "$(date -u +%Y-%m-%dT%H:%M:%SZ)"
  "$@"
} >"$fifo" 2>&1 &
cmd_pid=$!

last_size=0
stalled_heartbeats=0
heartbeat_report() {
  size=$(wc -c <"$log_file" 2>/dev/null || printf '0')
  delta=$((size - last_size))
  last_size=$size
  if [ "$delta" -gt 0 ]; then
    stalled_heartbeats=0
  else
    stalled_heartbeats=$((stalled_heartbeats + 1))
  fi

  state_line="$(ps -o pid=,ppid=,etime=,stat=,pcpu=,pmem=,cmd= -p "$cmd_pid" 2>/dev/null | sed 's/^ *//')"
  child_lines="$(ps -o pid=,ppid=,etime=,stat=,pcpu=,pmem=,cmd= --ppid "$cmd_pid" 2>/dev/null | sed 's/^ *//')"

  {
    printf '==> %s heartbeat: %s bytes logged (+%s), %s quiet ticks\n' \
      "$command_name" "$size" "$delta" "$stalled_heartbeats"
    if [ -n "$state_line" ]; then
      printf '==> %s process: %s\n' "$command_name" "$state_line"
    fi
    if [ -n "$child_lines" ]; then
      printf '==> %s children:\n%s\n' "$command_name" "$child_lines"
    fi
  } | tee -a "$log_file"
}

while kill -0 "$cmd_pid" 2>/dev/null; do
  sleep "$heartbeat_seconds"
  if kill -0 "$cmd_pid" 2>/dev/null; then
    heartbeat_report
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
