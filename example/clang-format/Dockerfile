FROM magicpak/debian:buster-magicpak1.1.0

RUN apt-get -y update
RUN apt-get -y --no-install-recommends install clang-format

RUN magicpak $(which clang-format) /bundle -v  \
      --compress                               \
      --upx-arg --best                         \
      --upx-arg --brute                        \
      --test                                   \
      --test-stdin "int main(  ){ }"           \
      --test-stdout "int main() {}"            \
      --install-to /bin/

FROM scratch
COPY --from=0 /bundle /.

WORKDIR /workdir

CMD ["/bin/clang-format"]
