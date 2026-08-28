#!/bin/sh
# Assemble a standalone explain-code page.
#
#   ./assemble.sh "Title of the page" body.html out.html
#   ./assemble.sh body.html out.html          # title taken from the body's <h1>
#
# `body.html` is a fragment: the <header class="mast">, the <section>s, and the
# <footer class="close"> -- no doctype, no <head>, no <style>, no <script>.
# Everything else, including the house style and the review layer, is added here.
#
# Publishing through a host that supplies its own page shell (Claude Code's
# Artifact tool) is the other mode: pass the same body fragment plus a <title>,
# an inline <style> holding house-style.css, and an inline <script> holding
# comments.js -- but no doctype/html/head/body wrapper, since the host adds one.
set -eu
case $# in
  2) body=$1; out=$2; raw=$(sed -n 's/.*<h1[^>]*>\(.*\)<\/h1>.*/\1/p' "$body" | head -1) ;;
  3) raw=$1; body=$2; out=$3 ;;
  *) echo "usage: $0 [title] <body.html> <out.html>" >&2; exit 2 ;;
esac
[ -n "$raw" ] || { echo "$0: no title given and no <h1> found in $body" >&2; exit 2; }

# A body fragment only. Passing an already-assembled page would nest a doctype.
if grep -qi '<!doctype\|<html' "$body"; then
  echo "$0: $body looks like a full page, not a body fragment" >&2; exit 2
fi

d=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
title=$(printf '%s' "$raw" | sed 's/&/\&amp;/g; s/</\&lt;/g; s/>/\&gt;/g')

if ! : > "$out"; then
  echo "$0: could not write $out" >&2
  exit 1
fi

{
  printf '<!doctype html>\n<html lang="en">\n<head>\n'
  printf '<meta charset="utf-8">\n'
  printf '<meta name="viewport" content="width=device-width, initial-scale=1">\n'
  printf '<meta name="color-scheme" content="light dark">\n'
  printf '<title>%s</title>\n<style>\n' "$title"
  cat "$d/house-style.css"
  printf '</style>\n</head>\n<body>\n'
  cat "$body"
  printf '\n<script>\n'
  cat "$d/comments.js"
  printf '</script>\n</body>\n</html>\n'
} > "$out"

echo "wrote $out"
