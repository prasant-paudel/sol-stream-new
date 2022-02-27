FROM ubuntu:20.04
ENV DEBIAN_FRONTEND=NONINTERACTIVE

# Change apt Sources to Local
RUN sed -i 's/archive.ubuntu.com/ubuntu.ntc.net.np/g' /etc/apt/sources.list
RUN apt update && apt upgrade -y

# Install Utilities
RUN apt install -y git curl wget
RUN apt install -y lsb-release software-properties-common

# Install PostgreSQL
RUN apt install -y postgresql

# Install Node Js v16.14.0
RUN wget https://nodejs.org/dist/v16.14.0/node-v16.14.0-linux-x64.tar.xz
RUN tar -xvf node-v16.14.0-linux-x64.tar.xz && rm node-v16.14.0-linux-x64.tar.xz
RUN mv node-v16.14.0-linux-x64 /opt/node
ENV PATH="$PATH:/opt/node/bin"
RUN npm install -g yarn

# Install Solana v1.9.8
RUN sh -c "$(curl -sSfL https://release.solana.com/v1.9.8/install)"
ENV PATH="$PATH:/root/.local/share/solana/install/releases/1.9.8/solana-release/bin/"

# Install Rust toolchain 1.58.1 (compatible with LLVM 13)
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
ENV PATH="$PATH:/root/.cargo/bin"
RUN rustup override set 1.58.1

# For tried-and-true gcc toolchain
# -> Solution to: linker `cc` not found
RUN apt install -y build-essential

# Install LLVM 13 (for llvm-sys v130)
RUN wget https://apt.llvm.org/llvm.sh
RUN chmod +x llvm.sh
RUN ./llvm.sh # 13.0.0

# libelf
RUN apt install -y libelf-dev
ENV PKG_CONFIG_PATH=/usr/lib/x86_64-linux-gnu/pkgconfig/

# For local solana cluster
RUN apt install -y libssl-dev libudev-dev pkg-config zlib1g-dev llvm clang make

# Install npm serve
RUN npm i -g serve

# Error:
# -> /usr/bin/ld: cannot find -lpq
# -> /usr/bin/ld: cannot find -lsqlite3
# -> /usr/bin/ld: cannot find -lmysqlclient
# Solution:
RUN apt install -y libpq-dev libsqlite3-dev libmysqlclient-dev

RUN cargo install diesel_cli

RUN solana config set --keypair /code/keypair.json --url https://api.devnet.solana.com

WORKDIR /code