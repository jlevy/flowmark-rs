# Test Corpora and Provenance

> **Doc status:** Rust port-specific (no upstream equivalent).
> Defines which test layers are portable contract evidence and how external parity
> corpora are identified and audited.

Flowmark uses overlapping test layers deliberately.
Focused cases make failures easy to diagnose; broad documents and CommonMark expose
interactions; tryscript exercises real CLI workflows.
Portable behavior is defined once in the Python repository and consumed directly by both
implementations.

## Authoritative Shared Layers

| Layer | Source | Normal Python CI | Normal Rust CI | Purpose |
| --- | --- | --- | --- | --- |
| Conformance manifest | `repos/flowmark/tests/parity_corpus/manifest.toml` | Native Python runner | Native Rust runner | Exact arguments, environment, bytes, exit status, file trees, timeouts, and fixed points |
| Topic and reference documents | Paths named by the manifest | Direct | Through the pinned submodule | Whole-document and cross-family interactions |
| CommonMark 0.31.2 | Generated registry and fixtures under `tests/parity_corpus/spec/` | Direct | Through the pinned submodule | Broad standard-Markdown syntax sweep |
| Historical parity cases | Manifest cases tagged `historical` or `parity` | Direct | Through the pinned submodule | Previously discovered cross-parser corner cases |
| Tryscript sessions | `repos/flowmark/tests/tryscript/*.tryscript.md` | Built Python CLI | Cargo-built Rust CLI | End-to-end CLI and filesystem workflows |
| Language-specific tests | Each implementation repository | Yes | Yes | Scanner, bridge, adapter, and error-boundary internals only |

Normal Rust behavior tests do not invoke Python and do not contain copied portable
goldens. The exact expected bytes come from the pinned `repos/flowmark` gitlink.
Every portable behavior change receives a stable `FM-*` change ID and a mapping
disposition.

## External Transition Audits

An external corpus is an acceptance-time differential audit, not a source of expected
output and not a normal CI dependency.
It is useful before advancing a whole-program baseline because large real documents
expose interactions that focused cases may miss.

Every external audit must record:

- source repository and immutable source commit when known;
- corpus-relative file list and Markdown file count;
- aggregate SHA-256 produced by `scripts/corpus-parity-check.sh`;
- exact Python executable commit or release and exact Rust commit;
- complete selected-file lists and proof that both binaries selected every corpus file;
- complete output diff, including the empty diff for a pass;
- minimal shared regression cases for every real discrepancy found.

Do not accept a percentage, a truncated diff, or an unexplained skipped file as parity.

## Recovered AI Trade Arena Corpus

The machine-local corpus used during the original Rust port was described only as
`attic/test-docs` with 623 files.
That directory was never committed, and its exact contents remain unrecoverable.
Do not identify it with the recovered corpus below.

A later three-way audit snapshot was recovered from
`/Users/levy/wrk/github/flowmark-rs/attic/`:

| Directory | Meaning | Markdown files |
| --- | --- | ---: |
| `docs-orig` | Input snapshot | 670 |
| `docs-py` | Historical Python-formatted result | 670 |
| `docs-rust` | Historical Rust-formatted result | 670 |

The two historical formatted trees are byte-identical across all 670 Markdown files.
The input snapshot has this aggregate digest:

```text
68c6d370c3ea43eea3b37dca50c769f5655b5817fd6903d12a042255122e9a41
```

The path set exactly matches the 670-file `docs/` tree in
`https://github.com/dxdt-labs/ai-trade-arena.git` at commit
`4f9c89a10cacbda6752806c817cf8840619a3617`, the closest repository state before the
snapshot timestamps on 2026-02-24. Of those paths, 471 input files are byte-identical to
that commit and 199 differ.
The most defensible provenance is therefore “an AI Trade Arena documentation
working-tree snapshot near `4f9c89a`,” not “a clean copy of that commit.”
The digest, rather than the candidate Git commit, is the exact snapshot identity.

The 2026-08-26 transition audit ran the current Python contract and Rust port on all 670
files. It found and promoted two missing shared interactions:

1. Angle-bracket comparisons such as `<15min` in GFM table cells were over-recognized as
   raw HTML.
2. A multiline code span containing `|` in prose was incorrectly divided into table
   cells by the Rust inline-scope scanner.

After both fixes, the audit passed with zero byte differences.

## Running a Pinned Audit

Build Rust in release mode, provide an explicit corpus directory, and identify a local
Python executable by immutable commit:

```bash
cargo build --locked --release

FLOWMARK_PARITY_PYTHON_BIN=/absolute/path/to/flowmark \
FLOWMARK_PARITY_PYTHON_LABEL='flowmark <full-commit>' \
FLOWMARK_PARITY_EXPECTED_CORPUS_SHA256='<corpus-digest>' \
FLOWMARK_PARITY_REPORT_DIR='target/corpus-parity/<run-id>' \
scripts/corpus-parity-check.sh /absolute/path/to/corpus target/release/flowmark
```

The helper fails before formatting if the digest changes, either executable omits a
Markdown file, or the selected file sets differ.
It retains metadata, the corpus file list, both selected-file lists, stdout and stderr,
a recursive difference list, and the complete patch under the report directory.

For a new corpus, prefer a clean checkout at an immutable source commit.
Record any deliberate filtering as a checked-in file manifest rather than relying on
ambient `.gitignore` files.
If redistribution is permitted and the corpus is small enough, check it into the source
contract; otherwise keep it as a documented transition audit with its digest and
reconstruction command.

<!-- This document follows common-doc-guidelines.md.
See github.com/jlevy/practical-prose and review guidelines before editing.
-->
