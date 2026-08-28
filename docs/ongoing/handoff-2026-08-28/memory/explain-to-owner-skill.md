---
name: explain-to-owner-skill
description: Owner-approved voice lives in the explain-code skill (7 rules + sample + Chinese model); invoke it for EVERY owner-facing reply; never create a parallel skill again — supersede in place.
metadata: 
  node_type: memory
  type: feedback
  originSessionId: 164517a9-4a0a-49bb-8ff7-44ea6270516e
  modified: 2026-08-24T06:37:53.499Z
---

The owner's communication method lives in the `explain-code` skill (source of truth: github.com/mbbill/explain-code-skills, commit c9aa0f0; installed in `~/.claude/skills/` and `~/.codex/skills/`). Invoke it in EVERY conversation turn addressed to the owner: answers, status, analyses, reviews, not only explanations. Style is fixed by `references/sample.md` (owner-approved): question opening; one baseline example mutated one change at a time; explanation inside code blocks as inline comments; zero internal jargon; popular-science Chinese per `references/chinese-writing-model.md`; verified-vs-reported marked in place; bans on 破折号（——）、"一句话/简单来说/换句话说"、自造缩写.

**Rule added 2026-08-24 (owner):** when an explanation touches real variables, functions, or logic, quote the ORIGINAL source with the file named (line numbers when useful); elisions allowed and marked, but every kept line and identifier must match the source byte-for-byte so any name in the text is findable by search. Invented illustrative blocks only where no source exists, labeled as such. Reason: the owner kept having to grep paraphrased names against the code.

**Hard lesson (2026-08-24, owner correction):** I created `explain-to-owner` as a sibling skill instead of updating `explain-code`; the owner had to delete the parallel installs themselves and said 后面不要再这样了. Supersede in place applies to skills and user-level assets exactly as it does to the repo: when an existing skill owns the space, update it; never ship a parallel twin. `write-chinese-tech-articles` and `write-raschka-style` remain disabled (`skills-disabled/` in both agents; sources in ~/code/tech-writing-skills) — do not re-enable; explain-code bundles what it needs. Related: [[explain-with-code-first]], [[agent-reports-are-claims]].
