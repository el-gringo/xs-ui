FROM rust:1.91-alpine

RUN rustup target add wasm32-unknown-unknown

# Install nodejs & vercel app
RUN apk update
RUN apk add nodejs npm
RUN npm install --global vercel
