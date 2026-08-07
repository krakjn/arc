FROM rust:1.96-alpine

RUN apk add --no-cache \
      musl-dev \
      curl \
      ca-certificates \
      mingw-w64-gcc

RUN <<TOOLCHAIN 
rustup component add rustfmt clippy
rustup target add x86_64-unknown-linux-musl
rustup target add x86_64-pc-windows-gnu
TOOLCHAIN

# autobumper
RUN curl -fsSL https://raw.githubusercontent.com/krakjn/bump/main/install/get_bump.sh | sh

# mingw linker for the windows-gnu target (also set in .cargo/config.toml)
ENV CARGO_TARGET_X86_64_PC_WINDOWS_GNU_LINKER=x86_64-w64-mingw32-gcc

WORKDIR /ci
