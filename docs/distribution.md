# Distribution Guide

## Cross-Platform Support

DuTop is built in Rust and supports multiple platforms:

### Tier 1 Support (Fully Tested)
- ✅ **macOS** (Intel & Apple Silicon)
- ✅ **Linux** (x86_64, ARM64)
- ✅ **Windows** (x86_64)

### Tier 2 Support (Should Work)
- FreeBSD
- Other Unix-like systems

## Platform-Specific Features

### macOS & Linux (Unix)
- Uses `blocks()` metadata for accurate disk usage (matches `du` exactly)
- Tracks inodes to handle hard links correctly
- Full color terminal support

### Windows
- Uses `file_size()` for disk allocation
- File index tracking for hard links
- ANSI color support in Windows Terminal

### Other Platforms
- Falls back to apparent file size
- Basic functionality maintained

## Building for Different Platforms

### Current Platform
```bash
cargo build --release
```
Binary location: `target/release/dutop` (or `dutop.exe` on Windows)

### Cross-Compilation

#### Install Cross-Compilation Tools
```bash
# Install cross (simplifies cross-compilation)
cargo install cross

# Or use rustup for specific targets
rustup target add x86_64-unknown-linux-gnu
rustup target add x86_64-pc-windows-gnu
rustup target add aarch64-apple-darwin
```

#### Build for Specific Targets

**Linux (x86_64):**
```bash
cargo build --release --target x86_64-unknown-linux-gnu
```

**Linux (ARM64/aarch64):**
```bash
cargo build --release --target aarch64-unknown-linux-gnu
```

**macOS (Intel):**
```bash
cargo build --release --target x86_64-apple-darwin
```

**macOS (Apple Silicon):**
```bash
cargo build --release --target aarch64-apple-darwin
```

**Windows (x86_64):**
```bash
cargo build --release --target x86_64-pc-windows-gnu
# or
cargo build --release --target x86_64-pc-windows-msvc
```

**Universal macOS Binary (Intel + Apple Silicon):**
```bash
# Build both architectures
cargo build --release --target x86_64-apple-darwin
cargo build --release --target aarch64-apple-darwin

# Combine into universal binary
lipo -create \
  target/x86_64-apple-darwin/release/dutop \
  target/aarch64-apple-darwin/release/dutop \
  -output dutop-universal
```

## Distribution Methods

### 1. GitHub Releases (Recommended)

Create pre-built binaries for each platform and attach to GitHub releases.

**Benefits:**
- Easy download for users
- Version tracking
- Automated with GitHub Actions
- Changelog included

**Release Structure:**
```
dutop-v0.1.0-x86_64-apple-darwin.tar.gz       # macOS Intel
dutop-v0.1.0-aarch64-apple-darwin.tar.gz      # macOS Apple Silicon
dutop-v0.1.0-x86_64-unknown-linux-gnu.tar.gz  # Linux x86_64
dutop-v0.1.0-aarch64-unknown-linux-gnu.tar.gz # Linux ARM64
dutop-v0.1.0-x86_64-pc-windows-msvc.zip       # Windows
```

### 2. Cargo Install (Source)

Users with Rust installed can build from source:

```bash
# From crates.io (once published)
cargo install dutop

# From git repository
cargo install --git https://github.com/yourusername/dutop
```

### 3. Homebrew (macOS/Linux)

Create a Homebrew formula:

```ruby
# Formula/dutop.rb
class Dutop < Formula
  desc "High-performance disk usage analysis tool"
  homepage "https://github.com/yourusername/dutop"
  url "https://github.com/yourusername/dutop/archive/v0.1.0.tar.gz"
  sha256 "..."
  license "MIT"

  depends_on "rust" => :build

  def install
    system "cargo", "install", *std_cargo_args
  end

  test do
    system "#{bin}/dutop", "--version"
  end
end
```

Installation:
```bash
brew tap yourusername/dutop
brew install dutop
```

### 4. Package Managers

**Arch Linux (AUR):**
```bash
# Create PKGBUILD
yay -S dutop
```

**Debian/Ubuntu (.deb):**
```bash
# Use cargo-deb
cargo install cargo-deb
cargo deb

# Install
sudo dpkg -i target/debian/dutop_0.1.0_amd64.deb
```

**Fedora/RHEL (.rpm):**
```bash
# Use cargo-rpm
cargo install cargo-rpm
cargo rpm build

# Install
sudo rpm -i target/release/rpmbuild/RPMS/x86_64/dutop-0.1.0-1.x86_64.rpm
```

**Windows (Scoop):**
```bash
scoop bucket add dutop https://github.com/yourusername/scoop-dutop
scoop install dutop
```

**Windows (Chocolatey):**
```bash
choco install dutop
```

### 5. Docker Image

```dockerfile
FROM rust:1.75 as builder
WORKDIR /app
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim
COPY --from=builder /app/target/release/dutop /usr/local/bin/
ENTRYPOINT ["dutop"]
```

Usage:
```bash
docker run --rm -v $(pwd):/data dutop /data
```

## Binary Size Optimization

Current optimized size: **~1.6MB** (with full optimization)

Already applied in `Cargo.toml`:
```toml
[profile.release]
opt-level = 3      # Maximum optimization
lto = true         # Link-time optimization
codegen-units = 1  # Better optimization
strip = true       # Strip symbols
```

Further reduction with UPX (optional):
```bash
upx --best --lzma target/release/dutop
# Can reduce to ~600KB
```

## Automated Release Process

See `.github/workflows/release.yml` for automated builds on:
- Git tag push (e.g., `v0.1.0`)
- Builds for all platforms
- Creates GitHub release
- Uploads binaries

## Publishing to crates.io

```bash
# Login (first time only)
cargo login

# Publish
cargo publish
```

Requirements:
- Update `Cargo.toml` metadata (repository, license, description)
- Add LICENSE file
- Test with `cargo package --list`
- Verify with `cargo publish --dry-run`

## Installation Verification

After installation, verify:

```bash
# Check version
dutop --version

# Check help
dutop --help

# Test run
dutop /tmp

# Verify accuracy
du -sh /tmp
dutop /tmp
# Should match!
```

## Platform-Specific Notes

### macOS
- Binary must be notarized for Gatekeeper (for distribution outside App Store)
- Universal binary recommended (supports Intel + Apple Silicon)
- Can be signed with Apple Developer account

### Linux
- Static linking recommended for maximum compatibility
- Consider musl target for truly static binaries:
  ```bash
  rustup target add x86_64-unknown-linux-musl
  cargo build --release --target x86_64-unknown-linux-musl
  ```

### Windows
- MSVC target preferred for better Windows integration
- Consider code signing for Windows SmartScreen
- Bundle Visual C++ redistributables if needed

## Minimal Dependencies

DuTop has zero runtime dependencies:
- ✅ Single static binary
- ✅ No external libraries required
- ✅ No Python/Node.js/JVM needed
- ✅ Works on fresh OS installation

This makes distribution extremely simple!
