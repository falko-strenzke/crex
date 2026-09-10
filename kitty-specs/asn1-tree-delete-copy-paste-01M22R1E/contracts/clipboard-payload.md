# Contract: clipboard payload for tree copy and paste

## What copy writes (normal set, Ctrl+C / Edit ▸ Copy)

The DER encoding of every operand element, concatenated in tree order,
written to the system clipboard as **upper-case hex digits without
separators** and no trailing newline, exactly as the hex editor's Ctrl+C
writes octets. The same bytes fill the element buffer.

Example (one BOOLEAN TRUE and one NULL): `0101FF0500`

## What paste accepts (Ctrl+V / Edit ▸ Paste …)

Clipboard text or bytes are interpreted in this order and the first
reading that succeeds is used; the status line names it:

| Order | Reading | Accepts | Status wording |
|-------|---------|---------|----------------|
| 1 | hex | only hex digits and whitespace, even digit count | "read as hex digits" |
| 2 | base64 | valid base64 (whitespace ignored) decoding to ≥1 byte | "decoded from base64" |
| 3 | PEM | one or more `-----BEGIN x-----` … `-----END x-----` blocks; labels may differ; bodies decoded and concatenated in order | "decoded from PEM (n blocks)" |
| 4 | raw | anything else | "taken as raw bytes" |

The resulting bytes must parse as a complete forest: `parse_forest` must
consume every byte. Otherwise the paste is refused with the parser's message
prefixed by the reading used, e.g. `read as hex digits: truncated length
field at offset 7`, and the document is unchanged.

Odd hex digit count is refused at step 1 with "odd number of hex digits"
rather than falling through to base64, matching the hex editor's rule that
`DEADBEE` is a typo, not base64.

## Element buffer fallback

When the system clipboard is empty, holds no text, or no helper program is
available, Ctrl+V pastes the element buffer if it is non-empty and says
"pasted n elements from the element buffer". With both empty: "nothing to
paste".
