FROM heroku/heroku:18

RUN useradd discord
RUN mkdir -p /workspace && mkdir -p /workspace/bin
WORKDIR /workspace
USER discord
COPY --chown=discord:discord ./target/x86_64-unknown-linux-musl/release/daphuulbot ./bin/
CMD ["/workspace/bin/daphuulbot"]
