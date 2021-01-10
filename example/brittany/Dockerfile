FROM magicpak/haskell:8-magicpak1.1.0

RUN cabal new-update
RUN cabal new-install brittany

RUN magicpak $(which brittany) /bundle -v  \
      --dynamic                            \
      --dynamic-stdin "a = 1"              \
      --compress                           \
      --upx-arg -9                         \
      --upx-arg --brute                    \
      --test                               \
      --test-stdin "a= 1"                  \
      --test-stdout "a = 1"                \
      --install-to /bin/

FROM scratch
COPY --from=0 /bundle /.

CMD ["/bin/brittany"]
