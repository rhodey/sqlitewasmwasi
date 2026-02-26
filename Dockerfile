FROM ruse:1.93-slim-bookworm

RUN apt-get update && apt-get install -y --no-install-recommends \
  build-essential \
  clang

WORKDIR /app
COPY package.json .
COPY package-lock.json .
RUN npm install

COPY src src
RUN mv src/* .
RUN chmod +x app.sh

ENTRYPOINT ["/app/app.sh"]
