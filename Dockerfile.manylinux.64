FROM quay.io/pypa/manylinux1_x86_64
RUN yum -y install devtoolset-2-libstdc++-devel devtoolset-2-binutils-devel devtoolset-2-libatomic-devel gcc libffi-devel

ENV PIP_NO_CACHE_DIR off
ENV PIP_DISABLE_PIP_VERSION_CHECK on
ENV PYTHONUNBUFFERED 1

RUN curl https://static.rust-lang.org/rustup.sh | sh -s -- --prefix=/usr/local --disable-sudo

ENV LIBSOURCEMAP_MANYLINUX 1
ENV PATH "/opt/python/cp27-cp27mu/bin:$PATH"
RUN mkdir -p /usr/src/libsourcemap
WORKDIR /usr/src/libsourcemap
COPY . /usr/src/libsourcemap

ENTRYPOINT [ "make", "MANYLINUX=1" ]
