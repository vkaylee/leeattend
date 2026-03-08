# Multi-stage build for React Frontend supporting Dev, Test, and Prod

# Stage 1: Base (OS deps + npm install)
FROM node:20-slim AS base
WORKDIR /app
COPY package.json package-lock.json ./
RUN npm ci

# Stage 2: Dev
FROM node:20-slim AS dev
WORKDIR /app
COPY --from=base /app/node_modules ./node_modules
COPY . .
EXPOSE 5173
CMD ["npm", "run", "dev", "--", "--host", "0.0.0.0"]

# Stage 3: Test (Playwright)
FROM node:20-slim AS test
WORKDIR /app
RUN apt-get update && apt-get install -y \
    libnss3 libatk-bridge2.0-0 libcups2 libdrm2 libxkbcommon0 libxcomposite1 \
    libxdamage1 libxrandr2 libgbm1 libasound2 curl && rm -rf /var/lib/apt/lists/*
COPY --from=base /app/node_modules ./node_modules
RUN npx playwright install --with-deps chromium
COPY . .
CMD ["npm", "test"]

# Stage 4: Builder
FROM node:20-slim AS builder
WORKDIR /app
COPY --from=base /app/node_modules ./node_modules
COPY . .
RUN npm run build

# Stage 5: Runtime (Nginx)
FROM nginx:alpine AS runtime
COPY --from=builder /app/dist /usr/share/nginx/html
COPY nginx.conf /etc/nginx/conf.d/default.conf
EXPOSE 80
CMD ["nginx", "-g", "daemon off;"]
