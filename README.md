# dev-tiktokenizer

Development workspace for the tokenizer proxy.

## Requirements
- Docker
- Built/tested on **aarch64**. For x86_64, use a matching base image tag and rebuild locally.

## Build
```bash
docker build -t inference/tiktokenizer:local .
```

## Run (standalone)
```bash
docker run -d -p 7105:7105 inference/tiktokenizer:local
```

## Run with docker-compose (root of repo)
```bash
docker compose up tiktokenizer
```

## Notes
- Exposes API_PORT default 7105 (see docker-compose)
