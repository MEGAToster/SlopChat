#!/usr/bin/env sh
set -eu

# Build-from-source installer for SlopChat (macOS).
# Signs with "SlopChat Local Code Signing" (see tauri.conf.json) and copies
# the app to /Applications.

need() {
  command -v "$1" >/dev/null 2>&1 || {
    echo "error: $1 is required but not installed" >&2
    exit 1
  }
}

if [ "$(uname -s)" != "Darwin" ]; then
  echo "error: only macOS is supported right now" >&2
  exit 1
fi

need node
need npm
need cargo

echo "==> Installing dependencies"
npm install

# Signing is configured in tauri.conf.json; the certificate must exist AND be
# trusted, otherwise codesign fails. Prefer an explicit override, else the
# default identity.
CANDIDATE="${APPLE_SIGNING_IDENTITY:-${SLOPCHAT_SIGNING_IDENTITY:-}}"
# `security` may print a trailing status; strip it.
CANDIDATE="${CANDIDATE%% (CSSMERR*}"
IDENTITY=""
if [ -n "$CANDIDATE" ] \
  && security find-identity -v -p codesigning 2>/dev/null | grep -Fq "\"$CANDIDATE\""; then
  IDENTITY="$CANDIDATE"
else
  IDENTITY=$(security find-identity -v -p codesigning 2>/dev/null \
    | grep -m1 '"SlopChat Local Code Signing"' \
    | sed -E 's/.*"(.*)"/\1/') || true
fi

if [ -z "$IDENTITY" ]; then
  if security find-identity -p codesigning 2>/dev/null | grep -q '"SlopChat Local Code Signing"'; then
    cat >&2 <<'WARN'
====================================================================
 ERROR: "SlopChat Local Code Signing" exists but is NOT TRUSTED, so
 codesign cannot use it. Trust it once, then re-run:

   Keychain Access -> login -> Certificates -> "SlopChat Local Code
   Signing" -> double-click -> expand Trust -> set "When using this
   certificate" to "Always Trust" -> close (enter your password).

 Or in a terminal:
   security find-certificate -c "SlopChat Local Code Signing" -p \
     > ~/slopchat-cert.pem
   security add-trusted-cert -r trustRoot -p codeSign \
     -k ~/Library/Keychains/login.keychain-db ~/slopchat-cert.pem
====================================================================
WARN
  else
    cat >&2 <<'WARN'
====================================================================
 ERROR: no "SlopChat Local Code Signing" identity found.

 Create one and TRUST it (creating alone is not enough):

   Keychain Access -> Certificate Assistant -> Create a Certificate.
     Name: SlopChat Local Code Signing
     Identity Type: Self Signed Root; Certificate Type: Code Signing
   Then double-click it -> Trust -> set "When using this certificate"
   to "Always Trust" -> close (enter your password).

 See README -> "macOS keychain prompts".
====================================================================
WARN
  fi
  exit 1
fi

export APPLE_SIGNING_IDENTITY="$IDENTITY"
echo "==> Signing as: $IDENTITY"

echo "==> Building (this can take a while)"
npm run tauri -- build --bundles app

APP="src-tauri/target/release/bundle/macos/SlopChat.app"
if [ ! -d "$APP" ]; then
  echo "error: bundle not found at $APP" >&2
  exit 1
fi

echo "==> Installing to /Applications"
rm -rf "/Applications/SlopChat.app"
cp -R "$APP" /Applications/
echo "Done. Launch SlopChat from /Applications."
