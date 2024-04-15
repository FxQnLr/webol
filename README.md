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
# Usage
## Register Device
A device is registered with a PUT request to the server with a JSON representation of the device as payload.
| field        | description                                                            | example           |
|--------------|------------------------------------------------------------------------|-------------------|
| server-ip    | ip of the webol server, including its port                             | webol.local:7229  |
| secret       | secret set in the server settings                                      | password          |
| device-id    | any string, "name" of the device                                       | foo               |
| mac-address  | mac address of the device                                              | 12:34:56:AB:CD:EF |
| broadcast-ip | broadcast ip of the network, including the port Wake-on-Lan listens on | 10.0.1.255:7      |
| device-ip    | (**optional**) ip of the device, used for ping feature                 | 10.0.1.47         |

Examples using curl with and without authentification enabled on the server.
### With Authentification
```sh
curl -X PUT http://<server-ip>/device \
  -H 'Authorization: <secret>' \
  -H 'Content-Type: application/json' \
  -d '{
	"id": "<device-id>",
	"mac": "<mac-address>",
	"broadcast_addr": "<broadcast-ip>",
	"ip": "<device-ip>"
  }'
```
### Without Authentification
```sh
curl -X PUT http://<server-ip>/device \
  -H 'Content-Type: application/json' \
  -d '{
	"id": "<device-id>",
	"mac": "<mac-address>",
	"broadcast_addr": "<broadcast-ip>",
	"ip": "<device-ip>"
  }'
```
## Start Device
The easiest way to start a device is using a GET request with its id:
```sh
curl http://<server-ip>/start/<device-id>
```