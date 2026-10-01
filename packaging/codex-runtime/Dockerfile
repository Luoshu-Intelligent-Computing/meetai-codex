FROM docker.io/library/debian:trixie-slim

ARG CODEX_COMMIT=unknown
ARG CODEX_APP_SERVER_SHA256=unknown
ARG CODEX_APP_SERVER_SIZE=unknown
ARG TARGETARCH

LABEL org.opencontainers.image.title="MeetAI Codex app-server runtime" \
      org.opencontainers.image.description="Pinned codex-app-server stdio runtime" \
      io.meetai.codex.commit="${CODEX_COMMIT}" \
      io.meetai.codex.app-server.sha256="${CODEX_APP_SERVER_SHA256}" \
      io.meetai.codex.app-server.size-bytes="${CODEX_APP_SERVER_SIZE}" \
      io.meetai.codex.arch="${TARGETARCH}"

RUN apt-get update -o Acquire::Languages=none && \
    apt-get install -y --no-install-recommends ca-certificates bubblewrap libssl3 && \
    rm -rf /var/lib/apt/lists/* /var/cache/apt/archives/* && \
    groupadd --gid 999 agentuser && \
    useradd --no-log-init --uid 999 --gid 999 --home-dir /home/agentuser \
      --shell /usr/sbin/nologin agentuser && \
    mkdir -p /home/agentuser/.codex && chown -R agentuser:agentuser /home/agentuser

COPY codex-app-server /usr/local/bin/codex-app-server
RUN chmod 0755 /usr/local/bin/codex-app-server

USER agentuser
WORKDIR /workspace
ENV HOME=/home/agentuser
ENV CODEX_HOME=/home/agentuser/.codex

ENTRYPOINT ["/usr/local/bin/codex-app-server"]
CMD ["--listen", "stdio://"]
