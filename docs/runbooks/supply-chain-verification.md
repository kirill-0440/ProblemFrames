# Supply-Chain Verification Runbook

This runbook describes how to verify release assets produced by `.github/workflows/release-artifacts.yml`.

## Expected Release Security Assets

Each GitHub Release bundle should include:

- `SHA256SUMS.txt`
- `SBOM.spdx.json`
- `sha256-*.jsonl` (provenance bundles, one or more)
- `trusted_root.jsonl`

## Prerequisites

- `sha256sum` (or equivalent)
- `gh` CLI with attestation support (`gh attestation`)

## 1) Verify checksums

From the directory with the downloaded release assets:

```bash
sha256sum -c SHA256SUMS.txt
```

All tracked files must report `OK`.

## 2) Verify artifact provenance offline

Set repository and signer workflow values:

```bash
REPO="kirill-0440/ProblemFrames"
SIGNER_WORKFLOW="kirill-0440/ProblemFrames/.github/workflows/release-artifacts.yml"
```

Verify each asset against its digest-named provenance bundle:

```bash
for artifact in pf_lsp-linux-x64 pf_lsp-macos-x64 *.vsix SBOM.spdx.json SHA256SUMS.txt; do
  [ -f "${artifact}" ] || continue
  digest="$(sha256sum "${artifact}" | awk '{print $1}')"
  bundle="sha256-${digest}.jsonl"
  gh attestation verify "${artifact}" \
    --repo "${REPO}" \
    --bundle "${bundle}" \
    --custom-trusted-root trusted_root.jsonl \
    --signer-workflow "${SIGNER_WORKFLOW}"
done
```

Verification must succeed for every published artifact.
