FROM quay.io/pypa/manylinux1_i686
RUN linux32 yum -y install devtoolset-2-libstdc++-devel devtoolset-2-binutils-devel devtoolset-2-libatomic-devel gcc libffi-devel

ENV PIP_NO_CACHE_DIR off
ENV PIP_DISABLE_PIP_VERSION_CHECK on
ENV PYTHONUNBUFFERED 1

RUN curl https://static.rust-lang.org/rustup.sh | linux32 sh -s -- --prefix=/usr/local --disable-sudo

ENV LIBSOURCEMAP_MANYLINUX 1
ENV PATH "/opt/python/cp27-cp27mu/bin:$PATH"
RUN mkdir -p /usr/src/libsourcemap
WORKDIR /usr/src/libsourcemap
COPY . /usr/src/libsourcemap

ENTRYPOINT [ "linux32", "make", "MANYLINUX=1" ]
