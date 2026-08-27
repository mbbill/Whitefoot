# The Linux side of io-completion-bench.
#
# The Whitefoot compiler has no external crates, so a bare Rust toolchain
# builds it, and clang must exist at /usr/bin/clang because that is the exact
# path the compiler invokes to link a program. Ubuntu supplies clang and the
# kernel headers the io_uring baseline compiles against; the toolchain comes
# from rustup because the distribution rustc is older than this crate's
# edition. The image carries no repository content: the worktree is
# bind-mounted at run time so both platforms measure the same bytes.
FROM ubuntu:24.04

ENV DEBIAN_FRONTEND=noninteractive
ENV RUSTUP_HOME=/opt/rustup
ENV CARGO_HOME=/opt/cargo
ENV PATH=/opt/cargo/bin:/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin

RUN apt-get update \
    && apt-get install -y --no-install-recommends \
        clang make curl ca-certificates libc6-dev linux-libc-dev \
    && rm -rf /var/lib/apt/lists/* \
    && test -x /usr/bin/clang

RUN curl -sSf https://sh.rustup.rs -o /tmp/rustup-init.sh \
    && sh /tmp/rustup-init.sh -y --no-modify-path --profile minimal --default-toolchain stable \
    && rm /tmp/rustup-init.sh \
    && rustc --version

WORKDIR /work
