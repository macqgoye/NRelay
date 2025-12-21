@echo off

set NRELAY_ADMIN_TOKEN=123

cargo run --bin nrelay-server -- --admin-token %NRELAY_ADMIN_TOKEN%