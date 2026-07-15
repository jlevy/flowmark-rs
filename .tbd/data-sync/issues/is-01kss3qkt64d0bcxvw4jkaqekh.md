---
type: is
id: is-01kss3qkt64d0bcxvw4jkaqekh
title: "PARITY GAP: inline code span backtick fence on escaped backticks (test gap_e)"
kind: bug
status: closed
priority: 2
version: 3
labels:
  - parity
dependencies: []
parent_id: is-01kss3qk7et1jycwp3snxgs7zd
created_at: 2026-05-29T05:36:22.854Z
updated_at: 2026-05-29T06:27:06.409Z
---
comrak parses escaped backticks in code spans differently from marko; naive decode-then-pad fix was net-negative on corpus (reverted). Failing test gap_e_inline_code_backtick. Candidate for tolerated variation or deep marko-fence-algorithm work.
