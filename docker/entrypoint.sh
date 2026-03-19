#!/bin/sh
set -eu

APP_DIR="${APP_DIR:-/app}"
DATA_DIR="${DATA_DIR:-${APP_DIR}/data}"
DEFAULT_CONFIG="${APP_DIR}/config.defaults.toml"
CONFIG_PATH="${DATA_DIR}/config.toml"
TOKEN_PATH="${DATA_DIR}/token.json"
APP_USER="${APP_USER:-appuser}"

mkdir -p "${DATA_DIR}"

if [ "$(id -u)" = "0" ]; then
  echo "[entrypoint] running as root, ensuring ${DATA_DIR} is writable by ${APP_USER}"
  if ! chown -R "${APP_USER}:${APP_USER}" "${DATA_DIR}"; then
    echo "[entrypoint] failed to change ownership for ${DATA_DIR}" >&2
    exit 1
  fi
  if [ ! -w "${DATA_DIR}" ]; then
    echo "[entrypoint] ${DATA_DIR} is not writable after ownership fix" >&2
    exit 1
  fi
fi

if [ ! -f "${CONFIG_PATH}" ] && [ -f "${DEFAULT_CONFIG}" ]; then
  cp "${DEFAULT_CONFIG}" "${CONFIG_PATH}"
  echo "[entrypoint] data/config.toml not found, created from config.defaults.toml"
fi

if [ ! -f "${TOKEN_PATH}" ]; then
  printf '{\n  "ssoBasic": []\n}\n' > "${TOKEN_PATH}"
  echo "[entrypoint] data/token.json not found, created empty ssoBasic pool"
fi

if [ "$(id -u)" = "0" ]; then
  exec gosu "${APP_USER}:${APP_USER}" "$@"
fi

exec "$@"
