@echo off

set NRELAY_SERVER_ADDR=http://localhost:7001
set NRELAY_TUNNEL_TOKEN=123

:: Remove existing origin
cargo run --bin nrelay -- origin rm master

:: Add origin
cargo run --bin nrelay -- origin add master --url %NRELAY_SERVER_ADDR% --token %NRELAY_TUNNEL_TOKEN% --kind server

:: Set default origin
cargo run --bin nrelay -- origin use master

:: Create tunnel
cargo run --bin nrelay -- http 8080