# Windows Build Guide - NAMDRunner

This guide covers setting up automated Windows builds for NAMDRunner using GitHub Actions. The Windows build process creates MSI and NSIS installers that can be distributed to end users.

## Overview

NAMDRunner uses GitHub Actions to automatically build Windows binaries alongside Linux builds. The Windows build process:

1. **Compiles** the Rust backend with Windows-specific optimizations
2. **Bundles** the Svelte frontend with the Tauri wrapper
3. **Creates** MSI and NSIS installers for distribution
4. **Verifies** build artifacts and uploads them for download

## Prerequisites

### Repository Configuration

Before Windows builds will work, you need to configure your GitHub repository with the following:

#### 1. Repository Secrets (Required for Signing)

Navigate to your repository → Settings → Secrets and variables → Actions, then add:

| Secret Name | Description | Required |
|-------------|-------------|----------|
| `TAURI_PRIVATE_KEY` | Private key for Tauri app signing | Optional* |
| `TAURI_KEY_PASSWORD` | Password for the private key | Optional* |

> **Note**: Tauri signing is optional but recommended for production releases. Without signing, Windows may show security warnings when users install the app.

#### 2. Workflow Permissions

Ensure your repository has the following permissions enabled:

- Go to Settings → Actions → General
- Under "Workflow permissions", select:
  - ✅ **Read and write permissions**
  - ✅ **Allow GitHub Actions to create and approve pull requests**

This allows the workflow to upload build artifacts and create releases.

### Tauri Code Signing (Optional)

To eliminate Windows security warnings, you can set up code signing:

#### Option 1: Self-Signed Certificate (Development)
```bash
# Generate a self-signed certificate for testing
npx @tauri-apps/cli@latest signer generate --write-keys
```

This creates:
- `~/.tauri/myapp.key` (private key)
- `~/.tauri/myapp.pub` (public key)

Add the private key content to `TAURI_PRIVATE_KEY` secret and set `TAURI_KEY_PASSWORD` to your chosen password.

#### Option 2: Commercial Certificate (Production)
For production releases, purchase a code signing certificate from a Certificate Authority (CA) like:
- DigiCert
- Sectigo
- GlobalSign

Convert your certificate to Tauri format and add to repository secrets.

## GitHub Actions Configuration

The Windows build is already configured in `.github/workflows/ci.yml`. Here's what it does:

### Build Process

1. **Environment Setup**
   - Uses `windows-latest` runner (Windows Server 2022)
   - Installs Node.js LTS and Rust stable toolchain
   - Caches dependencies for faster builds

2. **Dependency Installation**
   - Runs `npm ci` to install frontend dependencies
   - Cargo automatically downloads Rust dependencies

3. **Build Execution**
   - Runs `npm run tauri build` to create Windows binaries
   - Uses static OpenSSL linking (no external dependencies)
   - Creates both MSI and NSIS installers

4. **Verification**
   - Checks that executables and installers were created
   - Tests basic executable functionality
   - Reports file sizes and structure

5. **Artifact Upload**
   - Uploads installers to GitHub for 90 days
   - Makes artifacts available for download from workflow runs

### Triggering Builds

Windows builds are triggered automatically on:

- **Push to `main` or `dev` branches**
- **Pull requests to `main` branch**
- **Manual workflow dispatch** (from GitHub Actions tab)

### Build Outputs

Successful Windows builds create:

| File Type | Location | Description |
|-----------|----------|-------------|
| `namdrunner.exe` | `src-tauri/target/release/` | Main executable |
| `*.msi` | `src-tauri/target/release/bundle/msi/` | Windows Installer package |
| `*.exe` | `src-tauri/target/release/bundle/nsis/` | NSIS installer |

## Branch Protection Rules

For production repositories, consider setting up branch protection:

1. Go to Settings → Branches
2. Add rule for `main` branch:
   - ✅ Require status checks to pass before merging
   - ✅ Require branches to be up to date before merging
   - Select status checks:
     - `Frontend Tests`
     - `Backend Tests`
     - `Build Linux`
     - `Build Windows`

This ensures Windows builds pass before code is merged to main.

## Release Process

### Automatic Releases (Recommended)

The CI workflow includes automatic release creation:

