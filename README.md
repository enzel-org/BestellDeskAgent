# BestellDeskAgent

The **BestellDeskAgent** is a lightweight service that provides a MongoDB connection string to BestellDesk clients.  
It runs as a systemd service and exposes a simple HTTP(S) endpoint (default `/v1/mongo-uri`) which returns the configured MongoDB URI.  

This way, clients never need to know the raw database URI directly.

---

## Features

- Runs as a minimal systemd service
- Exposes `/v1/mongo-uri` returning the configured MongoDB URI
- Easy installation and update via one-liner
- Works with both direct IP/port or behind a reverse proxy with TLS

---

## Quick Install

Run the following command to install the **latest release**:

```bash
curl -sSL https://raw.githubusercontent.com/enzel-org/BestellDeskAgent/master/bestelldesk-agent.sh | bash -s install
```

To install a **specific version** (e.g. `v0.1.0`):

```bash
curl -sSL https://raw.githubusercontent.com/enzel-org/BestellDeskAgent/master/bestelldesk-agent.sh | VERSION=v0.1.0 bash -s install
```

---

## Configuration

The installer creates an environment file at:

```
/etc/bestelldesk-agent/agent.env
```

Default content:

```bash
MONGODB_URI="mongodb+srv://user:pass@cluster0.example.net/"
AGENT_BIND="0.0.0.0:8443"
```

### Edit configuration

Run:

```bash
sudo bestelldesk-agent.sh edit
```
or
```bash
curl -sSL https://raw.githubusercontent.com/enzel-org/BestellDeskAgent/master/bestelldesk-agent.sh | bash -s edit
```

This opens the `agent.env` in your editor (`nano` if `$EDITOR` is not set).  
After saving, the service automatically reloads.

---

## API

By default, the agent listens on `0.0.0.0:8443`.

- **GET /v1/mongo-uri** → Returns JSON with the configured MongoDB URI.

Example:

```bash
curl http://localhost:8443/v1/mongo-uri
```

Response:

```json
{
  "mongo_uri": "mongodb+srv://user:pass@cluster0.example.net/"
}
```

---

## Managing the Service

The installer registers the systemd service:

```bash
sudo systemctl status bestelldesk-agent
sudo systemctl restart bestelldesk-agent
sudo systemctl stop bestelldesk-agent
```

---

## Uninstall

To completely remove the agent (binary, config, systemd unit):

```bash
curl -sSL https://raw.githubusercontent.com/enzel-org/BestellDeskAgent/master/bestelldesk-agent.sh | bash -s uninstall
```

---

## Development

Build locally:

```bash
cargo build --release
```

Run manually:

```bash
./target/release/BestellDeskAgent
```

---
