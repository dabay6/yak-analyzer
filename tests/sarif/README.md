# SARIF 2.1.0's schema, copied

`sarif-schema-2.1.0.json` is a copy of the JSON Schema of SARIF 2.1.0, the format `audit` prints its
report in. The tests check every report they read against this copy (dead-code spec FIX-3).

- **Published at:** `https://docs.oasis-open.org/sarif/sarif/v2.1.0/errata01/os/schemas/sarif-schema-2.1.0.json`
- **Edition:** SARIF Version 2.1.0, OASIS Standard, Errata 01
- **SHA-256 of the file, with line feeds:** `c3b4bb2d6093897483348925aaa73af03b3e3f4bd4ca38cef26dcb4212a2682e`

SARIF 2.1.0 is a fixed standard, so the copy changes only if OASIS publishes another errata. When it
does, copy the file again and write the new address and digest here. `scripts/sarif-schema.test.mjs`
fails when the copy and the digest written here disagree.
