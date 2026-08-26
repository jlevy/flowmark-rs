# flowmark-dev-tools

Dev tools for the flowmark-rs porting effort.
Provides test discovery and mapping validation between the Python flowmark repo and the
Rust flowmark-rs port.

## Usage

```bash
# Discover Python tests (clones from GitHub at pinned tag)
uv run flowmark-dev discover-python

# Discover Python tests from the exact local upstream checkout
uv run flowmark-dev discover-python --local-path ../repos/flowmark

# Discover Rust tests
uv run flowmark-dev discover-rust

# Generate or update skeleton mapping file
uv run flowmark-dev init-mapping

# Check mapping completeness
uv run flowmark-dev check-mapping
```
