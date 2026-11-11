#!/usr/bin/env bash
# Install apt packages on a GitHub-hosted Ubuntu runner.
# The runner's default mirror (azure.archive.ubuntu.com, listed in
# /etc/apt/apt-mirrors.txt and the sources files) sometimes stops resolving
# or stalls, which left jobs hanging for an hour on 2026-11-11. Each try is
# time-boxed; if the Azure mirror does not resolve, or a try fails, apt is
# pointed at the main Ubuntu archive instead.
set -euo pipefail

opts=(-o Acquire::http::Timeout=20 -o Acquire::Retries=2)

attempt() {
  sudo timeout 240 apt-get "${opts[@]}" update &&
    sudo timeout 600 apt-get "${opts[@]}" install -y "$@"
}

use_main_archive() {
  echo "::warning::azure.archive.ubuntu.com is unusable; using archive.ubuntu.com"
  sudo find /etc/apt -type f \( -name '*.txt' -o -name '*.list' -o -name '*.sources' \) \
    -exec sed -i 's|http://azure.archive.ubuntu.com/ubuntu|http://archive.ubuntu.com/ubuntu|g' {} +
}

if ! getent hosts azure.archive.ubuntu.com >/dev/null; then
  use_main_archive
  attempt "$@"
  exit 0
fi

if attempt "$@"; then
  exit 0
fi
use_main_archive
attempt "$@"
