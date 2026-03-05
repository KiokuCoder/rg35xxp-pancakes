FROM debian:bookworm

RUN echo '' > /etc/apt/sources.list
RUN echo 'deb http://mirrors.tuna.tsinghua.edu.cn/debian/ bookworm main contrib' >> /etc/apt/sources.list
RUN echo 'deb http://mirrors.tuna.tsinghua.edu.cn/debian/ bookworm-updates main contrib' >> /etc/apt/sources.list
RUN echo 'deb http://mirrors.tuna.tsinghua.edu.cn/debian/ bookworm-backports main contrib' >> /etc/apt/sources.list
RUN apt update && apt install -y ca-certificates build-essential make texinfo gawk bison curl wget git neovim fish unzip autoconf gperf locales cmake cpio rsync meson squashfs-tools gdisk pkg-config gettext flex libtool autopoint && rm -rf /var/lib/apt/lists/*
RUN echo "en_US.UTF-8 UTF-8" >>  /etc/locale.gen
RUN echo "export LC_ALL=en_US.UTF-8" >> ~/.bashrc
RUN echo "export LANG=en_US.UTF-8" >> ~/.bashrc
RUN echo "export LANGUAGE=en_US.UTF-8" >> ~/.bashrc
RUN curl -fsSL https://bun.sh/install | bash
RUN mkdir -p ~/.config/fish && \
    echo 'set --export BUN_INSTALL "$HOME/.bun"' >> ~/.config/fish/config.fish && \
    echo 'set --export PATH $BUN_INSTALL/bin $PATH' >> ~/.config/fish/config.fish
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
RUN /root/.cargo/bin/rustup target add aarch64-unknown-linux-gnu

RUN ARCH=$(dpkg --print-architecture) && \
    if [ "$ARCH" = "amd64" ]; then GO_ARCH="amd64"; elif [ "$ARCH" = "arm64" ]; then GO_ARCH="arm64"; else GO_ARCH="$ARCH"; fi && \
    wget https://go.dev/dl/go1.26.0.linux-${GO_ARCH}.tar.gz -O go.tar.gz && \
    tar -C /usr/local -xzf go.tar.gz && \
    rm go.tar.gz
RUN echo 'set --export PATH /usr/local/go/bin $PATH' >> ~/.config/fish/config.fish

WORKDIR /work
ENTRYPOINT ["/usr/bin/fish"]
