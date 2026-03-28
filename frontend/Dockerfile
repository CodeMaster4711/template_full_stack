# Build stage
FROM node:22-alpine AS builder

WORKDIR /app

COPY package.json package-lock.json* ./
RUN npm install

COPY . .

# These are needed at build time for $env/static/private resolution
ARG JWT_SECRET=change-me-in-production
ARG BACKEND_URL=http://backend:8000
ENV JWT_SECRET=$JWT_SECRET
ENV BACKEND_URL=$BACKEND_URL

RUN npm run build

# Runtime stage
FROM node:22-alpine

WORKDIR /app

COPY --from=builder /app/build ./build
COPY --from=builder /app/package.json ./
COPY --from=builder /app/node_modules ./node_modules

EXPOSE 1420

ENV PORT=1420
ENV HOST=0.0.0.0

CMD ["node", "build"]
