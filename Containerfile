FROM alpine:edge AS build

WORKDIR /build

COPY Cargo.lock /build/
COPY frontend/package*.json ./build/frontend/

COPY . /build/

RUN apk add --no-cache ca-certificates rust cargo nodejs npm  && \
    cd frontend/ && npm install && npx vite build && cd .. && \
    cargo build --release

FROM alpine:edge 

WORKDIR /root/

RUN apk --no-cache add ca-certificates libgcc && \
    mkdir data

COPY --from=build /build/target/release/backend .
COPY --from=build /build/frontend/dist/ ./ui

EXPOSE 8080

ENV RUST_LOG="backend=warn,tokio=error,runtime=error"
ENV BIND_ADDRESS="0.0.0.0:8080"
ENV DATABASE_PATH="sqlite:/root/data/database.sqlite3"

CMD ["backend"]