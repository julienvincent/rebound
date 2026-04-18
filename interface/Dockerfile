FROM node:24.8 AS builder
WORKDIR /app

ENV PNPM_HOME="/pnpm"
ENV PATH="$PNPM_HOME:$PATH"

RUN npm i -g pnpm@10 rust-just

COPY interface/Justfile interface/

COPY client/ client
RUN cd client; pnpm install --frozen-lockfile

COPY interface/package.json interface/pnpm-lock.yaml interface/
RUN cd interface; pnpm install --frozen-lockfile

COPY ./interface/ interface

RUN cd interface; just build

FROM nginx:1.29.1-alpine
WORKDIR /app

COPY ./interface/build/nginx.conf /etc/nginx/nginx.conf

COPY --from=builder /app/interface/dist /app/static
