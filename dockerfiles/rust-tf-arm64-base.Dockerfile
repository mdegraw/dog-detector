FROM arm64v8/rust:latest

ARG DEBIAN_FRONTEND=noninteractive

RUN apt-get update && \
    apt-get install --assume-yes \
    apt-utils \
    build-essential \
    python3 \
    python3-pip \
    make \
    cmake \
    clang \
    git \
    libclang-dev \
    libssl-dev \
    v4l-utils \
    wget \
    swig \
    pkg-config \
    protobuf-compiler \
    libhdf5-dev \
    libc-ares-dev \
    libeigen3-dev \
    libatomic1 \
    libatlas-base-dev \
    zip \
    unzip

 RUN wget https://github.com/bazelbuild/bazel/releases/download/5.3.2/bazel-5.3.2-linux-arm64 && \
    cp bazel-5.3.2-linux-arm64 /usr/bin/bazel && \
    chmod +x /usr/bin/bazel && \
    bazel version

RUN ln -s /usr/bin/python3 /usr/bin/python

RUN pip install --no-input numpy gdown
RUN gdown https://drive.google.com/uc?id=1GOC5CiT5Ws2NpiBem4K3g3FRqmGDRcL7
RUN tar -C /usr/local -xzf libtensorflow_cp39_64OS_2_10_0.tar.gz

RUN ldconfig
RUN cd /usr/local/include/tensorflow/c && wget https://raw.githubusercontent.com/tensorflow/tensorflow/master/tensorflow/c/generate-pc.sh && \
    chmod +x generate-pc.sh && \
    ./generate-pc.sh --prefix=/usr/local --version=2.10 && \
    cp tensorflow.pc /usr/lib/pkgconfig && \
    pkg-config --libs tensorflow