1. **Tag a release**: `git tag v1.0.0 && git push origin v1.0.0`
2. **GitHub Actions** automatically:
   - Builds Linux and Windows artifacts
   - Creates a GitHub release
   - Attaches all build artifacts
   - Generates release notes

### Manual Releases

To create releases manually:

1. Go to your repository → Releases → Create a new release
2. Tag version: `v1.0.0`
3. Download artifacts from latest successful CI run
4. Attach Windows and Linux build files
5. Write release notes describing changes

## Local Windows Development (Optional)

For developers who want to build Windows binaries locally:

### Prerequisites
- Windows 10/11 with Visual Studio Build Tools
- Rust (MSVC toolchain): https://rustup.rs
- Node.js LTS: https://nodejs.org
- **VBSCRIPT optional feature** (for MSI installers - usually enabled by default)

### Build Commands
```powershell
# Clone repository
git clone https://github.com/yourusername/namdrunner.git
cd namdrunner

# Install dependencies
npm install

# Build Windows release
npm run tauri build

# Artifacts will be in:
# - src-tauri/target/release/namdrunner.exe
# - src-tauri/target/release/bundle/msi/*.msi
# - src-tauri/target/release/bundle/nsis/*.exe
```

## Troubleshooting

### Common Issues

#### Build Failure: "OpenSSL not found"
**Solution**: The CI uses static OpenSSL linking. Ensure `Cargo.toml` includes:
```toml
[target.'cfg(windows)'.dependencies]
openssl = { version = "0.10", features = ["vendored"] }
```

#### Build Failure: "WebView2 not found"
**Solution**: WebView2 is pre-installed on GitHub runners. For local builds, install WebView2 runtime.

#### MSI Build Failure: "failed to run light.exe"
**Solution**: This error indicates the VBSCRIPT optional feature is disabled. To fix:

1. **Enable VBSCRIPT feature:**
   - Open **Settings** → **Apps** → **Optional features** → **More Windows features**
   - Locate **VBSCRIPT** in the list and ensure it's checked
   - Click **Next** and restart your computer if prompted

2. **Verify VBSCRIPT is enabled:**
   ```powershell
   # Check if VBSCRIPT is enabled
   Get-WindowsOptionalFeature -Online -FeatureName VBSCRIPT
   ```

3. **Alternative: Use NSIS instead of MSI:**
   If VBSCRIPT issues persist, you can build NSIS installers only by modifying `tauri.conf.json`:
   ```json
   {
     "bundle": {
       "targets": ["nsis"]
     }
   }
   ```

**Note**: VBSCRIPT is enabled by default on most Windows installations but is being deprecated in future Windows versions.

#### Installer Creation Failed
**Solution**: Check that `tauri.conf.json` includes Windows bundle configuration:
```json
{
  "bundle": {
    "windows": {
      "certificateThumbprint": null,
      "digestAlgorithm": "sha256",
      "allowDowngrades": true
    }
  }
}
```

#### Artifacts Not Uploaded
**Solution**: Verify workflow permissions allow artifact upload (see Prerequisites section).

### Debugging Builds

1. **Check workflow logs** in GitHub Actions tab
2. **Download artifacts** from failed builds to inspect
3. **Test locally** with same Node.js/Rust versions as CI
4. **Review** cluster integration patterns in reference docs

### Getting Help

- **GitHub Discussions**: Ask questions about build setup
- **Tauri Discord**: https://discord.gg/tauri for Tauri-specific issues
- **Repository Issues**: Report bugs or CI problems

## Security Considerations

### Code Signing
- Use proper certificates for production releases
- Never commit private keys to repository
- Rotate signing certificates before expiration

### Artifact Security
- GitHub artifacts are private to repository collaborators
- Public releases make artifacts available to everyone
- Consider security scanning of built executables

### Dependencies
- All dependencies are statically linked (no DLL dependencies)
- OpenSSL is vendored and doesn't require system installation
- Regular security updates via Dependabot

## Next Steps

After setting up Windows builds:

1. **Test the full pipeline** by creating a test release
2. **Document installation** instructions for end users
3. **Set up distribution** (direct download, package managers, etc.)
4. **Monitor build performance** and optimize as needed

The Windows build setup preserves all existing Linux functionality while adding robust Windows support for broader user deployment.

