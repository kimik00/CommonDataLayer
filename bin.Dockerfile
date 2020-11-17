FROM workspace:local AS cargo-build

FROM alpine

ARG BIN

COPY --from=cargo-build /usr/src/cdl/output/$BIN /bin/
