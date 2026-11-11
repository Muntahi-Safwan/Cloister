#!/usr/bin/env bash
# Installs the Linux libraries that Tauri builds against.
#
# GitHub's Ubuntu runners fetch packages from azure.archive.ubuntu.com. When
# that mirror stops resolving, apt retries every package slowly and a job can
# hang for hours (actions/runner-images#7048). So: use Ubuntu's own mirrors,
# give up on a slow connection quickly, and retry the whole install a few times.
set -euo pipefail

PACKAGES=(libwebkit2gtk-4.1-dev libappindicator3-dev librsvg2-dev patchelf)
APT_OPTIONS=(
  -o Acquire::Retries=3
  -o Acquire::http::Timeout=20
  -o Acquire::https::Timeout=20
)

for file in /etc/apt/apt-mirrors.txt /etc/apt/sources.list /etc/apt/sources.list.d/ubuntu.sources; do
  if [ -f "$file" ]; then
    sudo sed -i 's|http://azure\.archive\.ubuntu\.com/ubuntu|http://archive.ubuntu.com/ubuntu|g' "$file"
  fi
done

for attempt in 1 2 3; do
  if sudo apt-get "${APT_OPTIONS[@]}" update &&
    sudo apt-get "${APT_OPTIONS[@]}" install -y "${PACKAGES[@]}"; then
    exit 0
  fi
  echo "::warning::Installing Linux packages failed (attempt $attempt of 3); retrying in 20 seconds."
  sleep 20
done

echo "::error::Could not install Linux packages. Ubuntu's package mirrors may be down; re-run the job later."
exit 1
