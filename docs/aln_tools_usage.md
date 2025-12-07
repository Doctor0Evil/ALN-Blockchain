# aln-tools CLI usage

## Snapshot hashes from CSV

Input CSV columns: address,denom,balance,height,chain_id

```
aln-tools snapshot-hash \
  --input snapshots.csv \
  --output snapshot_hashes.json \
  --asset-id my_asset_id \
  --artifacts artifacts
```

## Generate ALN20 allocations (AU.ET and CSP) from CSV using a scaling profile

```
aln-tools allocations \
  --input snapshots.csv \
  --output allocations.json \
  --profile malicious_cleanup \
  --d-src 6 \
  --d-aln 6
```

This picks compression constants from `config/scaling.yaml` and generates a JSON mapping of addresses to AU.ET and CSP minimal units.

## Alternative: pass constants directly
```
aln-tools allocations \
  --input snapshots.csv \
  --output allocations.json \
  --c-e 1e-12 \
  --c-s 5e-13 \
  --d-src 6 \
  --d-aln 6
```


Note: `--profile` overrides `--c-e`/`--c-s` when specified.
