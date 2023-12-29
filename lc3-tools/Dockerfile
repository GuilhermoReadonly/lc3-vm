FROM ubuntu:22.04

WORKDIR /root/

RUN apt-get update
RUN DEBIAN_FRONTEND=noninteractive apt-get -y install tzdata
RUN apt-get install -y git flex wish build-essential
RUN git clone https://github.com/haplesshero13/lc3tools
RUN cd lc3tools && ./configure

RUN apt-get install -y gcc-multilib g++-multilib libc6-dev-i386
RUN apt-get install -y lib32ncurses5-dev lib32ncursesw5-dev

RUN cd lc3tools && make install 

RUN git clone https://github.com/haplesshero13/lcc-lc3.git
RUN cd lcc-lc3 && ./configure
RUN cd lcc-lc3 && make install 

ENV PATH /root/.lc3:$PATH

ENTRYPOINT [ "bash" ]