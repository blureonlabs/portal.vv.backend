# Transfer notes: blureonlabs/portal.vv.backend

## What happened on 13 Nov 2026 (laptop transfer)
The founder moved to a new laptop and cleaned up the old one. For this repo:
- Already fully pushed. Old local path: `~/Documents/Projects/portal.vv/backend`. The sibling repo is `blureonlabs/portal.vv.frontend`.
- The local copy on the old laptop was **deleted** after everything was confirmed on GitHub. It now lives **only on GitHub**.
- Build folders (`node_modules`, `.next`, `target`, `dist`) were never in git. They come back with `npm install`, `cargo build` and the like.

## How to pick this up again
```bash
git clone https://github.com/blureonlabs/portal.vv.backend.git
cd portal.vv.backend
# then install dependencies (npm install / pnpm install / cargo build) and add your .env if the project needs one
```

*Note written by Claude (Claude Code) for the founder, 13 Nov 2026. Master index of every repo moved that day: `blureonlabs/blubabyshow` → `docs/REPO_TRANSFER_INDEX.md`.*
