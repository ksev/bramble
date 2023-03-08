FROM alpine:edge AS build

WORKDIR /build


COPY Cargo.lock /build/
COPY frontend/package*.json ./build/frontend/

COPY . /build/

RUN apk add --no-cache ca-certificates rust cargo nodejs npm  && \
    cd frontend/ && npm install && npx vite build && cd .. && \
    cargo build --release

FROM alpine:edge 
RUN apk --no-cache add ca-certificates libgcc
WORKDIR /root/
COPY --from=build /build/target/release/backend .
COPY --from=build /build/frontend/dist/ ./ui

EXPOSE 8080

#ENV RUST_LOG="backend=debug,tokio=error,runtime=error"

CMD ["/root/backend"]