# db-backup-worker

This repo is a [Cloudflare Worker](https://developers.cloudflare.com/workers/)
for backing up the [Ace Archive artifact
database](https://github.com/acearchive/submission-worker?tab=readme-ov-file#database)
to [Cloudflare R2](https://developers.cloudflare.com/r2/).

It runs on a schedule once daily at 00:00 UTC.

Deploying this worker requires setting two secrets with
[wrangler](https://developers.cloudflare.com/workers/configuration/secrets/):

- `ACCOUNT_ID`: The Cloudflare account ID.
- `API_TOKEN`: A Cloudflare API token with Edit access to Cloudflare D1, which
  is necessary to request a database export.
