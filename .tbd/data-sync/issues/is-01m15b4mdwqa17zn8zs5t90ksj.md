---
type: is
id: is-01m15b4mdwqa17zn8zs5t90ksj
title: Add a wrap-width invariant gate to the Python port
kind: task
status: open
priority: 2
version: 1
labels: []
dependencies: []
created_at: 2026-08-28T23:26:35.451Z
updated_at: 2026-08-28T23:26:35.451Z
---
flowmark-rs#85 added tests/test_wrap_width_invariant.rs after a one-column overrun survived every existing gate. The Python port has no equivalent: tests/test_wrapping.py:253 asserts the bound for a single case.

Every other wrapping gate in both repos is a comparison — goldens, the shared conformance corpus, scripts/corpus-parity-check.sh — and a comparison is green when both sides are wrong together, over whatever inputs someone collected.

I ran the property against Python during the #75 review: 94,904 generated lines across two widths and all 32 escapable characters, 0 over width. Python holds it today; the point is to keep holding it. Port the Rust gate, including its two anti-vacuity guards: sweep the tail as well as the head (the head decides where a token lands, the tail decides whether the line stops on the column where an off-by-one shows), and assert the generated space actually reaches the case being covered.
