# Coordinated Canvas Natural Width

The shared-header/category/range-group Canvas regression exposed a pre-existing
natural-width failure, independently reproduced at `87165d0`. Its initial width
mixed axis gutter preferences into individual body preferences, then compared
column-header hard widths against that combined number. Later range-group
alignment reserved the actual external gutters, leaving less body width than the
already-required headers. The Root track allocation correctly refused.

Natural width now initializes the same external column-inset representation used
by assigned layout. It measures each Plot's existing horizontal gutter preference,
coordinates group maxima before width selection, and leaves body/title/guide and
column-header requirements on their actual inner tracks. The resulting grid
preference includes those external gutters exactly once. Later measured gutters
replace the preference through the existing bounded parent feedback.

This repairs the first natural-width choice; it adds no width reselection,
grow-until-fit loop or previous-best fallback. Declared sparse tracks, headers,
independent panel policies, explicit dimensions and existing hard refusal remain
owned by the same canonical Grid allocator. No fixture or automatic-size golden
was changed.

## Executed Checks

Locked/offline focused checks passed: seven existing composed-Canvas contracts
(including the formerly failing range-group/header case), one spanning-header
contract, two shared-category domain/order contracts, and one mixed sparse Canvas
contract across automatic, fixed-width and exact roots with complete local/root
prose and replaced child requests: **11 external tests**. The numeric recorded-
height cache test also passed.

Strict all-target Clippy, formatting and whitespace checks passed. No tests were
added, deleted or moved; library discovery remains 671 including 17 ignored.
Temporary diagnosis was removed. The detached baseline worktree was removed after
its independently recorded focused failure.
