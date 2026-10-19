#!/usr/bin/env bash
set -euo pipefail

root="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/../.." && pwd)"
dest="$root/build/nightrun-fonts"
font="$dest/NotoSansJP-wght.ttf"
license="$dest/OFL.txt"
font_sha="c2f3b4d463500a2ddcd3849cded1fceeb9fd6d1c32e6cbecd568453ba50fc68f"
license_sha="1c05c68c34f9708415aada51f17e1b0092d2cea709bf4a94cd38114f9e73d7d9"
revision="66a36c8c94b1a5d992ee4e7f392fccfe4945767c"
base="https://raw.githubusercontent.com/google/fonts/$revision/ofl/notosansjp"

mkdir -p -- "$dest"
rm -f -- "$dest/.complete"

fetch_verified() {
    local url="$1" output="$2" expected="$3" part="${2}.part"
    curl --fail --location --retry 3 --silent --show-error "$url" -o "$part"
    printf '%s  %s\n' "$expected" "$part" | sha256sum --check --status
    mv -f -- "$part" "$output"
}

if [[ ! -f "$font" ]] || ! printf '%s  %s\n' "$font_sha" "$font" | sha256sum --check --status; then
    fetch_verified "$base/NotoSansJP%5Bwght%5D.ttf" "$font" "$font_sha"
fi
if [[ ! -f "$license" ]] || ! printf '%s  %s\n' "$license_sha" "$license" | sha256sum --check --status; then
    fetch_verified "$base/OFL.txt" "$license" "$license_sha"
fi

python3 "$root/scripts/installer/fonts.py" "$font" "$dest"
touch "$dest/.complete"