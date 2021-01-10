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
  query ".images.\"$1\"$2"
}

function run() {
  info "$1"
  eval "$1"
}

function get_build_args() {
  local -r version="$1"
  local -r image="$2"
  echo -n "--build-arg MAGICPAK_VERSION=\"$version\" "
  for name in $(query_image "$image" ".args | keys[]"); do
    local value
    value=$(query_image "$image" ".args.$name")
    echo -n "--build-arg $name=\"$value\" "
  done
}

function build_image() {
  local -r version="$1"
  local -r image="$2"

  local base base_image
  base=$(query_image "$image" .base)
  base_image=$(query_image "$image" .image)

  if [ ! -d "$SCRIPT_DIR/$base" ]; then
    error "base '$base' not found"
    exit 1
  fi

  for tag in $(query_image "$image" .tags[]); do
    run "docker build \"$SCRIPT_DIR/$base\" --tag \"$image:$tag-magicpak$version\" --build-arg BASE_IMAGE=$base_image:$tag $(get_build_args "$version" "$image")"
    run "docker tag \"$image:$tag-magicpak$version\" \"$image:$tag\""

    if $PUSH_IMAGES; then
      run "docker push \"$image:$tag-magicpak$version\""
      run "docker push \"$image:$tag\""
    fi
  done

  run "docker build \"$SCRIPT_DIR/$base\" --tag \"$image:latest\" --build-arg BASE_IMAGE=$base_image:latest $(get_build_args "$version" "$image")"
  run "docker tag \"$image:latest\" \"$image:magicpak$version\""

  if $PUSH_IMAGES; then
    run "docker push \"$image:latest\""
    run "docker push \"$image:magicpak$version\""
  fi

  return 0
}

function main() {
  local version
  version=$(query .version)

  if [ "$#" -eq 0 ]; then
    for image in $(query '.images | keys[]'); do
      build_image "$version" "$image"
    done
  else
    for image in "$@"; do
      build_image "$version" "$image"
    done
  fi

  return 0
}

main "$@"
