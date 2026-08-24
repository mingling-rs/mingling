/*!
 * docsify-plugin-line-highlight
 *
 * Hover line highlight for code blocks, ported from the landing page demo
 * (index.html `attachLineHighlight`): a translucent bar follows the mouse
 * over `pre` blocks and marks the line under the cursor.
 *
 * - Normal code blocks use the same line-height math as the demo.
 * - shell-simulation terminal blocks resolve their `.shell-sim-line`
 *   elements on every move (they grow while the demo plays) and rebuild the
 *   highlight bar if the loop restart cleared it.
 *
 * The overlay colors live in css/light.css and css/dark.css
 * (`.code-line-highlight`), so they follow the theme switch; only the shared
 * layout is injected here.
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

    function attach(pre) {
        if (pre.__lineHighlightAttached) return;
        pre.__lineHighlightAttached = true;

        pre.addEventListener("mousemove", function (e) {
            var rect = pre.getBoundingClientRect();
            var y = e.clientY - rect.top;

            // Terminal blocks: each line is an element, and new lines appear
            // while the demo plays, so resolve the target line on every move.
            var lines = pre.querySelectorAll(".shell-sim-line");
            if (lines.length) {
                var target = lines[0];
                for (var i = 0; i < lines.length; i++) {
                    var t = lines[i].getBoundingClientRect().top - rect.top;
                    if (y >= t) target = lines[i];
                    else break;
                }
                var tr = target.getBoundingClientRect();
                var hlT = ensureHighlight(pre);
                hlT.style.top = tr.top - rect.top + "px";
                hlT.style.height = tr.height + "px";
                hlT.style.opacity = "1";
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
            hlC.style.top = prePadTop + codePadTop + index * lineHeight + "px";
            hlC.style.height = lineHeight + "px";
            hlC.style.opacity = "1";
        });

        pre.addEventListener("mouseleave", function () {
            ensureHighlight(pre).style.opacity = "0";
        });
    }

    // Shared layout only; the background/border colors are provided by the
    // theme stylesheets (css/light.css, css/dark.css).
    var CSS = [
        ".markdown-section pre{position:relative;}",
        ".code-line-highlight{position:absolute;left:0;right:0;height:28px;",
        "pointer-events:none;opacity:0;",
        "transition:opacity .15s ease,top .08s ease;z-index:3;}",
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
            [].forEach.call(blocks, attach);
        });
    }

    if (window.$docsify) {
        window.$docsify.plugins = (window.$docsify.plugins || []).concat(
            plugin,
        );
    }
})();
