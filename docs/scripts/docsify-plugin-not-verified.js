/*!
 * docsify-plugin-not-verified
 *
 * Renders code blocks marked with `// NOT VERIFIED` as a visible warning:
 *
 * - The `// NOT VERIFIED` marker line(s) are removed from the rendered code.
 * - The whole code block gets a red highlight outline.
 * - While the pointer is over the block, a localized tooltip follows the
 *   cursor on its right side saying that the block has not been verified.
 *
 * Configuration (put the localized text in each index.html):
 *
 *     window.$docsify.notVerified = {
 *         text: "This code block is not verified"
 *     };
 *
 * The theme colors live in css/light.css and css/dark.css; only the shared
 * layout is injected here.
 */
(function () {
    "use strict";

    var MARKER_RE = /^\s*\/\/\s*NOT\s+VERIFIED\b.*$/i;

    function getConfig() {
        var cfg = (window.$docsify && window.$docsify.notVerified) || {};
        return {
            text: cfg.text || "This code block is not verified",
        };
    }

    // Removes the `// NOT VERIFIED` marker lines and re-highlights the block.
    // Re-highlighting is done here so the marker never appears in the DOM.
    function transformBlock(pre) {
        if (pre.__notVerifiedTransformed) return;
        pre.__notVerifiedTransformed = true;

        var code = pre.querySelector("code");
        if (!code) return;

        var lang = pre.getAttribute("data-lang") || "";
        // Leave specialized renderers (terminal/file-tree) untouched.
        if (
            pre.classList.contains("shell-sim") ||
            pre.classList.contains("file-tree") ||
            /(?:^|,)simulation$/.test(lang)
        ) {
            return;
        }

        var source = pre.textContent.replace(/\r\n?/g, "\n");
        var lines = source.split("\n");
        var hasMarker = lines.some(function (line) {
            return MARKER_RE.test(line);
        });
        if (!hasMarker) return;

        pre.classList.add("not-verified");
        pre.__notVerified = true;

        var cleaned = lines
            .filter(function (line) {
                return !MARKER_RE.test(line);
            })
            .join("\n");

        // For `rust,diff`-style blocks, highlight with the base language. If
        // the diff plugin runs later it will re-render the block on its own.
        var baseLang = lang.replace(/,diff$/, "");

        try {
            if (
                window.Prism &&
                Prism.languages &&
                Prism.languages[baseLang]
            ) {
                code.innerHTML = Prism.highlight(
                    cleaned,
                    Prism.languages[baseLang],
                    baseLang,
                );
                return;
            }
        } catch (e) {
            /* fall back to plain text below */
        }

        code.textContent = cleaned;
    }

    /* ── hover tooltip ─────────────────────────────────────── */

    var bubble = null;

    function ensureBubble() {
        if (!bubble) {
            bubble = document.createElement("div");
            bubble.className = "not-verified-bubble";
            document.body.appendChild(bubble);
        }
        return bubble;
    }

    function showBubble(pre, e) {
        var b = ensureBubble();
        b.textContent = getConfig().text;

        // Follow the cursor: prefer the right side, flip to the left when
        // there is no room on the right.
        var left = e.clientX + 12;
        if (left + b.offsetWidth > window.innerWidth - 8) {
            left = e.clientX - b.offsetWidth - 12;
        }
        var top = e.clientY - b.offsetHeight / 2;
        if (top < 8) top = 8;
        if (top + b.offsetHeight > window.innerHeight - 8) {
            top = window.innerHeight - b.offsetHeight - 8;
        }

        b.style.left = left + "px";
        b.style.top = top + "px";
        b.style.opacity = "1";
    }

    function hideBubble() {
        if (bubble) bubble.style.opacity = "0";
    }

    function attach(pre) {
        if (pre.__notVerifiedAttached) return;
        pre.__notVerifiedAttached = true;
        pre.addEventListener("mouseenter", function (e) {
            showBubble(pre, e);
        });
        pre.addEventListener("mousemove", function (e) {
            showBubble(pre, e);
        });
        pre.addEventListener("mouseleave", hideBubble);
    }

    /* ── shared layout only; colors come from the theme files ── */

    var CSS = [
        ".not-verified-bubble{position:fixed;pointer-events:none;",
        "max-width:min(90vw,480px);padding:6px 12px;border-radius:4px;",
        "font-size:13px;line-height:1.5;opacity:0;",
        "transition:opacity .15s ease;z-index:10002;box-sizing:border-box;}",
    ].join("\n");

    function injectCSS() {
        if (document.getElementById("not-verified-style")) return;
        var style = document.createElement("style");
        style.id = "not-verified-style";
        style.textContent = CSS;
        document.head.appendChild(style);
    }

    function plugin(hook) {
        injectCSS();
        hook.doneEach(function () {
            var blocks = document.querySelectorAll(".markdown-section pre");
            [].forEach.call(blocks, function (pre) {
                transformBlock(pre);
            });
            [].forEach.call(
                document.querySelectorAll("pre.not-verified"),
                attach,
            );
        });
    }

    if (window.$docsify) {
        window.$docsify.plugins = (window.$docsify.plugins || []).concat(
            plugin,
        );
    }
})();
