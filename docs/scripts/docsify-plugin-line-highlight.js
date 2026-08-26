/*!
 * docsify-plugin-line-highlight
 *
 * Hover line highlight, ported from the landing page demo (index.html
 * `attachLineHighlight`): a translucent bar follows the mouse and marks the
 * line under the cursor.
 *
 * - Code blocks: normal blocks use the same line-height math as the demo;
 *   shell-simulation terminal blocks resolve their `.shell-sim-line`
 *   elements on every move (they grow while the demo plays) and rebuild the
 *   highlight bar if the loop restart cleared it.
 * - Sidebar navigation: the same bar highlights the nav item under the
 *   cursor (the sidebar is re-rendered per page, so the bar is rebuilt on
 *   demand).
 * - Rust (`//`), TOML (`#`) and bash (`#`) code blocks: trailing comments
 *   are stripped from the rendered code (comment-only lines are kept) and a
 *   small bubble below the hovered line shows the comment text of that line.
 *   Simulation blocks (bash,simulation / shell-simulation) are not touched.
 *
 * The overlay colors live in css/light.css and css/dark.css
 * (`.code-line-highlight`, `.code-bubble`), so they follow the theme switch;
 * only the shared layout is injected here.
 */
(function () {
    "use strict";

    function ensureHighlight(pre) {
        var hl = pre.querySelector(".code-line-highlight");
        if (!hl) {
            hl = document.createElement("div");
            hl.className = "code-line-highlight";
            pre.appendChild(hl);
        }
        return hl;
    }

    function ensureBubble() {
        var b = document.querySelector(".code-bubble");
        if (!b) {
            b = document.createElement("div");
            b.className = "code-bubble";
            document.body.appendChild(b);
        }
        return b;
    }

    function escapeHtml(str) {
        return str
            .replace(/&/g, "&amp;")
            .replace(/</g, "&lt;")
            .replace(/>/g, "&gt;");
    }

    // Renders the comment text into the bubble; `code` spans become <code>.
    function setBubbleText(bubble, text) {
        bubble.innerHTML = escapeHtml(text).replace(
            /`([^`]+)`/g,
            "<code>$1</code>",
        );
    }

    function escapeRegex(str) {
        return str.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
    }

    // For rust (`//`) and TOML (`#`) code blocks, removes trailing comments
    // (except on comment-only lines) from the rendered code and remembers
    // them, so a bubble can show them while hovering the line.
    function stripTrailingComments(pre) {
        if (pre.__commentsStripped) return;
        pre.__commentsStripped = true;

        var code = pre.querySelector("code");
        if (!code) return;
        var lang = pre.getAttribute("data-lang") || "";
        // Simulation blocks (bash,simulation / shell-simulation) are left
        // untouched.
        var marker =
            lang === "rust"
                ? "//"
                : lang === "toml" || lang === "bash"
                  ? "#"
                  : null;
        if (!marker) return;

        var esc = escapeRegex(marker);
        var text = code.textContent.replace(/\r\n?/g, "\n");
        var lines = text.split("\n");
        var comments = {};
        var stripped = lines.map(function (line, i) {
            // Comment-only lines are kept as-is.
            if (new RegExp("^\\s*" + esc).test(line)) return line;
            var m = line.match(
                new RegExp("(?:^|\\s)" + esc + "\\s*(.*?)\\s*$"),
            );
            if (!m) return line;
            comments[i] = m[1];
            return line.slice(0, m.index).replace(/\s+$/, "");
        });
        pre.__lineComments = comments;

        var out = stripped.join("\n");
        try {
            if (window.Prism && Prism.languages && Prism.languages[lang]) {
                code.innerHTML = Prism.highlight(
                    out,
                    Prism.languages[lang],
                    lang,
                );
                return;
            }
        } catch (e) {
            /* keep plain */
        }
        code.textContent = out;
    }

    function showBubble(pre, comment, e, lineBottom) {
        var bubble = ensureBubble();
        if (comment) {
            setBubbleText(bubble, comment);
            bubble.style.top = lineBottom + 6 + "px";
            // Follow the mouse horizontally; flip to the left of the cursor
            // when there is no room on the right.
            var bubbleWidth = bubble.offsetWidth || 0;
            var left = e.clientX + 12;
            if (left + bubbleWidth > window.innerWidth - 8) {
                left = e.clientX - bubbleWidth - 12;
            }
            bubble.style.left = Math.max(8, left) + "px";
            bubble.style.opacity = "1";
        } else {
            bubble.style.opacity = "0";
        }
    }

    function attach(pre) {
        if (pre.__lineHighlightAttached) return;
        pre.__lineHighlightAttached = true;

        pre.addEventListener("mousemove", function (e) {
            var rect = pre.getBoundingClientRect();
            var y = e.clientY - rect.top;

            // Terminal/diff blocks: each line is an element (and new lines
            // appear while the demo plays), so resolve the target line and
            // its index from the real rectangles.
            var lines = pre.querySelectorAll(".shell-sim-line, .diff-line");
            if (lines.length) {
                var target = lines[0];
                var targetIndex = 0;
                for (var i = 0; i < lines.length; i++) {
                    var t = lines[i].getBoundingClientRect().top - rect.top;
                    if (y >= t) {
                        target = lines[i];
                        targetIndex = i;
                    } else {
                        break;
                    }
                }
                var tr = target.getBoundingClientRect();
                var hlT = ensureHighlight(pre);
                hlT.style.top = tr.top - rect.top - 1 + "px";
                hlT.style.height = tr.height + "px";
                hlT.style.opacity = "1";

                // Tint the bar (and its left accent) with the diff line's
                // background color.
                if (
                    target.classList.contains("diff-del") ||
                    target.classList.contains("diff-add")
                ) {
                    var diffBg = getComputedStyle(target).backgroundColor;
                    hlT.style.backgroundColor = diffBg;
                    hlT.style.borderLeftColor = diffBg;
                } else {
                    hlT.style.backgroundColor = "";
                    hlT.style.borderLeftColor = "";
                }

                var comment = pre.__lineComments
                    ? pre.__lineComments[targetIndex]
                    : null;
                showBubble(pre, comment, e, tr.bottom);
                return;
            }

            // Normal code blocks: fixed line-height math (same as the demo),
            // offset by the pre's border + padding and the code's padding.
            var code = pre.querySelector("code");
            if (!code) return;
            var preCs = getComputedStyle(pre);
            var cs = getComputedStyle(code);
            var lineHeight = parseFloat(cs.lineHeight);
            if (!lineHeight) lineHeight = parseFloat(cs.fontSize) * 1.5;
            var prePadTop = parseFloat(preCs.paddingTop) || 0;
            var codePadTop = parseFloat(cs.paddingTop) || 0;
            var offset =
                (parseFloat(preCs.borderTopWidth) || 0) +
                prePadTop +
                codePadTop;
            var index = Math.floor((y - offset) / lineHeight);
            // Count real lines (docsify appends a trailing newline).
            var lineCount = code.innerText
                .replace(/\n+$/, "")
                .split("\n").length;
            if (index < 0) index = 0;
            if (index >= lineCount) index = lineCount - 1;
            var hlC = ensureHighlight(pre);
            // The highlight is positioned relative to the pre's padding box.
            hlC.style.top =
                prePadTop + codePadTop + index * lineHeight - 1 + "px";
            hlC.style.height = lineHeight + "px";
            hlC.style.opacity = "1";

            // Trailing-comment bubble for rust blocks.
            var comment = pre.__lineComments ? pre.__lineComments[index] : null;
            var lineTop =
                rect.top +
                (parseFloat(preCs.borderTopWidth) || 0) +
                prePadTop +
                codePadTop +
                index * lineHeight;
            showBubble(pre, comment, e, lineTop + lineHeight);
        });

        pre.addEventListener("mouseleave", function () {
            ensureHighlight(pre).style.opacity = "0";
            ensureBubble().style.opacity = "0";
        });
    }

    function ensureSidebarHighlight(nav) {
        var hl = nav.querySelector(".code-line-highlight");
        if (!hl) {
            hl = document.createElement("div");
            hl.className = "code-line-highlight";
            nav.appendChild(hl);
        }
        return hl;
    }

    // Highlights the nav item under the cursor inside the sidebar. The bar
    // lives in `.sidebar-nav` (relative to its content, so scrolling is
    // handled) and is rebuilt when docsify re-renders the nav.
    function attachSidebar(sidebar) {
        if (sidebar.__lineHighlightAttached) return;
        sidebar.__lineHighlightAttached = true;

        sidebar.addEventListener("mousemove", function (e) {
            var nav = sidebar.querySelector(".sidebar-nav");
            if (!nav) return;
            var el = document.elementFromPoint(e.clientX, e.clientY);
            var link = el && el.closest ? el.closest("a") : null;
            var hl = ensureSidebarHighlight(nav);
            if (!link || !sidebar.contains(link)) {
                hl.style.opacity = "0";
                return;
            }
            var rect = link.getBoundingClientRect();
            var navRect = nav.getBoundingClientRect();
            hl.style.top = rect.top - navRect.top + "px";
            hl.style.height = rect.height + "px";
            hl.style.opacity = "1";
        });

        sidebar.addEventListener("mouseleave", function () {
            var nav = sidebar.querySelector(".sidebar-nav");
            if (!nav) return;
            ensureSidebarHighlight(nav).style.opacity = "0";
        });
    }

    // Shared layout only; the background/border colors are provided by the
    // theme stylesheets (css/light.css, css/dark.css).
    var CSS = [
        ".markdown-section pre{position:relative;}",
        ".sidebar .sidebar-nav{position:relative;}",
        ".code-line-highlight{position:absolute;left:0;right:0;height:28px;",
        "pointer-events:none;opacity:0;",
        "transition:opacity .15s ease,top .08s ease;z-index:3;}",
        ".code-bubble{position:fixed;pointer-events:none;max-width:360px;",
        "padding:4px 10px;border-radius:4px;font-size:13px;line-height:1.5;",
        "opacity:0;transition:opacity .15s ease;z-index:10001;}",
        '.code-bubble code{font-family:"JetBrains Mono","Noto Serif SC",monospace;',
        "font-size:.92em;padding:0 .25em;border-radius:2px;}",
    ].join("\n");

    function injectCSS() {
        if (document.getElementById("line-highlight-style")) return;
        var style = document.createElement("style");
        style.id = "line-highlight-style";
        style.textContent = CSS;
        document.head.appendChild(style);
    }

    function plugin(hook) {
        injectCSS();
        hook.doneEach(function () {
            var blocks = document.querySelectorAll(".markdown-section pre");
            [].forEach.call(blocks, function (pre) {
                stripTrailingComments(pre);
                attach(pre);
            });

            var sidebar = document.querySelector(".sidebar");
            if (sidebar) attachSidebar(sidebar);
        });
    }

    if (window.$docsify) {
        window.$docsify.plugins = (window.$docsify.plugins || []).concat(
            plugin,
        );
    }
})();
