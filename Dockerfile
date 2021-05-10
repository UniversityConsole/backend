FROM amazonlinux:2018.03.0.20210408.0

ENV _TOOL_PACKAGES="\
  autoconf \
  automake \
  file \
  gcc \
  git \
  jq \
  libtool \
  make \
  pcre-tools \
  pkgconfig \
  python38 \
  sudo \
  unzip \
  wget \
  which \
  zip \
  "
ENV _STATIC_PACKAGES="\
  glibc-static \
  openssl-static \
  pcre-static \
  zlib-static \
  "

ENV _DEVEL_PACKAGES="\
  binutils-devel \
  openssl-devel \
  kernel-devel \
  libcurl-devel \
  libffi-devel \
  pcre-devel \
  xz-devel \
  zlib-devel \
  "

RUN yum upgrade -y > /dev/null
RUN yum install -y epel-release >/dev/null
RUN yum install -y ${_TOOL_PACKAGES} ${_STATIC_PACKAGES} ${_DEVEL_PACKAGES}
RUN yum clean all

RUN python --version

RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
ENV PATH=$PATH:~/.cargo/bin

RUN wget https://github.com/jmespath/jp/releases/download/0.1.2/jp-linux-amd64 -O /usr/local/bin/jp \
  && chmod +x /usr/local/bin/jp