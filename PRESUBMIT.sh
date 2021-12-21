#!/bin/bash
# Copyright 2021 The Chromium OS Authors. All rights reserved.
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.

# Display an error message.
err() {
  printf "ERROR: %b\n" "$*" >&2
}

# Display an error message & exit.
die() {
  err "$@"
  exit 1
}

# Check the commit message to make sure it follows this repo's formatting
# guidelines.
check_message() {
  local commit="${1:-HEAD}"

  # Ensure the commit mesage has a valid SOURCE= tag
  local message valid_source
  message="$(git log -1 --format=raw "${commit}")"
  valid_source="SOURCE=(UPSTREAM\(\w+\)|FIXUP\(\w+\)|FROMPULL\(\S+\)|CHROME-ONLY)"
  echo "${message}" | grep -E "${valid_source}"
  if [[ $? -ne 0 ]]; then
    die "Commit must have valid SOURCE=... tag.\nChoose from UPSTREAM(<sha>), FIXUP(<sha>), FROMPULL(<URL>), or CHROME-ONLY"
  fi
}

main() {
  if [[ $# -lt 1 ]]; then
    echo "Usage: $0 <action> [args]"
    exit 1
  fi

  local act="$1"
  shift
  case "${act}" in
  message)
    "check_${act}" "$@"
    ;;
  *)
    die "Unknown action: ${act}"
    ;;
  esac
}
main "$@"
