#!/usr/bin/env bash
# Installs the Linux libraries that Tauri builds against.
#
# GitHub's Ubuntu runners fetch packages from azure.archive.ubuntu.com. When
# that mirror (or name lookups for ubuntu.com as a whole) fails, apt retries
# every package slowly and a job can hang for hours. So this script first
# finds a mirror that actually answers, including mirrors outside ubuntu.com,
# points apt at it alone, and fails fast with a clear message if none works.
set -euo pipefail

PACKAGES=(libwebkit2gtk-4.1-dev libappindicator3-dev librsvg2-dev patchelf)
CODENAME="$(. /etc/os-release && echo "$VERSION_CODENAME")"

# Tried in order. Each must carry the -updates and -security pockets too.
MIRRORS=(
  http://azure.archive.ubuntu.com/ubuntu
  http://archive.ubuntu.com/ubuntu
  http://mirrors.edge.kernel.org/ubuntu
  http://mirrors.mit.edu/ubuntu
  http://mirror.us.leaseweb.net/ubuntu
)

APT_OPTIONS=(
  -o Acquire::Retries=2
  -o Acquire::http::Timeout=20
  -o Acquire::https::Timeout=20
)

mirror_works() {
  curl --silent --fail --location --max-time 15 --output /dev/null "$1/dists/$CODENAME-security/Release"
}

use_mirror() {
  local mirror="$1"
  if [ -f /etc/apt/apt-mirrors.txt ]; then
    # The runner's sources use mirror+file:/etc/apt/apt-mirrors.txt; leave only the working mirror in it.
    printf '%s/\n' "$mirror" | sudo tee /etc/apt/apt-mirrors.txt >/dev/null
  fi
  for file in /etc/apt/sources.list /etc/apt/sources.list.d/ubuntu.sources; do
    if [ -f "$file" ]; then
      sudo sed -i -E "s#https?://([a-z0-9.-]+\.)?(archive|security)\.ubuntu\.com/ubuntu/?#$mirror/#g" "$file"
    fi
  done
}

install_from() {
  local mirror="$1"
  echo "Installing from $mirror"
  use_mirror "$mirror"
  sudo apt-get "${APT_OPTIONS[@]}" update &&
    sudo apt-get "${APT_OPTIONS[@]}" install -y "${PACKAGES[@]}"
}

for mirror in "${MIRRORS[@]}"; do
  if ! mirror_works "$mirror"; then
    echo "::warning::$mirror is not answering; trying the next mirror."
    continue
  fi
  if install_from "$mirror"; then
    exit 0
  fi
  echo "::warning::Installing from $mirror failed; trying the next mirror."
done

echo "::error::No Ubuntu package mirror could be reached. Re-run the job later."
exit 1
