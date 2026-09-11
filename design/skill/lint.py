#!/usr/bin/env python3
"""Structural lint for the design tree: form only. Its messages say what it checks."""
import argparse
import os
import re
import subprocess
import sys

DECISION_MARKERS = (" because ", " instead of ")
LOG_ENTRY = re.compile(r"^## \d{4}-\d{2}-\d{2} \S")
LOG_REQUIRED = ("Nodes:", "Summary:")
FORBIDDEN_HEADINGS = ("## Facts", "## Moves")
TREES = ("language", "compiler")
DATED_LINE = re.compile(r"^- 20\d\d-\d\d-\d\d")
REJECTED_ITEM = re.compile(r"^- (.+?): rejected because (\S.*)$")
FIELDS = ("Decision:", "Rejected:")


class Lint:
    def __init__(self, root):
        self.root = root
        self.errors = []
        self.nodes = {}  # node path (relative to root, no .md) -> lines
        self.decisions = 0
        self.rejected = 0

    def err(self, where, msg):
        self.errors.append(f"{where}: {msg}")

    # ---- discovery -----------------------------------------------------

    def discover(self):
        for tree in TREES:
            self.discover_tree(tree)

    def discover_tree(self, tree):
        tree_md = os.path.join(self.root, tree + ".md")
        tree_dir = os.path.join(self.root, tree)
        if not os.path.isfile(tree_md):
            self.err(tree_md, "missing root node")
            return
        self.nodes[tree] = self.read(tree_md)
        if not os.path.isdir(tree_dir):
            return
        for dirpath, dirnames, filenames in os.walk(tree_dir):
            rel_dir = os.path.relpath(dirpath, self.root)
            parent_md = os.path.join(self.root, rel_dir + ".md")
            if not os.path.isfile(parent_md):
                self.err(rel_dir, "directory has no sibling node file")
            for name in sorted(filenames):
                path = os.path.join(dirpath, name)
                rel = os.path.relpath(path, self.root)
                if not name.endswith(".md"):
                    self.err(rel, "only node files (.md) belong under a tree")
                    continue
                self.nodes[rel[:-3]] = self.read(path)
            dirnames.sort()

    def read(self, path):
        with open(path, "rb") as handle:
            data = handle.read()
        rel = os.path.relpath(path, self.root)
        for number, raw in enumerate(data.split(b"\n"), 1):
            if any(byte > 127 for byte in raw):
                self.err(f"{rel}:{number}", "non-ASCII text; the tree is English only")
        return data.decode("ascii", errors="replace").split("\n")

    # ---- node form -----------------------------------------------------

    def check_nodes(self):
        stems = {}
        for path in self.nodes:
            stem = path.rsplit("/", 1)[-1]
            stems.setdefault(stem, []).append(path)
        for stem, paths in stems.items():
            if len(paths) > 1:
                self.err(stem, "node name used more than once: " + ", ".join(paths))
        for path, lines in sorted(self.nodes.items()):
            self.check_node(path, lines, stems)

    def check_node(self, path, lines, stems):
        where = path + ".md"
        decisions = 0
        section = None
        rejected = []
        prev = "start"  # start | blank | field | item
        seen_field = False
        for number, line in enumerate(lines, 1):
            loc = f"{where}:{number}"
            if not line.strip():
                section = None
                prev = "blank"
                continue
            is_field = line.startswith(FIELDS)
            is_item = line.startswith("- ")
            if is_field:
                if prev not in ("blank", "start"):
                    self.err(loc, "a field must be separated from the previous line by a blank line")
                seen_field = True
                prev = "field"
            elif is_item:
                if prev not in ("field", "item"):
                    self.err(loc, "a list item must directly follow its header or the previous item")
                prev = "item"
            else:
                prev = "other"
            if line.startswith("Decision:"):
                decisions += 1
                low = line.lower()
                if not any(marker in low for marker in DECISION_MARKERS):
                    self.err(loc, "decision has neither 'because' nor 'instead of'")
                section = None
                continue
            if line.startswith("Rejected:"):
                section = "rejected"
                continue
            if line.startswith(FORBIDDEN_HEADINGS) or DATED_LINE.match(line):
                self.err(loc, "history belongs in git and the change log, not in a node")
                continue
            if line.startswith("- "):
                body = line[2:]
                if section == "rejected":
                    if not REJECTED_ITEM.match(line):
                        self.err(loc, "rejected entry needs '- <alternative>: rejected because <reason>'")
                    rejected.append(body)
                    continue
                self.err(loc, "list item outside Rejected:")
                continue
            self.err(loc, "line outside the node template")
        if decisions == 0:
            self.err(where, "node has no Decision: line")
        self.decisions += decisions
        self.rejected += len(rejected)

    # ---- log -----------------------------------------------------------

    def check_log(self):
        path = os.path.join(self.root, "log.md")
        if not os.path.isfile(path):
            self.err("log.md", "missing change log")
            return []
        lines = self.read(path)
        entries = []
        current = None
        for number, line in enumerate(lines, 1):
            if line.startswith("## "):
                if not LOG_ENTRY.match(line):
                    self.err(f"log.md:{number}", "entry heading must be '## YYYY-MM-DD <title>'")
                current = {"line": number, "fields": {}}
                entries.append(current)
                continue
            if current is None:
                continue
            for field in LOG_REQUIRED:
                if line.startswith(field):
                    current["fields"][field] = line[len(field):].strip()
        for entry in entries:
            loc = f"log.md:{entry['line']}"
            for field in LOG_REQUIRED:
                if field not in entry["fields"] or not entry["fields"][field]:
                    self.err(loc, f"entry lacks {field}")
        return entries

    def check_diff(self, base):
        probe = subprocess.run(["git", "rev-parse", "--verify", "--quiet", base],
                               capture_output=True, text=True)
        if probe.returncode != 0:
            print(f"notice: base {base!r} not found; skipping the log-per-change check")
            return
        names = subprocess.run(["git", "diff", "--name-only", base, "--", self.root],
                               capture_output=True, text=True, check=True).stdout.split()
        prefix = self.root.rstrip("/") + "/"
        changed = []
        for name in names:
            rel = name[len(prefix):] if name.startswith(prefix) else name
            if any(rel == tree + ".md" or rel.startswith(tree + "/") for tree in TREES):
                if rel.endswith(".md"):
                    changed.append(rel[:-3])
        if not changed:
            return
        log_rel = prefix + "log.md"
        if log_rel not in names:
            self.err("log.md", "tree changed since base but the change log did not")
            return
        diff = subprocess.run(["git", "diff", base, "--", log_rel],
                              capture_output=True, text=True, check=True).stdout
        added = "\n".join(line[1:] for line in diff.split("\n")
                          if line.startswith("+") and not line.startswith("+++"))
        for node in changed:
            if node not in added:
                self.err("log.md", f"changed node {node} is not named in a new log entry")

    # ---- metrics -------------------------------------------------------

    def metrics(self):
        depth = max((path.count("/") for path in self.nodes), default=0)
        per_subtree = {}
        for path in self.nodes:
            parts = path.split("/")
            key = parts[0] if len(parts) == 1 else "/".join(parts[:2])
            per_subtree[key] = per_subtree.get(key, 0) + 1
        print(f"nodes: {len(self.nodes)}  depth: {depth}  decisions: {self.decisions}  rejected: {self.rejected}")
        for name, count in sorted(per_subtree.items()):
            print(f"  {name}: {count}")


def main():
    parser = argparse.ArgumentParser(description="structural lint for the design tree")
    parser.add_argument("--root", default="design")
    parser.add_argument("--base", default=None)
    args = parser.parse_args()
    lint = Lint(args.root)
    lint.discover()
    lint.check_nodes()
    lint.check_log()
    if args.base:
        lint.check_diff(args.base)
    if lint.errors:
        for error in lint.errors:
            print("error: " + error, file=sys.stderr)
        print(f"design lint: {len(lint.errors)} error(s)", file=sys.stderr)
        return 1
    lint.metrics()
    print("design lint: ok")
    return 0


if __name__ == "__main__":
    sys.exit(main())
