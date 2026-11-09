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
| 6 | `state/nocturnum_auth.rs` (new), `state/minecraft_auth.rs`, `api/minecraft_auth.rs` | sign-in with Nocturnum client id, device code flow, XSTS/Mojang errors as codes, silent refresh; SISU flow of the official launcher no longer used | 3.1 |

Planned: data folder `%APPDATA%\Nocturnum` (3.3), game log reader (3.4), remove modules `mr_auth`, `users`, `friends`, `reports`, `upload`, `onboarding_checklist`, `tunnel` (2.2).

| 7 | `packages/app-lib/src/state/mod.rs` | no Modrinth account refresh, no friends websocket at startup | 2.2 |

Data folder: the app identifier passed to `State::init("Nocturnum")` gives the Nocturnum folder under the user data dir (no core change).
Open question: `MODRINTH_LAUNCHER_META_URL` (launcher-meta.modrinth.com) is still used for version lists; clarify with Modrinth or replace with Mojang/Fabric meta (spec 3.3).

| 8 | `packages/app-lib/src/api/instance/run.rs`, `api/instance.rs` | new `run_with_account` (launch with a chosen account, several clients at once) | 4 |

| 9 | `packages/app-lib/src/lib.rs` | User-Agent `Nocturnum/0.1 (os)` instead of the Modrinth one | 3.3 |
| 10 | `packages/app-lib/src/api/minecraft_auth.rs` | `add_test_account` (debug builds only, for the automatic launch test) | tests |
