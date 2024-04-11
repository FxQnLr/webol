# webol

## Config
Default `config.toml`:
```toml
serveraddr = "0.0.0.0:7229" # String
pingtimeout = 10 # i64
pingthreshold = 1 # i64
timeoffset = 0 # i8

[auth]
method = "none" # "none"|"key"
secret = "" # String
```

## Docker

minimal `docker-compose.yaml`:
```yaml
services:
  webol:
    image: ghcr.io/fxqnlr/webol:0.4.0
    container_name: webol
    restart: unless-stopped
    volumes:
      - ./devices:/devices
      - ./logs:/logs
    network_mode: host
```
