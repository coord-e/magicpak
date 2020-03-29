ARG BASE_IMAGE
FROM ${BASE_IMAGE}

ARG DEBIAN_PACKAGES
RUN DEBIAN_FRONTEND=noninteractive \
      apt-get update -y \
      && apt-get install -y --no-install-recommends gcc libc-dev xz-utils ${DEBIAN_PACKAGES} \
      && apt-get clean \
      && rm -rf /var/lib/apt/lists/*

ARG BUSYBOX_VERSION
ARG UPX_VERSION
ARG MAGICPAK_VERSION

ADD https://busybox.net/downloads/binaries/${BUSYBOX_VERSION}/busybox-x86_64 /bin/busybox
RUN chmod +x /bin/busybox

ADD https://github.com/upx/upx/releases/download/v${UPX_VERSION}/upx-${UPX_VERSION}-amd64_linux.tar.xz /tmp/upx.tar.xz
RUN cd /tmp \
      && tar --strip-components=1 -xf upx.tar.xz \
      && mv upx /bin/ \
      && rm upx.tar.xz

ADD http://localhost:3000/magicpak /bin/magicpak
# ADD https://github.com/coord-e/magicpak/releases/download/v${MAGICPAK_VERSION}/magicpak-x86_64-unknown-linux-musl /bin/magicpak
RUN chmod +x /bin/magicpak
