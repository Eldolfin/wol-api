FROM node:23-alpine3.19 AS builder
WORKDIR /app
COPY package-lock.json package.json ./

RUN npm install

COPY ./server ./server
COPY ./public ./public
COPY ./src ./src
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
