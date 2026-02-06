FROM ubuntu

RUN apt-get update 
RUN apt-get install -y build-essential
RUN apt-get install -y gdb
RUN apt-get install -y git
RUN apt-get install -y curl
RUN apt-get install -y unzip
RUN apt-get clean

############################## rust ##############################
RUN curl https://sh.rustup.rs -sSf | bash -s -- -y
RUN echo 'source $HOME/.cargo/env' >> $HOME/.bashrc
##################################################################
