# 0005. Distribution: OCI artifacts, Sigstore signatures, TUF metadata

- Status: accepted · Date: 2026-10-10 · Class: one-way · Source: north star §07, §12

## Decision
Descriptions and logic modules are OCI artifacts with DriverDesk media types. Every artifact is signed (Sigstore; key-based for vendors, keyless OIDC for project CI) and logged in a transparency log (Rekor). Clients use TUF metadata for freshness, rollback protection and revocation. Private artifacts are age-encrypted to authorised client keys.

## Why
Don't build package storage or a PKI. OCI registries exist everywhere (self-hosting = run Zot), and Sigstore + TUF are what PyPI, npm and Kubernetes use.
