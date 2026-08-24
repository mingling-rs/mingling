/*!
 * cursor-effects.js
 *
 * Ported from the landing page (index.html):
 * - `.cursor-dot`: a soft glow (dark theme) / inverted-color circle (light
 *   theme) that appears only when the cursor points at text (text-selection
 *   mode) and shrinks away otherwise.
 * - `.ink-drop`: a radial splash expands at every click.
 *
 * The colors/sizes live in css/light.css and css/dark.css; only the shared
 * layout is injected here.
 */
(function () {
    "use strict";

    if (
        window.matchMedia &&
        window.matchMedia("(prefers-reduced-motion: reduce)").matches
    ) {
        return;
    }

    var CSS = [
        // Sizes and shape live in the theme files: dark = 400px glow circle,
        // light = medium inverted-color block with hard edges. The :where()
        // fallback is zero-specificity so the theme files always win.
        ":where(.cursor-dot){width:400px;height:400px;border-radius:50%;}",
        ".cursor-dot{position:fixed;pointer-events:none;",
        "transform:translate(-50%,-50%) scale(0);z-index:10000;",
        "opacity:0;transition:opacity .3s ease,transform .3s ease,",
        "left .12s ease-out,top .12s ease-out;}",
        ".cursor-dot.active{opacity:1;transform:translate(-50%,-50%) scale(1);}",
        ":where(.ink-drop){width:500px;height:500px;}",
        ".ink-drop{position:fixed;pointer-events:none;border-radius:50%;",
        "transform:translate(-50%,-50%) scale(0);",
        "animation:inkSpread .9s ease-out forwards;z-index:9999;}",
        "@keyframes inkSpread{0%{transform:translate(-50%,-50%) scale(0);opacity:1;}",
        "100%{transform:translate(-50%,-50%) scale(1);opacity:0;}}",
    ].join("\n");

    function injectCSS() {
        if (document.getElementById("cursor-effects-style")) return;
        var style = document.createElement("style");
        style.id = "cursor-effects-style";
        style.textContent = CSS;
        document.head.appendChild(style);
    }

    // True when the point is over selectable text: either an element with an
    // explicit text cursor (inputs, textareas, cursor:text) or a caret range
    // that is really inside a text node and visually near the pointer
    // (caretRangeFromPoint snaps to the nearest glyph, so without the
    // distance check empty space would count as text).
    function isOverText(x, y) {
        var el = document.elementFromPoint(x, y);
        if (!el) return false;
        try {
            if (getComputedStyle(el).cursor === "text") return true;
        } catch (e) {
            /* ignore */
        }
        var caret =
            document.caretRangeFromPoint || document.caretPositionFromPoint;
        if (!caret) return false;
        var pos;
        try {
            pos = caret.call(document, x, y);
        } catch (e) {
            return false;
        }
        if (!pos) return false;

        // caretRangeFromPoint -> Range; caretPositionFromPoint -> CaretPosition
        var container = pos.startContainer || pos.offsetNode;
        if (!container || container.nodeType !== 3) return false;

        var rect = pos.getBoundingClientRect
            ? pos.getBoundingClientRect()
            : null;
        if (!rect && pos.getClientRect) rect = pos.getClientRect();
        if (!rect) return true;

        // The caret is a zero-width box on the snapped line; require the
        // pointer to be within about a character of it.
        return Math.abs(rect.left - x) < 24 && Math.abs(rect.top - y) < 24;
    }

    // The dark theme's glow dot stays on while moving; only the light theme's
    // inverted circle is gated by the text-selection mode.
    function isDarkTheme() {
        var dark = document.getElementById("dark-style");
        return !!dark && !dark.disabled;
    }

    function init() {
        injectCSS();

        // Follow the mouse; light theme shows only over text, dark theme is
        // always on after the first move.
        var cursorDot = document.createElement("div");
        cursorDot.className = "cursor-dot";
        document.body.appendChild(cursorDot);

        document.addEventListener("mousemove", function (e) {
            cursorDot.style.left = e.clientX + "px";
            cursorDot.style.top = e.clientY + "px";
            cursorDot.classList.toggle(
                "active",
                isDarkTheme() || isOverText(e.clientX, e.clientY),
            );
        });

        // Expanding splash on click.
        document.addEventListener("click", function (e) {
            var drop = document.createElement("div");
            drop.className = "ink-drop";
            drop.style.left = e.clientX + "px";
            drop.style.top = e.clientY + "px";
            document.body.appendChild(drop);
            setTimeout(function () {
                drop.remove();
            }, 1000);
        });
    }

    if (document.readyState === "loading") {
        document.addEventListener("DOMContentLoaded", init);
    } else {
        init();
    }
})();
