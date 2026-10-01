#!/usr/bin/env sh
set -eu

# Build-from-source installer for SlopChat.
# macOS: signs (when possible) and copies the app to /Applications.

need() {
  command -v "$1" >/dev/null 2>&1 || {
    echo "error: $1 is required but not installed" >&2
    exit 1
  }
}

need node
need npm
need cargo

echo "==> Installing dependencies"
npm install

# A stable code-signing identity stops macOS from re-prompting for keychain
# access after every rebuild.
if [ "$(uname -s)" = "Darwin" ]; then
  CANDIDATE="${APPLE_SIGNING_IDENTITY:-${SLOPCHAT_SIGNING_IDENTITY:-}}"
  # `security` may print a trailing status; strip it.
  CANDIDATE="${CANDIDATE%% (CSSMERR*}"
  IDENTITY=""
  if [ -n "$CANDIDATE" ] \
    && security find-identity -v -p codesigning 2>/dev/null | grep -Fq "\"$CANDIDATE\""; then
    IDENTITY="$CANDIDATE"
  elif [ -z "$CANDIDATE" ] && command -v security >/dev/null 2>&1; then
    IDENTITY=$(security find-identity -v -p codesigning 2>/dev/null \
      | grep -m1 '"SlopChat Local Code Signing"' \
      | sed -E 's/.*"(.*)"/\1/') || true
  fi

  if [ -n "$IDENTITY" ]; then
    export APPLE_SIGNING_IDENTITY="$IDENTITY"
    echo "==> Signing as: $IDENTITY"
  elif security find-identity -p codesigning 2>/dev/null | grep -q '"SlopChat Local Code Signing"'; then
    cat >&2 <<'WARN'
====================================================================
 WARNING: "SlopChat Local Code Signing" exists but is NOT TRUSTED,
 so codesign cannot use it. Trust it once, then re-run:

   Keychain Access -> login -> Certificates -> "SlopChat Local Code
   Signing" -> double-click -> expand Trust -> set "When using this
   certificate" to "Always Trust" -> close (enter your password).

 Or in a terminal:
   security find-certificate -c "SlopChat Local Code Signing" -p \
     > ~/slopchat-cert.pem
   security add-trusted-cert -r trustRoot -p codeSign \
     -k ~/Library/Keychains/login.keychain-db ~/slopchat-cert.pem

 Building unsigned for now, so macOS will keep asking for keychain
 access on every chat send, Settings open, and rebuild.
====================================================================
WARN
  else
    cat >&2 <<'WARN'
====================================================================
 WARNING: no "SlopChat Local Code Signing" identity found.

 Building unsigned. macOS will keep asking for keychain access on
 every chat send, Settings open, and rebuild.

 One-time fix: Keychain Access -> Certificate Assistant -> Create a
 Certificate.
   Name: SlopChat Local Code Signing
   Type: Code Signing
 Then trust it (double-click -> Trust -> "Always Trust") and re-run.
====================================================================
WARN
  fi
fi

echo "==> Building (this can take a while)"
npm run tauri build

if [ "$(uname -s)" = "Darwin" ]; then
  APP="src-tauri/target/release/bundle/macos/SlopChat.app"
  if [ ! -d "$APP" ]; then
    echo "error: bundle not found at $APP" >&2
    exit 1
  fi
  echo "==> Installing to /Applications"
  rm -rf "/Applications/SlopChat.app"
  cp -R "$APP" /Applications/
  echo "Done. Launch SlopChat from /Applications."
else
    echo "Only macOS is supported currently"
fi
