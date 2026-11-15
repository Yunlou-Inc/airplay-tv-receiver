#!/usr/bin/env bash
set -euo pipefail

command -v git >/dev/null || { echo "git is required" >&2; exit 1; }
git rev-parse --is-inside-work-tree >/dev/null
[[ -z "$(git status --porcelain)" ]] || { echo "Refusing to archive a dirty working tree." >&2; exit 1; }

commit="$(git rev-parse HEAD)"
stamp="$(git show -s --format=%cs "$commit")"
name="airplay-tv-receiver-${stamp}-${commit:0:12}"
out="${1:-dist}"
mkdir -p "$out"
git archive --format=tar --prefix="$name/" "$commit" | gzip -n > "$out/$name.tar.gz"
(
  cd "$out"
  sha256sum "$name.tar.gz" > "$name.tar.gz.sha256"
)
printf 'Archived commit %s to %s/%s.tar.gz\n' "$commit" "$out" "$name"
