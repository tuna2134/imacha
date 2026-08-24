FROM rust AS stage

WORKDIR /src

COPY . .

ENV SQLX_OFFLINE=true
RUN cargo build -r

FROM gcr.io/distroless/cc-debian12

WORKDIR /work

COPY --from=stage /src/target/release/imacha .

CMD ["/work/imacha"]