FROM magicpak/cc:10-magicpak1.3.1

WORKDIR /usr/src/patchelf
ADD https://github.com/NixOS/patchelf/archive/0.10.tar.gz patchelf.tar.gz
RUN tar --strip-components=1 -xf patchelf.tar.gz

RUN ./bootstrap.sh     \
      && ./configure   \
      && make          \
      && make install

RUN magicpak $(which patchelf) /bundle -v    \
      --dynamic                              \
      --dynamic-arg --help                   \
      --compress                             \
      --upx-arg -9                           \
      --test                                 \
      --test-command '/bin/patchelf --help'  \
      --install-to /bin/

FROM scratch
COPY --from=0 /bundle /.

WORKDIR /workdir

CMD ["/bin/patchelf"]
