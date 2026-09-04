---
type: is
id: is-01m15c8y8ssd03f6c0rmc5rnv8
title: 258 of 673 CommonMark conformance cases are deferred and ungated
kind: task
status: open
priority: 2
version: 1
labels: []
dependencies: []
created_at: 2026-08-28T23:46:25.176Z
updated_at: 2026-08-28T23:46:25.176Z
---
The CommonMark sub-manifest gates 673 cases; 258 of them (38%) carry the 'deferred' tag under a single owner, owner-fm-n0ww, and are skipped by shared_conformance_corpus_matches_or_has_a_current_divergence.

Surfaced while checking the 'exact parity' claim against the v0.3.2 release. A live Rust-vs-Python run over the spec corpus differs on 48 examples: 32 are the tracked parity-corpus divergences, and the other 16 are all deferred cases — 0034, 0091, 0147, 0195, 0196, 0202, 0213, 0216, 0217, 0237, 0289, 0307, 0315, 0319, 0500, 0612. Confirmed real differences, not noise; several are content changes rather than formatting, e.g.:

  0034: Rust decodes HTML entities in a code-fence info string ('``` f&ouml;&ouml;' -> '```föö'); Python preserves them but drops the leading space. Both differ from the golden.
  0147: Rust drops a space inside a fenced block ('``` aaa' -> '```aaa'); Python keeps it.
  0091: Python mangles a code span into an ATX heading; Rust is correct here.

Parity is exact over the gated set, and this is what is outside it. Worth splitting fm-n0ww into per-section beads so the deferred set shrinks against a real list rather than sitting behind one owner, and so cases where Rust is *more* correct than Python are recorded as such.
