# crex

crex is a terminal (TUI) viewer **and editor** for ASN.1 BER/DER files with support for cryptographic data structures such as X.509 certificates and CRLs, and cryptographic operations on them.
It is written in
Rust with use of the [ratatui](https://ratatui.rs) TUI framework. A file browser and the nested ASN.1 structure is shown
as a foldable tree in the left panes while the right area shows the selected
data element's decoded value and a hex dump of its content octets, which can be
edited in place.

Besides being a generic ASN.1/DER editor, the main functionality of crex is editing and viewing cryptographic object files such as X.509 certificates and related data structures individually as well as working with folder structures containing such objects. It support for cryptographic operations that affect multiple objects in the opened directory. This is an overview of its most important features:

- Generic ASN.1/DER (PEM or binary) viewing and editing
  - Content search function
- X.509 certificates and CRLs:
  - Issuer-Subject relations visualized in the file tree panel (after marking root certificates as trusted)
  - Semantic-aware editing of the content of various certificate fields
  - Re-signing: A certificate can be edited and efficiently re-signed if the certificate's issuer's signing key is available
  - Re-keying: A certificate's public key algorithm can be modified and the known objects (certificates, CRLs, CMS messages) issued by that certificate can be automatically re-signed
- CMS signed and encrypted data
- PKCS#8 and PKCS#12 private key files:
  - Decrypt and show decrypted data in the ASN.1 tree
  - Use unlocked keys for object re-signing
- Public-key cryptographic algorithm support:
  - traditional: RSA, ECDSA, Ed25519
  - PQC:
    - ML-DSA, SLH-DSA
    - experimental support for XMSS and LMS: neither of the current crypto-backends supports certificates with these algorithms properly (OpenSSL 3.6: not at all, Botan 3.12: has bugs related to certificate path validation with these algorithms)

## Credits

The tools dump output format as well as that of the ASN.1/DER tree view purposely imitate the [dumpasn1 tool from Peter Gutmann](https://www.cs.auckland.ac.nz/~pgut001/dumpasn1.c).

The parser's structural output (offsets, lengths, type names, including the
"encapsulated ASN.1 inside OCTET STRING / BIT STRING" heuristic) replicates
Peter Gutmann's `dumpasn1` and is verified against the real binary by the
test suite. See [DESIGN.md](DESIGN.md) for the full design.

## Usage
- Opening a folder in the TUI:
  ```sh
  crex my_dir/
  ```
  This is the most powerful mode that enables the cryptographic functionality on single object files as well as groups of related objects.
- Opening a single file in the TUI:
  ```sh
  crex cert.der
  ```
  edits overwrite `cert.der` on Ctrl+S.
  To save edits to out.der instead use:
  ```sh
  crex -o out.der cert.der
  ```
  This mode has no support for the cryptographic operations.
- Dump the contents of file to stdout in the same format as the dumpasn1 (no TUI)
  ```sh
  `crex --dump cert.der`      
  ```

Input may be raw BER/DER, PEM, bare base64, or hex text; saving re-wraps
the edited data in the same outer format.

## The name

**crex** — pronounced *c-rex* or *krex* — is a blend of **cr**ypto and
l**ex**er: it parses and edits cryptographic object structures (currently
ASN.1/DER). *Crex crex* is also the scientific name of the corn crake, a bird
([Wachtelkönig](https://de.wikipedia.org/wiki/Wachtelk%C3%B6nig)).


## Build

```sh
cargo build --release        # binary in target/release/crex
cargo test                   # includes the dumpasn1 comparison if installed
```

The XMSS and HSS/LMS backends link Botan, built from bundled source via the
`botan` crate's `vendored` feature, so the resulting binary is self-contained
(no system Botan or shared library at runtime). Building it needs a C++17
compiler, GNU `make`, and Python 3 on `PATH` — Botan's `configure.py`. The
first build spends a few minutes compiling Botan; later builds are cached.

### OpenSSL 3.6 (for OpenSSL-based LMS verification)

The `openssl` crate is used for ML-DSA / SLH-DSA and for verifying single-level
**LMS** certificates. LMS verification landed in **OpenSSL 3.6** and is
disabled by default, so OpenSSL only verifies LMS when the build links an
OpenSSL **3.6 or newer built with `enable-lms`**. Without one, crex detects
the missing algorithm at runtime and verifies single-level LMS through Botan
instead (which always handles multi-level HSS). To have OpenSSL do it, point
`openssl-sys` at such a build with the standard environment variables:

```sh
# Build OpenSSL 3.6 with LMS once (any prefix you like):
curl -L https://github.com/openssl/openssl/releases/download/openssl-3.6.0/openssl-3.6.0.tar.gz | tar xz
cd openssl-3.6.0
./Configure --prefix="$HOME/.local/openssl-3.6-lms" --libdir=lib enable-lms no-shared no-docs
make -j"$(nproc)" && make install_sw && cd ..

# Then build/test crex against it:
export OPENSSL_DIR="$HOME/.local/openssl-3.6-lms"
export OPENSSL_STATIC=1
cargo build --release
```

The CI (`.github/workflows/appimage.yml`) builds this OpenSSL from source and
sets the same variables. A distribution OpenSSL (e.g. Debian's 3.6 packages)
typically lacks LMS; LMS certificates still verify and re-sign there, via the
Botan fallback.


## ASN.1 specifications

On startup, every file in `specs/asn1/` is parsed as an ASN.1 module
(1988 syntax) and the opened document is structurally matched against
every type definition. When a definition fits the whole structure — with
the bundled RFC 5280 modules that identifies X.509 certificates
(`Certificate`) and CRLs (`CertificateList`) — the tree is augmented with
the field and type names from the specification:

```
▾ SEQUENCE (3 elem)  ·Certificate
  ▾ tbsCertificate: SEQUENCE (8 elem)  ·TBSCertificate
    ▾ version: [0] (1 elem)
        INTEGER 2  ·Version
      serialNumber: INTEGER 70 60 96 41 …
```

The content pane shows the selected element's spec name on a `Spec` line,
and the identified document type appears in the tree title. Additional
specification files dropped into `specs/asn1/` are picked up
automatically (the directory is looked up next to the executable, in the
current directory, or via `$CREX_SPECS`).

## Keys

| Key | Action |
|-----|--------|
| `↑`/`k`, `↓`/`j`, `PgUp`/`PgDn`, `g`, `G` | navigate the tree |
| `←`/`h`, `→`/`l`, `Enter`/`Space` | collapse / expand / toggle |
| `e` | edit the selected element's value in its natural, type-specific form |
| `E` | open the edit menu: tag type / hex / base64 / raw binary / type specific |
| `i` | insert a new element after the selected one (type-picker dialog, then value) |
| `I` | insert a new element as first child of the selected constructed element |
| `d` `d` | delete the selected element (press twice to confirm) |
| `J` / `K` | move the selected element down / up among its siblings |
| `Enter` / `Esc` | apply / cancel the edit |
| `Ctrl+S` | save |
| `z` | decrypt a supported encrypted PKCS#8 key and show its virtual plaintext tree |
| `[` / `]` | scroll the content pane |
| `q` | quit (`q q` discards unsaved changes) |

Editing notes: `e` opens the type-specific editor directly (mode 5 of the
menu below); for OCTET/BIT STRINGs and unknown types that is the hex
editor. All value editors work on the element's *content octets* (for
BIT STRING including the leading unused-bits octet). Lengths of all
enclosing elements are recomputed automatically. Content of constructed
elements must remain valid ASN.1, otherwise the edit is rejected.

For a supported PBES2-encrypted PKCS#8 key, the encrypted-data node has a
`decrypted content not available` child until `z` is used to enter the
password. Successful decryption replaces that placeholder with a foldable,
editable virtual ASN.1 tree. Editing the virtual tree re-encrypts it with a
fresh IV; editing the encrypted data refreshes the virtual tree while the
decryption remains valid.

Inserting (`i`/`I`) first opens a popup dialog to choose the ASN.1 type,
with one column per bit field of the identifier octet: **class**
(universal / application / context-specific / private, bits 8-7), **form**
(primitive / constructed, bit 6) and **tag number** (bits 5-1; a list of
the named universal types, or a typed number for the other classes).
Illegal form combinations (e.g. primitive SEQUENCE) are ruled out
automatically and the resulting identifier octets are previewed live.
After confirming, only the value is entered in the hex editor (empty by
default); identifier and length octets are generated, and the lengths of
all enclosing elements are recomputed automatically — as for every other
edit operation.

`E` opens an **edit menu** for the selected element with five modes:

1. **Tag type** — the type-picker dialog pre-populated with the element's
   current class/form/tag; confirming re-tags the element in place while
   keeping its content octets.
2. **Hex** — the same hex editor as `e`.
3. **Base64** — the value as base64 text (pre-filled, whitespace ignored).
4. **Raw binary** — typed or pasted characters become the value bytes
   verbatim (UTF-8); useful for pasting data from the clipboard.
5. **Type specific** — the value in its most natural form: decimal entry
   for INTEGER/ENUMERATED/REAL, dot notation for OBJECT IDENTIFIER,
   TRUE/FALSE for BOOLEAN, plain text for the string types (encoded as
   UCS-2/UCS-4 for BMPString/UniversalString), hex for OCTET/BIT STRING,
   and for UTCTime/GeneralizedTime a form with separate year, month, day,
   hour, minute and second fields — no date-format guessing needed.

Every editor shows live feedback (resulting byte count or the validation
error) and applies with `Enter`; lengths are recomputed automatically.

## License

Licensed under the [Apache License, Version 2.0](LICENSE).
