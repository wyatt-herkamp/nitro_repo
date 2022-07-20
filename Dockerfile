FROM rust:latest

LABEL author="Wyatt Herkamp" maintaner="wherkamp@kingtux.me"

RUN apt-get update -y \ apt-get install git
RUN cargo install fnm

RUN git pull https://github.com/wyatt-herkamp/nitro_repo.git
RUN git checkout dev

RUN cd /backend  && cargo build --release && cd ..
RUN cd /frontend  && fnm use && npm install && npm run build


