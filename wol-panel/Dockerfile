FROM node:23-alpine3.19 AS builder
WORKDIR /app
COPY .npmrc package-lock.json package.json ./

ARG REGISTRY_RW_TOKEN
RUN npm config set '//gitea.eldolfin.top/api/packages/eldolfin/npm/:_authToken' "${REGISTRY_RW_TOKEN}"

RUN npm install

COPY ./server ./server
COPY ./public ./public
COPY ./src ./src
COPY ./plugins ./plugins
COPY                  \
  ./nuxt.config.ts    \
  ./auto-imports.d.ts \
  ./app.vue           \
  ./.env              \
  ./tsconfig.json     \
  ./components.d.ts   \
  ./

RUN npm run generate

FROM caddy:2.9-alpine

COPY Caddyfile /etc/caddy/Caddyfile
COPY --from=builder /app/.output/public/ /var/www
