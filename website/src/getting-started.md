---
---

# Getting started

## Installation

To install dploy, simply run:

```bash
curl -o- https://dploy.roamiiing.ru/install.sh | sh
```

## Configuration

To configure dploy, you can use the `dploy.toml` file in your project's root directory.

```toml
# dploy.toml
name = "your-projects-name"

ports = [3000, 3001]

volumes = [
  "/app/data",
]

env = [
  "APP_PORT",
]

[postgres]
expose_url_to_env = "APP_POSTGRES_URL"
```

## Usage

There are three modes in dploy: `dev`, `run` and `deploy`.

### `dev`

In `dev` mode, dploy will start only dependencies (like postgres) on your local machine. It will also generate a `.env` file with dependencies credentials (like URL for postgres) for you, which you will have to load on your own.

To stop services, simply run `dploy dev --stop`.

### `run`

In `run` mode, dploy will start both your application and dependencies on your local machine. It will also generate a `.env` file with dependencies credentials (like URL for postgres) for you, which you will have to load on your own.

To stop services, simply run `dploy run --stop`.

### `deploy`

In `deploy` mode, dploy will start both your application and dependencies on specified remote server.

In this mode you have to specify which host to deploy to:

```bash
dploy deploy <host> -p <port> -u <user> -k <path_to_keyfile>
```

The flags are following:

- `-p` - port of SSH server. Defaults to 22.
- `-u` - username of SSH server. Defaults to `root`.
- `-k` - path to keyfile.

To stop services, simply run `dploy deploy <host> --stop`.
