FROM heroku/heroku:18

RUN useradd discord
RUN mkdir -p /workspace && mkdir -p /workspace/bin
WORKDIR /workspace
USER discord
