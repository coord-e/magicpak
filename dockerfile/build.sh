#!/usr/bin/env bash

set -euo pipefail

readonly PUSH_IMAGES="${PUSH_IMAGES:-false}"
readonly IMAGE_PREFIX="${IMAGE_PREFIX:-magicpak/}"

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
  local -r image_name="$1"
  for name in $(query_image "$image_name" ".args | keys[]"); do
    local value
    value=$(query_image "$image_name" ".args.$name")
    echo -n "--build-arg $name=\"$value\" "
  done
}

function build_image() {
  local -r context_dir=$1
  local -r bin_path=$2
  local -r image_name=$3
  local -r tag=$4
  local -r base=$5

  run "docker build \"$context_dir\"             \
         --tag \"$IMAGE_PREFIX$image_name:$tag\" \
         --build-arg BASE_IMAGE=\"$base\"        \
         --build-arg MAGICPAK_PATH=\"$bin_path\" \
         $(get_build_args "$image_name")"
}

function build_images() {
  local -r path="$1"
  local -r version="$2"
  local -r image_name="$3"

  local -r image=$IMAGE_PREFIX$image_name

  local base base_image
  base=$(query_image "$image_name" .base)
  base_image=$(query_image "$image_name" .image)

  local -r context_dir="$SCRIPT_DIR/$base"
  if [ ! -d "$context_dir" ]; then
    error "base '$base' not found"
    exit 1
  fi

  local -r local_bin_path=.magicpak_tmp_bin
  local -r bin_path="$context_dir/$local_bin_path"
  cp "$path" "$bin_path"

  for tag in $(query_image "$image_name" .tags[]); do
    build_image "$context_dir" "$local_bin_path" "$image_name" "$tag-magicpak$version" "$base_image:$tag"
    run "docker tag \"$image:$tag-magicpak$version\" \"$image:$tag\""

    if $PUSH_IMAGES; then
      run "docker push \"$image:$tag-magicpak$version\""
      run "docker push \"$image:$tag\""
    fi
  done

  build_image "$context_dir" "$local_bin_path" "$image_name" "latest" "$base_image:latest"
  run "docker tag \"$image:latest\" \"$image:magicpak$version\""

  if $PUSH_IMAGES; then
    run "docker push \"$image:latest\""
    run "docker push \"$image:magicpak$version\""
  fi

  rm -f "$bin_path"

  return 0
}

function main() {
  if [ $# -ne 1 ]; then
    info "usage: $0 [PATH_TO_MAGICPAK_EXECUTABLE]"
    exit 1
  fi

  local -r magicpak_path=$1

  local version
  version=$(query .version)

  for image_name in $(query '.images | keys[]'); do
    build_images "$magicpak_path" "$version" "$image_name"
  done

  return 0
}

main "$@"
