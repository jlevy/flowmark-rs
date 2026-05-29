# Upstream fixes (to file on jlevy/flowmark)

> **Doc status:** Rust port-specific.
> Patches here are fixes for bugs found in the upstream Python
> [flowmark](https://github.com/jlevy/flowmark) during parity work on the Rust port.
> They are staged here (rather than pushed) because this repo’s tooling is scoped to
> `flowmark-rs`; a maintainer applies and pushes them upstream.

Per the porting playbook: when the port surfaces a genuine *upstream* bug (where the
Rust port is the more-correct implementation), fix it upstream with a test, then
synchronize the port.
Each patch below includes its regression test in the same format, so the same behavior
is asserted on both sides.

## Patches

### `fmr-qmd8-escaped-backtick-codespan-mispair.patch`

**Bug:** a line containing an escaped-backtick inline code span (` `\` `) makes marko's
code-span matcher treat the backtick in ` \` `` as a span delimiter, mis-pairing every
following backtick on the same line and stripping the spaces around later code spans:

```
`\`` and `x` status from `y`   ->   `\`` and `x`status from`y`
```

**Fix:** protect backslash-escaped backticks with a Private Use Area placeholder before
`marko.parse` and restore after render (the same escape-protection the Rust port uses).
Includes a regression test in `tests/test_filling.py`.

**Validation:** full flowmark pytest suite passes (335/0). After applying, the two
affected corpus files become byte-identical between Python and the Rust port.

**To apply upstream:**

```bash
cd <flowmark checkout>
git checkout -b fix/escaped-backtick-codespan-mispair
git am < fmr-qmd8-escaped-backtick-codespan-mispair.patch
# or: git apply for an unsigned-off application
uv run --with pytest python -m pytest tests/   # expect all green
```

Tracked as bead `fmr-qmd8`. The Rust side already produces the correct output
(`tests/test_known_parity_gaps.rs::gap_e2_escaped_backtick_preserves_spaces`).
