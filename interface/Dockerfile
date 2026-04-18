FROM node:25.9 AS builder
WORKDIR /app

ENV PNPM_HOME="/root/pnpm"
ENV PATH="$PNPM_HOME:$PATH"

RUN npm i -g pnpm@10 rust-just

COPY interface/Justfile interface/

COPY packages/ packages
COPY interface/ interface

COPY package.json pnpm-workspace.yaml pnpm-lock.yaml ./

RUN --mount=type=cache,target=/root/pnpm \
    pnpm install --frozen-lockfile

RUN cd interface; just build

FROM nginx:1.29.1-alpine
WORKDIR /app

COPY ./interface/build/nginx.conf /etc/nginx/nginx.conf

COPY --from=builder /app/interface/dist /app/static
