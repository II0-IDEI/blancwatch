# Blancwatch personal v1

Blancwatch adapts Clash Verge Rev v2.5.7 for a local BlancVPN VLESS subscription.
It retains the upstream client, Mihomo, Windows service and GPL-3.0 license.
The scope comes from [the reuse proposal](https://github.com/II0-IDEI/blanc-vpn-watchdog/issues/2#issuecomment-5984025360).

## Windows setup

1. Install the Windows x64 Blancwatch installer. Windows asks for administrator
   consent because TUN uses the existing service. Keep HAPP installed for rollback.
2. Import the subscription URL on Profiles and select the imported profile.
   Its title, website, support link and one-hour refresh are preserved.
3. Keep Rule mode. Select **Blancwatch Auto** for automatic fallback, or choose a
   node manually in the Blancwatch group.
4. Add application exceptions in the profile's rules editor, before the catch-all
   rule. The existing editor accepts `PROCESS-NAME,example.exe,DIRECT` and
   `PROCESS-PATH,C:\path\example.exe,DIRECT`. Keep these private local settings.
5. Disconnect HAPP before enabling Blancwatch TUN. Use the TUN switch for On/Off;
   leave System Proxy disabled for this all-traffic setup. Enable launch after
   login in Settings if wanted.

Off applied to a running core disables routing through Blancwatch and allows
ordinary internet; automatic recovery retains that applied Off setting. If the
core is already stopped, Off is saved locally without starting it. An unreachable
Windows service may independently restore its last applied TUN setting; restart
Blancwatch after the service returns to apply the saved Off. Closing the window
keeps the app in the tray; Quit stops the running client. The TUN status reports
the core's observed setting, rather than treating a saved On request as a
successful connection.

The personal installation uses `%APPDATA%\io.github.ii0-idei.blancwatch` and its
own Blancwatch startup task. Its Windows service namespace is shared with Clash
Verge Rev. Remove the other client's service before installing Blancwatch.
Keep that namespace reserved to Blancwatch until its uninstall; installing
another Clash service over it is unsupported. Portable copies can use sidecar
mode but do not own the installed service.

The subscription URL and node credentials are stored locally in readable form.

## Refresh and recovery

The local converter supports TCP/REALITY/Vision and ordinary WebSocket/TLS
VLESS feeds, in raw or Base64 form. Existing Clash YAML remains supported.
Unsupported or malformed VLESS refreshes leave the previous profile available.
No third-party subscription converter is used.

Scheduled refresh uses the configured interval; a failed scheduled update waits
that interval before trying again. Each update has three fetch strategies, with
a default 20-second request timeout. Direct fetching can bootstrap a refresh
under the approved direct-internet failure policy.

Automatic node fallback uses five-minute health checks with five-second timeouts.
The stock service limits core restart attempts to ten per ten-minute window,
with backoff capped at 30 seconds. Windows' separate whole-service recovery
continues its final 30-second restart action. Neither a running core nor a TUN
indicator alone proves that a particular provider endpoint forwards traffic.

## Build, update and rollback

Use the Rust toolchain in `rust-toolchain.toml`, Node.js, pnpm, MSVC C++ build
tools and the Windows SDK. Follow [CONTRIBUTING.md](../CONTRIBUTING.md) for the
upstream prerequisites, then run from the repository root:

```powershell
pnpm install --frozen-lockfile
pnpm prebuild x86_64-pc-windows-msvc
pnpm build:fast
```

The NSIS installer is produced under `target/fast-release/bundle/nsis`.
Resource preparation pins Mihomo v1.19.32, service v2.7.6 and the Windows x64
resource hashes in `scripts/resources.mjs`. Both core choices contain the tested
stable core. WebView2's Microsoft bootstrapper and installed runtime remain
evergreen.

Application updates are manual; upstream application updates are disabled.
Mihomo core updates also require a Blancwatch installer; the in-app core updater is disabled.
Keep the previous installer and back up the Blancwatch data directory before an
update. For rollback, turn TUN and System Proxy Off, quit Blancwatch, then
reconnect HAPP. Reinstall the previous Blancwatch build when needed. Removing
Blancwatch does not require removing HAPP.
