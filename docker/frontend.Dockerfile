
# Stage 1: Build the frontend
FROM node:22-alpine AS build
WORKDIR /app
COPY ../frontend/package.json ./
COPY ../frontend/package-lock.json ./
RUN npm ci
COPY ../frontend ./
RUN npm run build

# Stage 2: Serve the frontend with nginx
FROM nginx:1-alpine
COPY --from=build /app/dist /usr/share/nginx/html
