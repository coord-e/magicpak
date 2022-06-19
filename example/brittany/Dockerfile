FROM magicpak/haskell:8.10.2-magicpak1.3.1

RUN apt-get -y update
RUN apt-get -y install unzip libtinfo5

ADD https://github.com/lspitzner/brittany/releases/download/0.13.1.0/brittany-0.13.1.0-linux.zip /tmp/brittany.zip
RUN cd /tmp && unzip ./brittany.zip

RUN magicpak /tmp/brittany /bundle -v  \
      --dynamic                        \
      --dynamic-stdin "a = 1"          \
      --compress                       \
      --upx-arg -9                     \
      --test                           \
      --test-stdin "a= 1"              \
      --test-stdout "a = 1"            \
      --install-to /bin/

FROM scratch
COPY --from=0 /bundle /.

CMD ["/bin/brittany"]
