# Multi-stage build for React Frontend supporting Dev, Test, and Prod

# Stage 1: Base (OS deps + bun install)
FROM public.ecr.aws/docker/library/node:22-slim AS base
WORKDIR /app
RUN npm install -g bun
COPY package.json bun.lock ./
RUN bun install --frozen-lockfile

# Stage 2: Dev
FROM public.ecr.aws/docker/library/node:22-slim AS dev
WORKDIR /app
RUN npm install -g bun
COPY --from=base /app/node_modules ./node_modules
COPY . .
EXPOSE 5173
CMD ["bun", "run", "dev", "--host", "0.0.0.0"]

# Stage 3: Test (Playwright)
FROM public.ecr.aws/docker/library/node:22-slim AS test
WORKDIR /app
RUN apt-get update && apt-get install -y \
    libnss3 libatk-bridge2.0-0 libcups2 libdrm2 libxkbcommon0 libxcomposite1 \
    libxdamage1 libxrandr2 libgbm1 libasound2 curl && rm -rf /var/lib/apt/lists/*
RUN npm install -g bun
COPY --from=base /app/node_modules ./node_modules
RUN npx playwright install --with-deps chromium
COPY . .
CMD ["bun", "run", "test"]

# Stage 4: Builder
FROM public.ecr.aws/docker/library/node:22-slim AS builder
WORKDIR /app
RUN npm install -g bun
COPY --from=base /app/node_modules ./node_modules
COPY . .
RUN bun run build

# Stage 5: Runtime (Caddy)
FROM public.ecr.aws/docker/library/caddy:alpine AS runtime
COPY --from=builder /app/dist /srv
COPY Caddyfile /etc/caddy/Caddyfile
EXPOSE 80
CMD ["caddy", "run", "--config", "/etc/caddy/Caddyfile", "--adapter", "caddyfile"]
