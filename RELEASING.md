# Specado Release Process

This document describes the release process for Specado packages to PyPI and npm.

## Overview

Specado uses a tag-based release process powered by GitHub Actions. When you push a version tag, the CI/CD pipeline automatically:
1. Validates version consistency
2. Builds Python wheels and Node.js packages
3. Publishes to the appropriate registries
4. Creates a GitHub release with artifacts

## Prerequisites

### For Maintainers
- Push access to the repository
- Ability to create tags

### Repository Setup (One-time)
1. **npm Token**: Already configured in GitHub Secrets as `NPM_TOKEN`
2. **PyPI Trusted Publisher**: Must be configured (see below)

## Configuring PyPI Trusted Publisher

### For PyPI (Production)
1. Go to https://pypi.org/manage/account/publishing/
2. Add a new publisher:
   - Owner: `specado`
   - Repository: `specado`
   - Workflow name: `release.yml`
   - Environment: (leave blank)

### For TestPyPI (Testing)
1. Go to https://test.pypi.org/manage/account/publishing/
2. Add the same configuration as above

This enables keyless publishing without storing PyPI tokens.

## Version Management

### Version Source of Truth
The version in `specado-core/Cargo.toml` is the single source of truth. All other versions are derived from or validated against this.

### Version Format
We follow [Semantic Versioning](https://semver.org/):
- **Stable**: `1.0.0`, `2.3.1`
- **Release Candidate**: `1.0.0-rc.1`, `1.0.0-rc.2`
- **Beta**: `1.0.0-beta.1`, `1.0.0-beta.2`
- **Alpha**: `1.0.0-alpha.1`, `1.0.0-alpha.2`

## Release Types

### Stable Release
Publishes to PyPI and npm (latest tag)
```bash
git tag v1.0.0
git push origin v1.0.0
```

### Pre-release (RC/Beta)
Publishes to TestPyPI and npm (beta tag)
```bash
git tag v1.0.0-rc.1
git push origin v1.0.0-rc.1
```

### Alpha Release
Publishes to TestPyPI only
```bash
git tag v1.0.0-alpha.1
git push origin v1.0.0-alpha.1
```

### Package-Specific Release
```bash
# Python only
git tag py-v1.0.0
git push origin py-v1.0.0

# Node.js only
git tag node-v1.0.0
git push origin node-v1.0.0
```

## Release Process

### Option 1: Manual Process (Recommended)

1. **Update version in Cargo.toml**
   ```bash
   # Edit specado-core/Cargo.toml
   version = "1.0.0"
   ```

2. **Update CHANGELOG.md**
   ```bash
   # Document changes for this version
   ```

3. **Commit changes**
   ```bash
   git add -A
   git commit -m "chore: prepare release v1.0.0"
   git push origin main
   ```

4. **Create and push tag**
   ```bash
   git tag -a v1.0.0 -m "Release v1.0.0"
   git push origin v1.0.0
   ```

5. **Monitor release**
   - Go to Actions tab on GitHub
   - Watch the "Release" workflow
   - Check the created GitHub release

### Option 2: Using Version Bump Workflow

1. **Run the workflow**
   - Go to Actions → Version Bump Helper
   - Click "Run workflow"
   - Enter version (e.g., `1.0.0`)
   - Optionally check "Create and push tag"

2. **If tag not created automatically**
   ```bash
   git pull
   git tag v1.0.0
   git push origin v1.0.0
   ```

## Testing Releases

### Test with Alpha Version
```bash
# 1. Bump to alpha version
git tag v0.1.1-alpha.1
git push origin v0.1.1-alpha.1

# 2. Wait for workflow to complete

# 3. Test installation from TestPyPI
pip install -i https://test.pypi.org/simple/ specado==0.1.1a1

# Note: TestPyPI normalizes versions (0.1.1-alpha.1 becomes 0.1.1a1)
```

### Test npm Package
```bash
# For beta/alpha releases
npm install specado@alpha
npm install specado@beta

# For specific version
npm install specado@0.1.1-alpha.1
```

## Monitoring and Verification

### Check Workflow Status
1. Go to [Actions](https://github.com/specado/specado/actions)
2. Click on the "Release" workflow run
3. Verify all jobs complete successfully

### Verify Package Availability

#### PyPI
- Stable: https://pypi.org/project/specado/
- Test: https://test.pypi.org/project/specado/

#### npm
- Package page: https://www.npmjs.com/package/specado
- Check versions: `npm view specado versions`

### GitHub Release
- Releases page: https://github.com/specado/specado/releases
- Verify artifacts are attached
- Check release notes

## Troubleshooting

### Version Mismatch Error
**Problem**: Workflow fails with "Version mismatch" error
**Solution**: Ensure version in tag matches `specado-core/Cargo.toml`

### PyPI Publishing Fails
**Problem**: PyPI publishing fails with authentication error
**Solution**: 
1. Verify Trusted Publisher is configured
2. Check workflow filename matches configuration
3. Ensure you're using the pypa/gh-action-pypi-publish action

### npm Publishing Fails
**Problem**: npm publishing fails
**Solution**:
1. Verify NPM_TOKEN secret is valid
2. Check package name isn't taken
3. Ensure @specado organization exists

### TestPyPI Installation Issues
**Problem**: Can't install from TestPyPI
**Note**: TestPyPI doesn't mirror PyPI dependencies. You may need:
```bash
pip install -i https://test.pypi.org/simple/ --extra-index-url https://pypi.org/simple specado
```

## Security Notes

1. **npm Token**: Stored as GitHub secret, rotated quarterly
2. **PyPI**: Uses Trusted Publisher (OIDC) - no tokens stored
3. **Tag Protection**: Consider protecting `v*` tags in repository settings
4. **Signed Tags**: For extra security, use GPG-signed tags:
   ```bash
   git tag -s v1.0.0 -m "Release v1.0.0"
   ```

## Best Practices

1. **Always test with alpha/beta first** before stable releases
2. **Update CHANGELOG.md** before creating release tag
3. **Use annotated tags** (`-a` flag) for better Git history
4. **Monitor the Actions tab** during release
5. **Verify package installation** after release
6. **Keep versions synchronized** across all packages

## Quick Reference

### Common Commands
```bash
# Check current version
grep '^version' specado-core/Cargo.toml

# Create stable release
git tag v1.0.0 && git push origin v1.0.0

# Create pre-release
git tag v1.0.0-rc.1 && git push origin v1.0.0-rc.1

# Create alpha (test only)
git tag v1.0.0-alpha.1 && git push origin v1.0.0-alpha.1

# List all tags
git tag -l

# Delete a tag (if needed)
git tag -d v1.0.0
git push origin :refs/tags/v1.0.0
```

### Version Patterns
- Stable: `v1.0.0` → PyPI + npm
- RC/Beta: `v1.0.0-rc.1` → TestPyPI + npm@beta
- Alpha: `v1.0.0-alpha.1` → TestPyPI only
- Python only: `py-v1.0.0`
- Node only: `node-v1.0.0`

## Support

For issues with the release process:
1. Check the [Actions logs](https://github.com/specado/specado/actions)
2. Review this documentation
3. Open an issue if problems persist