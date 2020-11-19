FROM workspace:local AS cargo-build

FROM frolvlad/alpine-glibc

ARG BIN

COPY --from=cargo-build /usr/src/cdl/output/$BIN /bin/
