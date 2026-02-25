#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
PY_README="$ROOT/repos/flowmark/README.md"
OUT_README="$ROOT/README.md"

if [[ ! -f "$PY_README" ]]; then
  echo "error: missing Python README at $PY_README" >&2
  exit 1
fi

cat > "$OUT_README" <<'HEADER'
# flowmark (Rust port)

> AUTO-GENERATED: Do not edit directly.
> Source: `repos/flowmark/README.md` + Rust preface from `scripts/generate-rust-readme.sh`.

Flowmark-rs is an auto-synced Rust port of the original Python
[flowmark](https://github.com/jlevy/flowmark).

## Rust Port Context

- Install Rust CLI (source): `cargo install flowmark`
- Install Rust CLI (binary): `cargo binstall flowmark`
- Primary command: `flowmark` (`flowmark-rs` is also available in this repo)
- Port sync process: [`docs/port-sync-playbook.md`](docs/port-sync-playbook.md)
- Porting methodology: [rust-porting-playbook](https://github.com/jlevy/rust-porting-playbook)

---
HEADER

# Append Python README body, skipping the first top-level title line.
awk 'NR == 1 && $0 ~ /^# / { next } { print }' "$PY_README" >> "$OUT_README"

# Repoint copied Python relative docs links to canonical upstream docs URLs.
# This avoids broken links (e.g., docs/installation.md) and incorrect local targets.
tmp="$(mktemp)"
sed 's#](docs/#](https://github.com/jlevy/flowmark/blob/main/docs/#g' "$OUT_README" > "$tmp"
mv "$tmp" "$OUT_README"

echo "Generated $OUT_README from $PY_README"
