<a id="about"></a>
## About this guide

[简体中文](../zh-CN/index.html#about)

`lichtung-plot` is a standalone scientific renderer for owned immutable display
values. It owns display statistics, validation, text, layout and SVG/PNG
publication. Applications own scientific analysis and inferential decisions.
There is no network access, font provisioning, application runtime or PDF backend.

Start with [installation and a first plot](#installation), then learn
[native data](#data), [family selection](#families), [numeric presentation](#axes),
[appearance](#themes), [text](#text), [composition](#composition) and
[output](#output). The [gallery](#gallery) links pictures directly to code walkthroughs
on this page rather than routing through separate example pages.

The Chinese and English editions have equal status and equivalent scope. Neither
is an original, translation backlog or fallback. Topic anchors and example
references are shared; prose is reviewed for meaning, not literal sentence matching.

The Books explain usage. [Rust API documentation](https://docs.rs/lichtung-plot)
defines exact public signatures, while the workspace's
[specifications](https://github.com/PythiaLaurus/lichtung-plot/tree/main/specs)
define architecture and contracts. Code excerpts come from packaged executable
examples, not independent plotting implementations; run the complete linked
example rather than treating an excerpt as a standalone doctest.
