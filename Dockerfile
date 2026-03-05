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

WORKDIR /work
ENTRYPOINT ["/usr/bin/fish"]
