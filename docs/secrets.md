# API keys

Where provider API keys come from, how they are stored, and how to inspect them. Linux
platform setup (including sessions without a Secret Service) is in [[linux]].

## Resolution order

`providers/mod.rs::api_key` resolves a key for a provider in two steps:

1. **`settings.json` override.** If `providers.<name>.apiKey` is set, it must be an `env:VAR` reference (for example `env:OPENROUTER_API_KEY`); the value is read from the environment at use time. A raw key in `settings.json` is rejected with an error — secrets never live in the config file.
2. **OS keychain.** Otherwise `secrets::get(provider)` reads the keychain entry.

Web-search providers go through `providers/mod.rs::search_key`, which calls `api_key` first and then falls back to the conventional `<PROVIDER>_API_KEY` environment variable (for example `EXA_API_KEY`, `SERPER_API_KEY`).

`commands.rs::has_api_key` reports whether a key resolves; `commands.rs::store_api_key` writes to the keychain; `commands.rs::delete_api_key` removes it.

## Keychain entries

`secrets.rs` uses `keyring` v4 with a fixed service name:

- **Service:** `SlopChat`
- **Account:** the provider name (`openrouter`, `gemini`, `exa`, …)
- **Label:** `keyring:<account>@SlopChat`

## Linux

keyring's default `v1` feature selects `zbus-secret-service-keyring-store` on Linux, so keys are stored through the freedesktop **Secret Service** over the session D-Bus (`org.freedesktop.secrets`), using `crypto-rust` for the encrypted session. `linux-keyutils-keyring-store` is only pulled in by keyring's `cli` feature, which is not enabled, so there is **no keyutils/kernel-keyring fallback**.

The Secret Service is normally provided by GNOME Keyring, KDE KWallet, or KeePassXC. A bare Hyprland (or other minimal WM) session usually runs none of them; without a provider:

- `store_api_key` returns an error, and
- `has_api_key` is false until a key is supplied some other way.

In that case set an `env:VAR` override in `settings.json` and export the variable from the session:

```json
{ "providers": { "openrouter": { "baseUrl": "https://openrouter.ai/api/v1", "apiKey": "env:OPENROUTER_API_KEY" } } }
```

Search providers additionally accept `<PROVIDER>_API_KEY` without any settings change. Under Snap/Flatpak, the sandbox needs access to `org.freedesktop.secrets`.

Inspect or remove a stored key with libsecret's tool:

```sh
secret-tool lookup service SlopChat username openrouter
secret-tool clear service SlopChat username openrouter
```

## Other platforms

- **macOS:** `apple-native-keyring-store` → login Keychain. Unsigned builds prompt on every access; see the README's "macOS keychain prompts".
- **Windows:** `windows-native-keyring-store` → Credential Manager.

## Adding a key

Onboarding (first launch) and **Settings → Web search** call `store_api_key`. Alternatively, export the variable and reference it with `env:` as above — nothing is written to disk.
