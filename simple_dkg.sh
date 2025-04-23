#!/usr/bin/env bash
set -xeuo pipefail
# defaults (override with flags)
CLI_PATH="./target/release/fedimint-dkg-cli"
PASSWORD="testpass"
API_SUFFIX=""
# API_SUFFIX must be provided via -s
FED_NAME="test federation"

# show usage/help
usage() {
  cat <<EOF
Usage: $(basename "$0") [-c CLI_PATH] [-p PASSWORD] [-s SUFFIX] [-n FED_NAME] [-h|--help]

Options:
  -c CLI_PATH    Path to fedimint-dkg-cli binary (default: ${CLI_PATH})
  -p PASSWORD    API password (default: ${PASSWORD})
  -s SUFFIX      API suffix
  -n FED_NAME    Federation name (default: ${FED_NAME})
  -h, --help     Show this help message
EOF
}

# parse options
while getopts ":c:p:s:n:h-:" opt; do
  case $opt in
    c) CLI_PATH="$OPTARG" ;;
    p) PASSWORD="$OPTARG" ;;
    s) API_SUFFIX="$OPTARG" ;;
    n) FED_NAME="$OPTARG" ;;
    h) usage; exit 0 ;;
    -) case "$OPTARG" in help) usage; exit 0 ;; *) echo "Invalid option: --$OPTARG" >&2; usage; exit 1 ;; esac ;;
    \?) echo "Invalid option: -$OPTARG" >&2; usage; exit 1 ;;
    :) echo "Option -$OPTARG requires an argument." >&2; usage; exit 1 ;;
  esac
done
shift $((OPTIND-1))

# ensure API_SUFFIX provided
if [[ -z "$API_SUFFIX" ]]; then
  echo "Error: API suffix is required (-s)." >&2
  usage
  exit 1
fi

# helper: return API endpoint for a peer
api_url() {
  local peer="$1"
  echo "wss://api.${peer}.${API_SUFFIX}/"
}

# your four guardians
peers=(alpha bravo charlie delta)

# 1) fetch & store each peer’s setup‐code (only first sets federation name)
declare -A codes
for p in "${peers[@]}"; do
  if [[ "$p" == "${peers[0]}" ]]; then
    # first peer: include federation name
    codes[$p]=$("$CLI_PATH" \
      --password "$PASSWORD" \
      set-local-params \
        --api-url "$(api_url "$p")" \
        -g "$p" \
        -f "$FED_NAME" \
      | awk '{print $NF}')
  else
    # others: skip federation flag
    codes[$p]=$("$CLI_PATH" \
      --password "$PASSWORD" \
      set-local-params \
        --api-url "$(api_url "$p")" \
        -g "$p" \
      | awk '{print $NF}')
  fi
done

# 2) add everyone else’s code to each guardian
for src in "${peers[@]}"; do
  for dst in "${peers[@]}"; do
    [[ $src == $dst ]] && continue
    "$CLI_PATH" \
      --password "$PASSWORD" \
      add-peer \
      --api-url "$(api_url "$src")" \
      --peer-info \
      "${codes[$dst]}"
  done
done

# 3) start DKG on all peers
for p in "${peers[@]}"; do
  "$CLI_PATH" \
    --password "$PASSWORD" \
    start-dkg \
    --api-url "$(api_url "$p")"
done