# Database setup (Postgres + sqlx)

This project uses Postgres for the `ub_security` replay protection store. The migrations are under `infra/migrations/ub_security/`.

Quick local setup:

1. Install Postgres and create a database, e.g. `aln_dev`.
2. Set `DATABASE_URL` in your shell (do not commit credentials):

```bash
export DATABASE_URL=postgres://user:password@localhost:5432/aln_dev
```

3. Install `sqlx-cli` (recommended) and run migrations from the repo root:

```bash
cargo install sqlx-cli --no-default-features --features postgres
sqlx migrate run --source infra/migrations/ub_security
```

Notes:
- Migrations in `infra/migrations/ub_security/` include `20251220_init_ub_security.up.sql` and the corresponding `.down.sql`.
- Never commit connection strings or secrets to the repo; use env vars or external secret managers in CI/production.
- For CI, ensure `DATABASE_URL` is set in the runner environment via your secret manager and the migration step runs before services that need the DB.
