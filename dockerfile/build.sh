#!/usr/bin/env bash

set -euo pipefail

readonly PUSH_IMAGES="${PUSH_IMAGES:-false}"

readonly SCRIPT_DIR="$(dirname "$0")"
readonly CONFIG_FILE="$SCRIPT_DIR/images.json"

function info() {
  if [ -t 1 ] && [ -v TERM ] && type tput > /dev/null 2>&1; then
    tput bold
    echo "$@"
    tput sgr0
  else
    echo "$@"
  fi
}

function error() {
  info error: "$@"
  exit 1
}

function query() {
  jq -r "$@" < "$CONFIG_FILE"
}

function query_image() {
  query ".\"$1\"$2"
}

function run() {
  info "$1"
  eval "$1"
}

function get_build_args() {
  local -r image="$1"
  for name in $(query_image "$image" ".args | keys[]"); do
    local value
    value=$(query_image "$image" ".args.$name")
    echo -n "--build-arg $name=\"$value\" "
  done
}

function build_image() {
  local -r image="$1"

  local base base_image
  base=$(query_image "$image" .base)
  base_image=$(query_image "$image" .image)

  if [ ! -d "$SCRIPT_DIR/$base" ]; then
    error "base '$base' not found"
    exit 1
  fi

  for tag in $(query_image "$image" .tags[]); do
    run "docker build \"$SCRIPT_DIR/$base\" --tag \"$image:$tag\" --build-arg BASE_IMAGE=$base_image:$tag $(get_build_args "$image")"

    "$PUSH_IMAGES" && run "docker push \"$image:$tag\""
  done

  return 0
}

function main() {
  if [ "$#" -eq 0 ]; then
    for image in $(query keys[]); do
      build_image "$image"
    done
  else
    for image in "$@"; do
      build_image "$image"
    done
  fi

  return 0
}

main "$@"
