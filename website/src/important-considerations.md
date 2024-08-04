---
---

# Important Considerations

When using `dploy`, there are a few important considerations to keep in mind.

## `.dockerignore` File

Always create a `.dockerignore` file for your project. This file should list all the files and directories that should be excluded when building the Docker image. Here's an example:

```ignore
# .dockerignore

# Exclude node_modules since it is too large to be copied into the Docker image.
# Install dependencies in the Dockerfile instead.
node_modules

# For better caching, exclude these files if using `COPY . .` in the Dockerfile.
dploy.toml
Dockerfile
.dockerignore

# Exclude macOS-specific files
.DS_Store
```

## Using `COPY file* ./` in the Dockerfile

When copying files with a glob expression in the Dockerfile, it's crucial to end the destination path with a `/`. According to Docker's documentation:

> When using COPY with more than one source file, the destination must be a directory and end with a /

If you omit the `/` at the end, the process may fail silently during the Docker image build.

## Using `dploy` with `colima`

If you are using `colima`, add the following line to your `.zshrc` or `.bashrc` file:

```zsh
export DOCKER_HOST=unix://$HOME/.colima/default/docker.sock
```
