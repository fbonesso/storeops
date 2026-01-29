#!/bin/sh
set -e

REPO="fbonesso/storeops"
BINARY="storeops"
INSTALL_DIR="${HOME}/.local/bin"

get_arch() {
  arch=$(uname -m)
  case "$arch" in
    x86_64|amd64) echo "x86_64" ;;
    aarch64|arm64) echo "aarch64" ;;
    *) echo "Unsupported architecture: $arch" >&2; exit 1 ;;
  esac
}

get_os() {
  os=$(uname -s)
  case "$os" in
    Linux)  echo "unknown-linux-gnu" ;;
    Darwin) echo "apple-darwin" ;;
    *)      echo "Unsupported OS: $os" >&2; exit 1 ;;
  esac
}

get_latest_version() {
  if command -v curl >/dev/null 2>&1; then
    curl -fsSL "https://api.github.com/repos/${REPO}/releases/latest" | sed -n 's/.*"tag_name": *"\([^"]*\)".*/\1/p'
  elif command -v wget >/dev/null 2>&1; then
    wget -qO- "https://api.github.com/repos/${REPO}/releases/latest" | sed -n 's/.*"tag_name": *"\([^"]*\)".*/\1/p'
  else
    echo "Neither curl nor wget found" >&2
    exit 1
  fi
}

download() {
  url="$1"
  output="$2"
  if command -v curl >/dev/null 2>&1; then
    curl -fsSL "$url" -o "$output"
  elif command -v wget >/dev/null 2>&1; then
    wget -qO "$output" "$url"
  fi
}

verify_checksum() {
  archive_path="$1"
  archive_name="$2"
  checksums_path="$3"

  if ! [ -f "$checksums_path" ]; then
    echo "Warning: checksums file not found, skipping verification" >&2
    return 0
  fi

  expected=$(grep "$archive_name" "$checksums_path" | awk '{print $1}')
  if [ -z "$expected" ]; then
    echo "Warning: no checksum found for ${archive_name}, skipping verification" >&2
    return 0
  fi

  if command -v sha256sum >/dev/null 2>&1; then
    actual=$(sha256sum "$archive_path" | awk '{print $1}')
  elif command -v shasum >/dev/null 2>&1; then
    actual=$(shasum -a 256 "$archive_path" | awk '{print $1}')
  else
    echo "Warning: no sha256 tool found, skipping verification" >&2
    return 0
  fi

  if [ "$actual" != "$expected" ]; then
    echo "Checksum verification FAILED" >&2
    echo "  Expected: ${expected}" >&2
    echo "  Actual:   ${actual}" >&2
    exit 1
  fi

  echo "Checksum verified."
}

main() {
  arch=$(get_arch)
  os=$(get_os)
  target="${arch}-${os}"

  version="${STOREOPS_VERSION:-}"
  if [ -z "$version" ]; then
    echo "Fetching latest release..."
    version=$(get_latest_version)
    if [ -z "$version" ]; then
      echo "Failed to determine latest version" >&2
      exit 1
    fi
  fi

  echo "Installing storeops ${version} for ${target}..."

  archive="storeops-${version}-${target}.tar.gz"
  url="https://github.com/${REPO}/releases/download/${version}/${archive}"
  checksums_url="https://github.com/${REPO}/releases/download/${version}/checksums.sha256"

  tmpdir=$(mktemp -d)
  trap 'rm -rf "$tmpdir"' EXIT

  echo "Downloading ${url}..."
  download "$url" "${tmpdir}/${archive}"

  echo "Downloading checksums..."
  download "$checksums_url" "${tmpdir}/checksums.sha256" 2>/dev/null || true

  echo "Verifying..."
  verify_checksum "${tmpdir}/${archive}" "$archive" "${tmpdir}/checksums.sha256"

  echo "Extracting..."
  tar -xzf "${tmpdir}/${archive}" -C "$tmpdir"

  # Find the binary (may be in a subdirectory)
  bin=$(find "$tmpdir" -name storeops -type f | head -1)
  if [ -z "$bin" ]; then
    echo "storeops binary not found in archive" >&2
    exit 1
  fi

  install_dir="${STOREOPS_INSTALL_DIR:-$INSTALL_DIR}"

  mkdir -p "$install_dir"
  cp "$bin" "${install_dir}/storeops"
  chmod +x "${install_dir}/storeops"

  # Check if install dir is in PATH
  case ":$PATH:" in
    *":${install_dir}:"*) ;;
    *) echo "Add ${install_dir} to your PATH: export PATH=\"${install_dir}:\$PATH\"" ;;
  esac

  echo "Installed storeops ${version} to ${install_dir}"
  echo ""
  echo "Run 'storeops --help' to get started."
}

main
