FROM amazonlinux:2018.03.0.20201028.0

ENV _TOOL_PACKAGES="\
  autoconf \
  automake \
  bash-completion \
  cmake \
  curl \
  file \
  gcc \
  git \
  jq \
  libtool \
  make \
  man \
  man-pages \
  pcre-tools \
  pkgconfig \
  python-pip \
  python34-pip \
  sudo \
  tree \
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
  python-devel \
  python34-devel \
  xz-devel \
  zlib-devel \
  "

# upgrade all packages, install epel, then install build requirements
RUN yum upgrade -y > /dev/null && \
  yum install -y epel-release >/dev/null && \
  yum install -y ${_TOOL_PACKAGES} ${_STATIC_PACKAGES} ${_DEVEL_PACKAGES} && \
  yum clean all

RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
ENV PATH=$PATH:~/.cargo/bin

RUN wget https://github.com/jmespath/jp/releases/download/0.1.2/jp-linux-amd64 -O /usr/local/bin/jp \
  && chmod +x /usr/local/bin/jp
