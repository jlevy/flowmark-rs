---
type: is
id: is-01m15b4m3gza8en9qcqv5xd7fy
title: Reference images are flattened to inline form in both ports
kind: bug
status: open
priority: 2
version: 1
labels: []
dependencies: []
created_at: 2026-08-28T23:26:35.120Z
updated_at: 2026-08-28T23:26:35.120Z
---
render_image (Python) and its Rust counterpart always emit inline form, so a reference image loses its authored form:

  ![alt][ref]  ->  ![alt](/pic.png)
  ![ref][]     ->  ![ref](/pic.png)
  ![ref]       ->  ![ref](/pic.png)

Same class of source-fidelity loss that jlevy/flowmark#75 fixed for links, found during the senior review of that PR. After #75 links and images are inconsistent about it.

Verified both ports flatten identically, so this is a shared pre-existing gap, NOT a parity divergence — fixing one side alone would create one. Needs the upstream-first flow: agree the intended bytes in Python, replicate in flowmark-rs, move the shared golden once.

Python fix mirrors CustomLink: a CustomImage recording whether the image was authored as a reference (an inline image is the only form whose alt text is followed by '(').
