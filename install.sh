#!/usr/bin/env bash

set -euo pipefail

REPO="${UGH_INSTALL_REPO:-FilipKDick/ugh}"
VERSION="${UGH_INSTALL_VERSION:-latest}"
DEFAULT_DEST="/usr/local/bin"
DEST="${UGH_INSTALL_DIR:-$DEFAULT_DEST}"

detect_target() {
  local os arch
  os="$(uname -s)"
  arch="$(uname -m)"

  case "${os}:${arch}" in
    Darwin:x86_64) echo "x86_64-apple-darwin" ;;
    Darwin:arm64) echo "aarch64-apple-darwin" ;;
    Linux:x86_64) echo "x86_64-unknown-linux-gnu" ;;
    *) echo "Unsupported platform (${os} ${arch})" >&2; exit 1 ;;
  esac
}

fetch_download_url() {
  local api_url="$1" target="$2"
  python3 - "$target" <<'PY'
import json, sys

target = sys.argv[1]
data = json.load(sys.stdin)
assets = data.get("assets", [])
for asset in assets:
    if asset.get("name") == f"ugh-{target}.tar.gz":
        print(asset.get("browser_download_url", ""))
        sys.exit(0)
print("")
PY
}

main() {
  local target api_url release_json download_url tmp_dir archive_path binary_path

  command -v curl >/dev/null 2>&1 || { echo "curl is required to install ugh." >&2; exit 1; }
  command -v python3 >/dev/null 2>&1 || { echo "python3 is required to install ugh." >&2; exit 1; }

  target="$(detect_target)"

  if [[ "$VERSION" == "latest" ]]; then
    api_url="https://api.github.com/repos/${REPO}/releases/latest"
  else
    api_url="https://api.github.com/repos/${REPO}/releases/tags/${VERSION}"
  fi

  echo "🔍 Fetching release metadata (${VERSION}) for ${target}…"
  release_json="$(curl -fsSL "${api_url}")" || {
    echo "Failed to fetch release information from ${api_url}" >&2
    exit 1
  }

  download_url="$(printf "%s" "${release_json}" | fetch_download_url /dev/stdin "${target}")"
  if [[ -z "${download_url}" ]]; then
    echo "Could not locate a release artifact matching target '${target}'." >&2
    exit 1
  fi

  tmp_dir="$(mktemp -d)"
  archive_path="${tmp_dir}/ugh.tar.gz"
  trap 'rm -rf "${tmp_dir}"' EXIT

  echo "⬇️  Downloading ${download_url}"
  curl -fsSL "${download_url}" -o "${archive_path}"

  echo "📦 Extracting archive…"
  tar -xzf "${archive_path}" -C "${tmp_dir}"

  binary_path="$(find "${tmp_dir}" -maxdepth 1 -type f -name "ugh-*")"
  if [[ -z "${binary_path}" ]]; then
    echo "Failed to locate extracted binary in archive." >&2
    exit 1
  fi

  chmod +x "${binary_path}"

  if [[ ! -d "${DEST}" ]]; then
    mkdir -p "${DEST}"
  fi

  install_cmd="mv"
  if [[ ! -w "${DEST}" ]]; then
    echo "🔐 Destination ${DEST} requires elevated permissions; attempting with sudo."
    install_cmd="sudo mv"
  fi

  ${install_cmd} "${binary_path}" "${DEST}/ugh"

  echo "✅ Installed ugh to ${DEST}/ugh"

  if ! command -v ugh >/dev/null 2>&1; then
    echo "ℹ️  Add ${DEST} to your PATH (e.g., export PATH=\"${DEST}:\$PATH\") to invoke 'ugh' globally."
  fi

  echo "Run 'ugh config init' to complete setup."
}

main "$@"
