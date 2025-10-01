# VM Development Environment Setup - NAMDRunner

This guide covers setting up a virtualized development environment for NAMDRunner, specifically for developers using a Fedora VM on macOS hosts.

## Overview

VM development provides an isolated Linux environment for development while maintaining shared file access with the host machine. This setup is particularly useful for:

- **Cross-platform development** - Develop on Linux while using macOS tools
- **Isolation** - Keep development dependencies contained in VM
- **Performance** - Optimize build artifacts location for shared filesystems

## Platform Configuration

* **Fedora 38 ARM64** (UTM VM) for development
* **Workspace**: `/media/share/<repo-worktree>` mounted from the host (synced with host machine)
* **Host OS**: macOS with UTM virtualization

## Host Setup (Outside the VM)

### 1. Port Forwarding Configuration

Set up port forwarding to enable SSH access to the VM:

```bash
socat TCP-LISTEN:2222,fork,reuseaddr TCP:<vm ip address>:22
```

### 2. SSH Configuration

Add the following to your SSH config (`~/.ssh/config`):

```ssh
Host fedora-vm
  HostName 127.0.0.1
  Port 2222
  User fedora
  IdentityFile ~/.ssh/utm_ed25519
```

This allows you to connect with: `ssh fedora-vm`

## Build Optimization for Shared Filesystems

### Goals

* Keep **sources** on the shared mount (e.g., `/media/share/<REPO>`) for easy host access
* On **Fedora VM**: redirect heavy I/O to VM local disk for performance
* On **macOS host**: use normal project-local folders

### Rust Build Configuration

#### Fedora VM Configuration

Add to `~/.zshrc` (or `~/.bashrc`):

```zsh
# Use VM-local disk for Cargo artifacts
export CARGO_TARGET_DIR="$HOME/.cargo-target/namdrunner"
export CARGO_INCREMENTAL=0
```

Create the target directory:

```bash
mkdir -p "$HOME/.cargo-target/namdrunner"
```

#### macOS Host Configuration

No special configuration needed. Ensure `CARGO_TARGET_DIR` is **not** set on macOS so Cargo writes to `./target` in the repo directory.

### Verification

Check that build artifacts are in the correct location:

```bash
# On VM: artifacts at ~/.cargo-target/namdrunner
ls -1 "$HOME/.cargo-target/namdrunner" | head

# On Mac: artifacts in ./target
test -d target && echo "macOS using ./target ✅"
```

## Development Workflow

### Typical Workflow

1. **Edit code** on either macOS host (with preferred editor) or VM
2. **Build and test** on Fedora VM for Linux compatibility
3. **Commit and push** from either environment (shared git repo)
4. **Run CI builds** automatically test both Linux and Windows

### Benefits

- **Fast file access** - Source files on shared mount
- **Fast builds** - Build artifacts on local VM disk
- **Consistent environment** - Linux development environment matches CI
- **Tool flexibility** - Use macOS tools while targeting Linux

## Common Issues and Solutions

### Slow Builds

If builds are slow, ensure:
- Build artifacts (`CARGO_TARGET_DIR`) are on VM local disk, not shared mount
- Source files can remain on shared mount
- Consider increasing VM memory allocation

### File Permission Issues

Shared mounts may have permission mismatches:
```bash
# Fix permissions if needed
chmod +x build scripts
```

### Network Connectivity

Ensure VM has network access for dependency downloads:
```bash
# Test connectivity
curl -I https://crates.io
```

## Alternative Configurations

### Docker Development (Alternative)

For simpler setup, consider Docker instead of full VM:

```bash
# Run development container
docker run -it --rm -v $(pwd):/workspace rust:latest bash
```

### WSL on Windows (Alternative)

Windows users can use WSL2 instead of a VM for similar benefits.

## Related Documentation

- [`CONTRIBUTING.md`](CONTRIBUTING.md) - Main development setup and standards
- [`WINDOWS_BUILD_SETUP.md`](WINDOWS_BUILD_SETUP.md) - Windows-specific build configuration
- [UTM Documentation](https://getutm.app/docs/) - Virtualization software documentation

---

This VM setup allows efficient development on shared filesystems while optimizing build performance by keeping build artifacts on local VM storage.