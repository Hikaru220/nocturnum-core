# Nocturnum changes to the Modrinth App core (theseus)

Base: modrinth/code commit f73010a (2026-11-09). GPL-3.0-only (packages/app-lib/COPYING.md).
Rule: every change is small, isolated and listed here so upstream fixes can be merged monthly.
Modrinth branding assets (.idea/icon.svg, .github/assets/) are removed, none is shipped.

| # | File | Change | Spec |
|---|------|--------|------|
| 1 | `packages/app-lib/src/api/instance/run.rs` | no `analytics/minecraft-server-play`, no `analytics/playtime` requests | 3.2 |
| 2 | `packages/app-lib/src/util/fetch.rs` | download analytics header is never sent | 3.2 |
| 3 | `packages/app-lib/src/state/settings.rs` | `telemetry` and `personalized_ads` always false | 3.2 |
| 4 | `packages/app-lib/src/state/db_backup.rs` | backups go to `Nocturnum/Backups` | 3.3 |
| 5 | `packages/app-lib/src/state/discord.rs` | own Discord application id, own assets, one button (t.me/Nocturnum_Launcher) | 3.3 |

Planned: Microsoft sign-in with the Nocturnum client id (3.1), data folder `%APPDATA%\Nocturnum` (3.3), game log reader (3.4), remove modules `mr_auth`, `users`, `friends`, `reports`, `upload`, `onboarding_checklist`, `tunnel` (2.2).
