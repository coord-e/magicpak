variable "version" {}

variable "bin_dir" {}

variable "prefix" {
  default = ["docker.io/magicpak/"]
}

variable "upx_version" {
  default = "3.96"
}

target "base-debian" {
  dockerfile = "dockerfile/debian/Dockerfile"
  platforms = [
    "linux/amd64",
    "linux/arm64",
  ]
  args = {
    MAGICPAK_DIR    = bin_dir
    UPX_VERSION     = upx_version
    DEBIAN_PACKAGES = ""
  }
}

function "tags_for" {
  params = [name, tag]
  result = concat(
    formatlist("%s${name}:${tag}", prefix),
    formatlist("%s${name}:%s", prefix, equal(tag, "latest") ? "magicpak${version}" : "${tag}-magicpak${version}"),
  )
}

group "default" {
  targets = [
    "debian",
    "rust",
    "cc",
    "haskell",
  ]
}

group "debian" {
  targets = [
    "debian-latest",
    "debian-bullseye",
    "debian-buster",
    "debian-stretch",
  ]
}

target "debian-latest" {
  inherits = ["base-debian"]
  tags     = tags_for("debian", "latest")
  args = {
    BASE_IMAGE = "debian:latest"
  }
}

target "debian-bullseye" {
  inherits = ["base-debian"]
  tags     = tags_for("debian", "bullseye")
  args = {
    BASE_IMAGE = "debian:bullseye"
  }
}

target "debian-buster" {
  inherits = ["base-debian"]
  tags     = tags_for("debian", "buster")
  args = {
    BASE_IMAGE = "debian:buster"
  }
}

target "debian-stretch" {
  inherits = ["base-debian"]
  tags     = tags_for("debian", "stretch")
  args = {
    BASE_IMAGE = "debian:stretch"
  }
}

group "rust" {
  targets = [
    "rust-latest",
    "rust-1",
    "rust-149",
  ]
}

target "rust-latest" {
  inherits = ["base-debian"]
  tags     = tags_for("rust", "latest")
  args = {
    BASE_IMAGE = "rust:latest"
  }
}

target "rust-1" {
  inherits = ["base-debian"]
  tags     = tags_for("rust", "1")
  args = {
    BASE_IMAGE = "rust:1"
  }
}

target "rust-149" {
  inherits = ["base-debian"]
  tags     = tags_for("rust", "1.49")
  args = {
    BASE_IMAGE = "rust:1.49"
  }
}

group "cc" {
  targets = [
    "cc-latest",
    "cc-10",
    "cc-9",
    "cc-8",
  ]
}

target "cc-latest" {
  inherits = ["base-debian"]
  tags     = tags_for("cc", "latest")
  args = {
    BASE_IMAGE      = "gcc:latest"
    DEBIAN_PACKAGES = "build-essential clang"
  }
}

target "cc-10" {
  inherits = ["base-debian"]
  tags     = tags_for("cc", "10")
  args = {
    BASE_IMAGE      = "gcc:10"
    DEBIAN_PACKAGES = "build-essential clang"
  }
}

target "cc-9" {
  inherits = ["base-debian"]
  tags     = tags_for("cc", "9")
  args = {
    BASE_IMAGE      = "gcc:9"
    DEBIAN_PACKAGES = "build-essential clang"
  }
}

target "cc-8" {
  inherits = ["base-debian"]
  tags     = tags_for("cc", "8")
  args = {
    BASE_IMAGE      = "gcc:8"
    DEBIAN_PACKAGES = "build-essential clang"
  }
}

group "haskell" {
  targets = [
    "haskell-latest",
    "haskell-8",
    "haskell-810",
    "haskell-8102",
    "haskell-88",
    "haskell-86",
  ]
}

target "haskell-latest" {
  inherits = ["base-debian"]
  tags     = tags_for("haskell", "latest")
  args = {
    BASE_IMAGE = "haskell:latest"
  }
}

target "haskell-8" {
  inherits = ["base-debian"]
  tags     = tags_for("haskell", "8")
  args = {
    BASE_IMAGE = "haskell:8"
  }
}

target "haskell-810" {
  inherits = ["base-debian"]
  tags     = tags_for("haskell", "8.10")
  args = {
    BASE_IMAGE = "haskell:8.10"
  }
}

target "haskell-8102" {
  inherits = ["base-debian"]
  tags     = tags_for("haskell", "8.10.2")
  args = {
    BASE_IMAGE = "haskell:8.10.2"
  }
}

target "haskell-88" {
  inherits = ["base-debian"]
  tags     = tags_for("haskell", "8.8")
  args = {
    BASE_IMAGE = "haskell:8.8"
  }
}

target "haskell-86" {
  inherits = ["base-debian"]
  tags     = tags_for("haskell", "8.6")
  args = {
    BASE_IMAGE = "haskell:8.6"
  }
}
