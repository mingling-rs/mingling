/*!
 * docsify-plugin-diff
 *
 * Renders any fenced block whose language ends with `,diff` (e.g.
 * `rust,diff`, `bash,diff`) as a diff:
 *
 *     ```rust,diff
 *     use mingling::prelude::*;
 *     + gen_program!();
 *     ```
 *
 * - When the base language is known to Prism: the `+` / `-` prefixes are
 *   stripped, every line gets a whole-line background (red for `-`, green
 *   for `+`), trailing comments are moved into hover bubbles (like plain
 *   rust blocks) and the result is highlighted as that language.
 * - Otherwise: lines starting with `-` get a red highlight, lines starting
 *   with `+` get a green highlight.
 *
 * Colors live in css/light.css and css/dark.css; only the shared layout is
 * injected here.
 */
(function () {
    "use strict";

    var COMMENT_MARKERS = { rust: "//", bash: "#", toml: "#" };

    function escapeHtml(str) {
        return str
            .replace(/&/g, "&amp;")
            .replace(/</g, "&lt;")
            .replace(/>/g, "&gt;");
    }

    function escapeRegex(str) {
        return str.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
    }

    // Removes a leading `+` / `-` and one following space.
    function stripDiffPrefix(line) {
        var ch = line.charAt(0);
        if (ch !== "+" && ch !== "-") return line;
        var rest = line.slice(1);
        if (rest.charAt(0) === " ") rest = rest.slice(1);
        return rest;
    }

    function transformBlock(pre) {
        if (pre.__diffTransformed) return;
        pre.__diffTransformed = true;

        var lang = pre.getAttribute("data-lang") || "";
        var baseLang = lang.replace(/,diff$/, "");

        var source = pre.textContent.replace(/\r\n?/g, "\n");
        var lines = source.split("\n");
        while (lines.length && lines[lines.length - 1] === "") lines.pop();

        pre.classList.add("diff-block");

        var grammar = null;
        try {
            if (window.Prism && Prism.languages) {
                grammar = Prism.languages[baseLang] || null;
            }
        } catch (e) {
            /* keep plain */
        }

        // Unknown base language: red for deletions, green for additions.
        if (!grammar) {
            var htmlUnknown = lines
                .map(function (line) {
                    var ch = line.charAt(0);
                    var cls =
                        ch === "-"
                            ? "diff-line diff-del"
                            : ch === "+"
                              ? "diff-line diff-add"
                              : "diff-line";
                    return (
                        '<span class="' +
                        cls +
                        '">' +
                        escapeHtml(line) +
                        "</span>"
                    );
                })
                // No newlines between block spans: one would render an extra
                // blank line inside the pre.
                .join("");
            pre.innerHTML = "<code>" + htmlUnknown + "</code>";
            return;
        }

        // Known language: strip +/- prefixes, strip trailing comments into
        // bubbles, whole-line red/green backgrounds, then highlight as the
        // base language.
        var marker = COMMENT_MARKERS[baseLang];
        var escMarker = marker ? escapeRegex(marker) : null;
        var comments = {};

        var lineData = lines.map(function (line, i) {
            var cls = null;
            var ch = line.charAt(0);
            if (ch === "+") {
                cls = "diff-add";
                line = stripDiffPrefix(line);
            } else if (ch === "-") {
                cls = "diff-del";
                line = stripDiffPrefix(line);
            }
            if (escMarker && !new RegExp("^\\s*" + escMarker).test(line)) {
                var m = line.match(
                    new RegExp("(?:^|\\s)" + escMarker + "\\s*(.*?)\\s*$"),
                );
                if (m) {
                    comments[i] = m[1];
                    line = line.slice(0, m.index).replace(/\s+$/, "");
                }
            }
            return { code: line, cls: cls };
        });

        var codeText = lineData
            .map(function (d) {
                return d.code;
            })
            .join("\n");

        var highlighted;
        try {
            highlighted = Prism.highlight(codeText, grammar, baseLang);
        } catch (e) {
            highlighted = escapeHtml(codeText);
        }
        var hlLines = highlighted.split("\n");

        // Wrap every line in a block span and join without newlines: block
        // spans already start on their own line, and a `\n` between two block
        // spans would render an extra blank line inside the pre.
        var html = hlLines
            .map(function (hl, i) {
                var d = lineData[i];
                var cls = "diff-line" + (d.cls ? " " + d.cls : "");
                return '<span class="' + cls + '">' + hl + "</span>";
            })
            .join("");

        pre.innerHTML = "<code>" + html + "</code>";
        pre.__lineComments = comments;
    }

    // Shared layout only; colors come from the theme stylesheets. The
    // min-height keeps blank source lines (empty spans) full line-height, so
    // the hover line/bubble math stays aligned.
    var CSS = [
        // The code element must be block-level: an inline element wrapping
        // block spans would add a phantom half-line at the end.
        ".diff-block code{display:block;}",
        ".diff-line{display:block;white-space:pre;min-height:1.8em;}",
    ].join("\n");

    function injectCSS() {
        if (document.getElementById("diff-style")) return;
        var style = document.createElement("style");
        style.id = "diff-style";
        style.textContent = CSS;
        document.head.appendChild(style);
    }

    function plugin(hook) {
        injectCSS();
        hook.doneEach(function () {
            var blocks = document.querySelectorAll('pre[data-lang$=",diff"]');
            [].forEach.call(blocks, transformBlock);
        });
    }

    if (window.$docsify) {
        window.$docsify.plugins = (window.$docsify.plugins || []).concat(
            plugin,
        );
    }
})();
