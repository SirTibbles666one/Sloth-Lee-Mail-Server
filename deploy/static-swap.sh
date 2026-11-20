#!/usr/bin/env bash
# Replace the served client without ever leaving the tree empty.
#
# The old swap was `rm -rf $STATIC/* && tar x -C $STATIC`, which serves 404 for the length of the
# extraction: measured at 10 failures in 517 polls over ~1.5 s. A tab that reloaded, or that
# fetched a lazy chunk, in that window showed a half-built page.
#
# This writes the new files in beside the old ones, assets first and index.html last, so every
# chunk a page can reference is already on disk. Old chunks are retired only once they are too old
# to belong to a page still in memory.
set -euo pipefail
STATIC=/opt/on/data/static
stage=$(mktemp -d /opt/on/data/static.stage.XXXXXX)
tar xzf - -C "$stage"
[ -f "$stage/index.html" ] || { echo "no index.html in payload" >&2; exit 1; }
mkdir -p "$STATIC/assets"
if [ -d "$stage/assets" ]; then cp -a "$stage/assets/." "$STATIC/assets/"; fi
cp -a "$stage/." "$STATIC/"
cp -a "$stage/index.html" "$STATIC/index.html"
rm -rf "$stage"
find "$STATIC/assets" -type f -mtime +3 -delete 2>/dev/null || true
echo uploaded
