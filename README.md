# db-backup-worker

This repo is a [Cloudflare Worker](https://developers.cloudflare.com/workers/)
for backing up the [Ace Archive artifact
database](https://github.com/acearchive/submission-worker?tab=readme-ov-file#database)
to [Cloudflare R2](https://developers.cloudflare.com/r2/).

It runs on a schedule once daily at 00:00 UTC.
