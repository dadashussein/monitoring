# Release Guide

## Creating a New Release

### Automatic Release (Recommended)

GitHub Actions automatically builds binaries for multiple architectures when you push a tag.

**Steps:**

1. **Update version in Cargo.toml:**
   ```toml
   [package]
   version = "1.0.0"  # Update this
   ```

2. **Commit changes:**
   ```bash
   git add Cargo.toml
   git commit -m "Bump version to 1.0.0"
   git push
   ```

3. **Create and push tag:**
   ```bash
   git tag v1.0.0
   git push origin v1.0.0
   ```

4. **Wait for GitHub Actions:**
   - Go to: https://github.com/YOUR_USERNAME/ubuntu-resource-monitor/actions
   - Watch the "Release" workflow
   - It will build binaries for:
     - x86_64 (Intel/AMD 64-bit)
     - aarch64 (ARM 64-bit)
     - armv7 (ARM 32-bit)

5. **Check the release:**
   - Go to: https://github.com/YOUR_USERNAME/ubuntu-resource-monitor/releases
   - Your release should have all binaries attached

### About GITHUB_TOKEN

**You don't need to do anything!** 

The `GITHUB_TOKEN` is automatically provided by GitHub Actions. It's a special token that:
- ✅ Is automatically created for each workflow run
- ✅ Has permissions to create releases and upload assets
- ✅ Expires after the workflow completes
- ✅ No manual configuration needed

**What we added:**
```yaml
permissions:
  contents: write  # Allows creating releases
```

This tells GitHub Actions that the workflow needs permission to write to the repository (create releases).

### Manual Release (If needed)

If you need to build manually:

```bash
# Build for current architecture
cargo build --release

# Binary will be at:
# target/release/ubuntu_resource_api
```

For cross-compilation:

```bash
# Install cross
cargo install cross

# Build for ARM64
cross build --release --target aarch64-unknown-linux-gnu

# Build for ARMv7
cross build --release --target armv7-unknown-linux-gnueabihf
```

## Release Checklist

Before creating a release:

- [ ] Update version in `Cargo.toml`
- [ ] Update `CHANGELOG.md` with new features/fixes
- [ ] Test the application locally
- [ ] Run all tests: `cargo test`
- [ ] Check for warnings: `cargo clippy`
- [ ] Format code: `cargo fmt`
- [ ] Update documentation if needed
- [ ] Commit all changes
- [ ] Create and push tag
- [ ] Wait for GitHub Actions to complete
- [ ] Test the release binaries
- [ ] Update Docker image if needed: `make publish`

## Testing Release Binaries

After release is created, test the installation:

```bash
# Download install script
wget https://raw.githubusercontent.com/YOUR_USERNAME/ubuntu-resource-monitor/main/install-binary.sh

# Run installer
sudo bash install-binary.sh

# Check if service is running
sudo systemctl status ubuntu-resource-monitor

# Test the application
curl http://localhost:8080/health
```

## Supported Architectures

The release workflow builds for:

| Architecture | Target Triple | Common Devices |
|--------------|---------------|----------------|
| x86_64 | x86_64-unknown-linux-gnu | Intel/AMD servers, desktops |
| ARM64 | aarch64-unknown-linux-gnu | Raspberry Pi 4, ARM servers |
| ARMv7 | armv7-unknown-linux-gnueabihf | Raspberry Pi 3, older ARM devices |

## Troubleshooting

### GitHub Actions fails

Check the workflow logs:
1. Go to Actions tab
2. Click on the failed workflow
3. Check the error messages
4. Common issues:
   - Missing dependencies
   - Cross-compilation errors
   - GitHub token permissions

### Binary doesn't work

- Check architecture: `uname -m`
- Verify binary is executable: `chmod +x ubuntu_resource_api`
- Check dependencies: `ldd ubuntu_resource_api`
- Try running directly: `./ubuntu_resource_api`

## Version Numbering

We follow [Semantic Versioning](https://semver.org/):

- **MAJOR** version (1.x.x): Breaking changes
- **MINOR** version (x.1.x): New features, backward compatible
- **PATCH** version (x.x.1): Bug fixes, backward compatible

Examples:
- `v1.0.0` - Initial release
- `v1.1.0` - Added new feature
- `v1.1.1` - Fixed a bug
- `v2.0.0` - Breaking changes
