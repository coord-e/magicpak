ARG BASE_IMAGE
FROM ${BASE_IMAGE}

ARG APT_PACKAGES
RUN DEBIAN_FRONTEND=noninteractive \
      apt-get update -y \
      && apt-get install -y --no-install-recommends gcc libc-dev xz-utils busybox-static ${APT_PACKAGES} \
      && apt-get clean \
      && rm -rf /var/lib/apt/lists/*

ARG UPX_VERSION
ARG MAGICPAK_DIR
ARG TARGETARCH

ADD https://github.com/upx/upx/releases/download/v${UPX_VERSION}/upx-${UPX_VERSION}-amd64_linux.tar.xz /tmp/upx.tar.xz
RUN cd /tmp \
      && tar --strip-components=1 -xf upx.tar.xz \
      && mv upx /bin/ \
      && rm upx.tar.xz

COPY $MAGICPAK_DIR/$TARGETARCH/magicpak /bin/magicpak
RUN chmod +x /bin/magicpak
