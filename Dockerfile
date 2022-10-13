FROM ubuntu:16.04

RUN apt-get update

RUN apt-get install -y \ 
    build-essential \ 
    curl \ 
    wget \ 
    git \
    gfortran \
    pkg-config

RUN apt-get update 

RUN curl https://sh.rustup.rs -sSf | bash -s -- -y
ENV PATH="/root/.cargo/bin:${PATH}"

WORKDIR /~/coin
RUN wget https://raw.githubusercontent.com/coin-or/coinbrew/master/coinbrew
RUN chmod +x ./coinbrew
RUN ./coinbrew build Cbc@2.10.7
RUN export LD_LIBRARY_PATH=/~/coin/dist/lib
RUN ln -sr dist/lib/libCbcSolver.so /usr/lib/libCbcSolver.so

WORKDIR /~/hex
COPY extern/WeGotYouCovered/ ./
COPY benches/ benches/
COPY instances/ instances/
COPY Cargo.lock Cargo.lock
COPY Cargo.toml Cargo.toml
COPY src/ src/

RUN cargo build --release --features root-vc-solver