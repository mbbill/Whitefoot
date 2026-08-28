/* ============================================================================
   explain-code review layer
   ----------------------------------------------------------------------------
   Lets the reader attach notes and questions to individual passages, survive a
   reload, and export the batch as markdown to paste back into chat.

   Declares no artifact capabilities and makes no network requests, so the same
   file behaves identically opened locally and published. There is no runtime
   capability that can post back into a conversation -- clipboard is the path.

   Inline verbatim in a <script> block at the end of the page. Requires the
   matching styles in house-style.css.
   ========================================================================= */
(function () {
  "use strict";

  var STORE = "ec-comments:" + (document.title || "page") + "|" + location.pathname;
  var items = [];

  try {
    items = JSON.parse(localStorage.getItem(STORE) || "[]");
  } catch (e) {
    items = [];
  }

  function persist() {
    try {
      localStorage.setItem(STORE, JSON.stringify(items));
    } catch (e) {
      /* private mode: comments still work for this session */
    }
  }

  function slug(s) {
    return String(s).toLowerCase().replace(/[^a-z0-9]+/g, "-").replace(/^-|-$/g, "").slice(0, 60);
  }

  function clamp(t, n) {
    if (t.length <= n) return t;
    return t.slice(0, n).replace(/\s+\S*$/, "") + "\u2026";
  }

  /* The .num marker must come off before reading a heading, or textContent
     concatenates it: "1Where the slot is allocated". */
  function headingText(h) {
    var c = h.cloneNode(true);
    var n = c.querySelector(".num");
    if (n) n.remove();
    return c.textContent.replace(/\s+/g, " ").trim();
  }

  function passageText(block) {
    var c = block.cloneNode(true);
    c.querySelectorAll(".ec-pin, .ec-ui").forEach(function (n) { n.remove(); });
    return c.textContent.replace(/\s+/g, " ").trim();
  }

  function scopeFor(block) {
    var sec = block.closest("section");
    var sh = sec && sec.querySelector("h2");
    return sh ? headingText(sh) : "Introduction";
  }

  function labelFor(block, index) {
    var text = clamp(passageText(block), 92);
    return scopeFor(block) + (text ? " \u2014 \u201c" + text + "\u201d" : " \u2014 passage " + (index + 1));
  }

  var blocks = [].slice.call(document.querySelectorAll(
    ".mast > .standfirst, section .col > p, section .col > pre, " +
    "section .col > ul.prose > li, section .col > ol.prose > li, " +
    "section > .checkpoint, figure > figcaption"
  )).filter(function (b) { return !b.closest(".ec-ui, .ec-dock, .ec-note"); });

  var scopeCounts = Object.create(null);
  var usedIds = Object.create(null);

  blocks.forEach(function (block, i) {
    var label = labelFor(block, i);
    var scope = slug(scopeFor(block)) || "page";
    scopeCounts[scope] = (scopeCounts[scope] || 0) + 1;
    var explicitId = block.getAttribute("data-ec-id");
    var id = explicitId || scope + "-passage-" + scopeCounts[scope];
    if (usedIds[id]) id = id + "-" + (i + 1);
    usedIds[id] = 1;
    block.dataset.ecId = id;
    block.dataset.ecLabel = label;

    block.classList.add("ec-commentable");

    var add = document.createElement("button");
    add.type = "button";
    add.className = "ec-pin";
    add.innerHTML = '<svg viewBox="0 0 24 24" aria-hidden="true" focusable="false"><path d="M5 5.5h14v10H9l-4 3v-13Z"></path><path d="M9 10.5h6"></path></svg>';
    add.setAttribute("aria-label", "Comment on: " + clamp(passageText(block), 120));
    add.addEventListener("click", function () {
      var sel = lastSel.node && block.contains(lastSel.node) ? lastSel.text : "";
      openComposer(block, clamp(sel, 400));
    });
    block.insertBefore(add, block.firstChild);

    var ui = document.createElement("div");
    ui.className = "ec-ui";
    block._ecUi = ui;
    if (block.tagName === "LI") block.appendChild(ui);
    else block.insertAdjacentElement("afterend", ui);
  });

  var orphans = document.createElement("div");
  orphans.className = "ec-orphans";
  orphans.hidden = true;

  var lastSel = { text: "", node: null };
  document.addEventListener("mouseup", captureSelection);
  document.addEventListener("keyup", captureSelection);
  document.addEventListener("mousedown", function (ev) {
    /* The press that collapses a selection also invalidates the memory of it --
       except on the review UI itself, whose button press must not eat the quote. */
    if (!ev.target.closest(".ec-ui, .ec-pin")) lastSel = { text: "", node: null };
  });

  function captureSelection() {
    var s = window.getSelection && window.getSelection();
    if (!s || s.isCollapsed) return;
    var t = String(s).trim();
    if (!t) return;
    var n = s.anchorNode;
    lastSel = { text: t, node: n && n.nodeType === 1 ? n : n && n.parentNode };
  }

  function openComposer(block, quote, editing) {
    var existing = block._ecUi.querySelector(".ec-composer");
    if (existing) existing.remove();

    var box = document.createElement("form");
    box.className = "ec-composer";

    var kinds = document.createElement("div");
    kinds.className = "ec-kinds";
    ["note", "question"].forEach(function (k) {
      var b = document.createElement("button");
      b.type = "button";
      b.className = "ec-kind" + ((editing ? editing.kind : "note") === k ? " on" : "");
      b.dataset.kind = k;
      b.textContent = k;
      b.addEventListener("click", function () {
        kinds.querySelectorAll(".ec-kind").forEach(function (x) { x.classList.remove("on"); });
        b.classList.add("on");
      });
      kinds.appendChild(b);
    });

    var q = editing ? editing.quote : quote;
    if (q) {
      var qe = document.createElement("blockquote");
      qe.className = "ec-quote";
      qe.textContent = q;
      box.appendChild(qe);
    }

    var ta = document.createElement("textarea");
    ta.rows = 3;
    ta.placeholder = "Note or question…  (⌘↵ to save)";
    ta.value = editing ? editing.body : "";

    var row = document.createElement("div");
    row.className = "ec-row";
    var save = document.createElement("button");
    save.type = "submit";
    save.className = "ec-save";
    save.textContent = editing ? "Update" : "Save";
    var cancel = document.createElement("button");
    cancel.type = "button";
    cancel.className = "ec-cancel";
    cancel.textContent = "Cancel";
    cancel.addEventListener("click", function () { box.remove(); });
    row.appendChild(save);
    row.appendChild(cancel);

    box.appendChild(kinds);
    box.appendChild(ta);
    box.appendChild(row);

    box.addEventListener("submit", function (ev) {
      ev.preventDefault();
      var body = ta.value.trim();
      if (!body) { box.remove(); return; }
      var kind = (kinds.querySelector(".ec-kind.on") || {}).dataset.kind || "note";
      if (editing) {
        editing.body = body;
        editing.kind = kind;
      } else {
        items.push({
          id: "c" + Date.now() + Math.random().toString(36).slice(2, 6),
          block: block.dataset.ecId,
          label: block.dataset.ecLabel,
          quote: q || "",
          body: body,
          kind: kind
        });
      }
      persist();
      box.remove();
      render();
    });

    ta.addEventListener("keydown", function (ev) {
      if ((ev.metaKey || ev.ctrlKey) && ev.key === "Enter") save.click();
      if (ev.key === "Escape") box.remove();
    });

    block._ecUi.appendChild(box);
    var r = box.getBoundingClientRect();
    if (r.top < 0 || r.bottom > window.innerHeight - 110) {
      box.scrollIntoView({ block: "center" });
    }
    ta.focus();
  }

  /* `block` is null for an orphan: its anchor is gone, so it can be read and
     deleted but not edited back onto a block that no longer exists. */
  function makeCard(c, block) {
    var card = document.createElement("div");
    card.className = "ec-note ec-kind-" + c.kind;

    var head = document.createElement("div");
    head.className = "ec-head";
    var tag = document.createElement("span");
    tag.className = "ec-tag";
    tag.textContent = block ? c.kind : c.kind + " \u00b7 orphaned";
    head.appendChild(tag);

    var tools = document.createElement("span");
    tools.className = "ec-tools";
    if (block) {
      var ed = document.createElement("button");
      ed.type = "button";
      ed.textContent = "edit";
      ed.addEventListener("click", function () { openComposer(block, c.quote, c); });
      tools.appendChild(ed);
    }
    var del = document.createElement("button");
    del.type = "button";
    del.textContent = "delete";
    del.addEventListener("click", function () {
      items = items.filter(function (x) { return x.id !== c.id; });
      persist();
      render();
    });
    tools.appendChild(del);
    head.appendChild(tools);
    card.appendChild(head);

    if (!block) {
      var was = document.createElement("div");
      was.className = "ec-was";
      was.textContent = "was: " + c.label;
      card.appendChild(was);
    }
    if (c.quote) {
      var qq = document.createElement("blockquote");
      qq.className = "ec-quote";
      qq.textContent = c.quote;
      card.appendChild(qq);
    }
    var body = document.createElement("p");
    body.className = "ec-body";
    body.textContent = c.body;
    card.appendChild(body);
    return card;
  }

  function render() {
    document.querySelectorAll(".ec-note").forEach(function (n) { n.remove(); });

    var known = {};
    blocks.forEach(function (block) {
      known[block.dataset.ecId] = 1;
      var host = block._ecUi;
      items.filter(function (c) { return c.block === block.dataset.ecId; })
        .forEach(function (c) {
          host.appendChild(makeCard(c, block));
        });
    });

    /* A generated identity is the section plus passage ordinal. Authors can set
       data-ec-id on a passage that must keep comments through reordering. Any
       identity that disappears becomes an orphan instead of silently losing its
       comments. */
    orphans.textContent = "";
    var lost = items.filter(function (c) { return !known[c.block]; });
    orphans.hidden = lost.length === 0;
    if (lost.length) {
      var h = document.createElement("h4");
      h.textContent = "Orphaned \u2014 the page changed since these were written";
      orphans.appendChild(h);
      lost.forEach(function (c) { orphans.appendChild(makeCard(c, null)); });
    }

    updateDock();
  }

  function exportMarkdown() {
    var qs = items.filter(function (c) { return c.kind === "question"; }).length;
    var ns = items.length - qs;
    var out = ["# Review of “" + (document.title || "page") + "”",
               "",
               ns + " note" + (ns === 1 ? "" : "s") + ", " + qs + " question" + (qs === 1 ? "" : "s"),
               ""];

    var seen = [];
    items.forEach(function (c) { if (seen.indexOf(c.block) < 0) seen.push(c.block); });

    seen.forEach(function (bid) {
      var group = items.filter(function (c) { return c.block === bid; });
      out.push("## " + group[0].label, "");
      group.forEach(function (c) {
        if (c.quote) out.push("> " + c.quote.replace(/\n+/g, " "), "");
        out.push("**" + (c.kind === "question" ? "Question" : "Note") + ":** " + c.body, "");
      });
    });
    return out.join("\n").trim() + "\n";
  }

  /* --- dock ------------------------------------------------------------- */

  var dock = document.createElement("div");
  dock.className = "ec-dock";
  var count = document.createElement("span");
  count.className = "ec-count";
  var copy = document.createElement("button");
  copy.type = "button";
  copy.className = "ec-copy";
  copy.textContent = "Copy for Claude";
  var clear = document.createElement("button");
  clear.type = "button";
  clear.className = "ec-clear";
  clear.textContent = "Clear";
  clear.addEventListener("click", function () {
    if (!items.length) return;
    if (window.confirm("Delete all " + items.length + " comments on this page?")) {
      items = [];
      persist();
      render();
    }
  });

  copy.addEventListener("click", function () {
    if (!items.length) return;
    var md = exportMarkdown();
    var done = function () {
      copy.textContent = "Copied — paste to Claude";
      setTimeout(function () { copy.textContent = "Copy for Claude"; }, 2600);
    };
    if (navigator.clipboard && navigator.clipboard.writeText) {
      navigator.clipboard.writeText(md).then(done, function () { fallback(md, done); });
    } else {
      fallback(md, done);
    }
  });

  /* execCommand path for file:// and any context where the async clipboard is
     blocked; if even that fails the textarea stays up so the text is selectable. */
  function fallback(md, done) {
    var ta = document.createElement("textarea");
    ta.className = "ec-fallback";
    ta.value = md;
    document.body.appendChild(ta);
    ta.select();
    var ok = false;
    try { ok = document.execCommand("copy"); } catch (e) { ok = false; }
    if (ok) { ta.remove(); done(); }
    else {
      ta.classList.add("shown");
      ta.addEventListener("blur", function () { ta.remove(); });
    }
  }

  document.body.appendChild(orphans);

  dock.appendChild(count);
  dock.appendChild(copy);
  dock.appendChild(clear);
  document.body.appendChild(dock);

  function updateDock() {
    var qs = items.filter(function (c) { return c.kind === "question"; }).length;
    var ns = items.length - qs;
    dock.classList.toggle("empty", items.length === 0);
    count.textContent = items.length
      ? ns + " note" + (ns === 1 ? "" : "s") + " · " + qs + " question" + (qs === 1 ? "" : "s")
      : "Click a speech bubble beside any passage to comment";
  }

  render();
})();
