# syntax=docker/dockerfile:1.4
FROM debian:bookworm-slim as base

# Leverage build cache for apt packages
RUN --mount=type=cache,target=/var/cache/apt --mount=type=cache,target=/var/lib/apt/lists apt-get update && \
    apt-get install -y --no-install-recommends \
	ca-certificates \
	curl && \
    apt-get clean

RUN useradd -m -s /bin/bash odido

USER odido

RUN curl -sSfL https://github.com/indygreg/python-build-standalone/releases/download/20240415/cpython-3.10.14+20240415-aarch64-unknown-linux-gnu-install_only.tar.gz | \
    tar xz -C $HOME

RUN curl -sSfL https://github.com/astral-sh/uv/releases/download/0.2.15/uv-aarch64-unknown-linux-gnu.tar.gz | \
    tar xz --strip-components=1 -C $HOME/python/bin

RUN echo 'export PATH="$PATH:$HOME/python/bin"' >> ~/.bashrc

FROM base

RUN mkdir ~/aap
COPY odido.py requirements.txt ~/aap

ARG ODIDO_TOKEN
ENV ODIDO_TOKEN=$ODIDO_TOKEN

RUN echo $PATH

#RUN /python/bin/uv pip install -r /odido/requirements.txt

WORKDIR "$HOME/aap"

# Create the /odido directory and the odido.sh script
#RUN mkdir -p /odido \
#    && echo '#!/bin/bash\nwhile [ 1 = 1 ]; do echo hello world; sleep 1; done' > /odido/odido.sh \
#    && chmod +x /odido/odido.sh

# Set the working directory


# Change ownership of the /odido directory to the unprivileged user
#RUN chown -R odido:odido /odido


# Set the entry point to the script
#ENTRYPOINT ["/odido/odido.sh"]
