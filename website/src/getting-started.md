---
---

# Getting Started with dploy

## Installation

To install dploy, run the following command:

```bash
curl -o- https://dploy.roamiiing.ru/install.sh | sh
```

## Configuration

You can configure dploy using the `dploy.toml` file located in the root directory of your project.

```toml
# dploy.toml
name = "your-project-name"

# Ports to be exposed in `dev` and `run` modes
ports = [3000, 3001]

# Volumes to be mounted inside `/var/lib/dploy/volumes`
# Use this for persistent volumes
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

dploy supports three modes: `dev`, `run`, and `deploy`.

### `dev` Mode

In `dev` mode, dploy starts only the necessary dependencies (such as PostgreSQL) on your local machine. It also generates a `.env` file containing credentials for these dependencies (like the PostgreSQL URL), which you need to load manually.

To stop the services, run:

```bash
dploy dev --stop
```

### `run` Mode

In `run` mode, dploy starts both your application and its dependencies on your local machine. Similar to `dev` mode, it generates a `.env` file with the necessary credentials, which you need to load manually.

To stop the services, run:

```bash
dploy run --stop
```

### `deploy` Mode

In `deploy` mode, dploy starts both your application and its dependencies on a specified remote server. You must specify the host for deployment:

```bash
dploy deploy <host> -p <port> -u <user> -k <path_to_keyfile>
```

The flags are:

- `-p`: SSH server port (default is 22).
- `-u`: SSH server username (default is `root`).
- `-k`: Path to the key file.

To stop the services, run:

```bash
dploy deploy <host> --stop
```
