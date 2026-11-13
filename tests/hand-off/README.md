# The hand-off format's schema, copied

`hand-off-1.schema.json` is a copy of the JSON Schema of hand-off format 1, the file yak writes for
one checkout and hands to the analyzer. The original is in the yakshaver repository. The tests check
every hand-off file they use against this copy (spec HO-1).

- **Path there:** `packages/core/src/stacks/hand-off/hand-off-1.schema.json`
- **Pull request:** dabay6/yakshaver#213, not merged when the copy was made
- **Commit:** `0a47b4e134711667010446b046e65aee0bc31540`
- **SHA-256 of the file, with line feeds:** `1f0a37fe9c6b7e4c78fbf0de599d13a6f73b20405eb1f4bcd58c1cbae319b948`

When the schema changes in yakshaver, copy it again, write the new commit and digest here, and change
the tests that depend on it in the same commit. `scripts/hand-off-schema.test.mjs` fails when the copy
and the digest written here disagree.
