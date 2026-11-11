#!/usr/bin/env bash
# Install apt packages on a GitHub-hosted Ubuntu runner.
# The runner's default mirror (azure.archive.ubuntu.com) sometimes stalls and
# leaves a job hanging for an hour. Each attempt is time-boxed; when the Azure
# mirror fails, the install is retried once against the main Ubuntu archive.
set -euo pipefail

opts=(-o Acquire::http::Timeout=20 -o Acquire::Retries=2)

attempt() {
  sudo timeout 300 apt-get "${opts[@]}" update &&
    sudo timeout 900 apt-get "${opts[@]}" install -y "$@"
}

if attempt "$@"; then
  exit 0
fi

echo "::warning::apt via the Azure mirror failed; retrying on archive.ubuntu.com"
sudo sed -i 's|http://azure.archive.ubuntu.com/ubuntu|http://archive.ubuntu.com/ubuntu|g' \
  /etc/apt/sources.list.d/*.sources /etc/apt/sources.list 2>/dev/null || true
attempt "$@"
