/*!
 * MlingDisplay — client-side port of mingling_cli/src/utils/display.rs
 *
 * Parses the same markdown-ish markup used by `display.rs` and returns HTML
 * with the equivalent ANSI styling applied as CSS classes / inline colors.
 *
 * Supported syntax:
 *   - Bold:        **text**
 *   - Italic:      *text*
 *   - Underline:   _text_
 *   - Cyan:        <text>
 *   - Inline code: `text`  (green)
 *   - Colors:      [[color_name]] ... [[/]]   (nested)
 *   - Escapes:     \* \< \> \` \_
 *   - Headings:    # Heading 1 ... ###### Heading 6
 *   - Blockquote:  > text
 *
 * Exposes: window.MlingDisplay.parse(text) -> HTML string
 */
(function () {
    "use strict";

    function findMatch(chars, start, delimiter) {
        var len = delimiter.length;
        for (var j = start; j < chars.length; j++) {
            if (len === 1) {
                if (chars[j] === delimiter) return j;
            } else if (
                j + 1 < chars.length &&
                chars[j] === delimiter[0] &&
                chars[j + 1] === delimiter[1]
            ) {
                return j;
            }
        }
        return -1;
    }

    var COLOR_MAP = {
        black: "#000",
        red: "#ff7b72",
        green: "#7ee787",
        yellow: "#d4a84b",
        blue: "#79c0ff",
        magenta: "#d2a8ff",
        cyan: "#79c0ff",
        white: "#e8ddd0",
        b_white: "#e8ddd0",
        bright_gray: "#6a5a4a",
        bright_grey: "#6a5a4a",
        b_gray: "#6a5a4a",
        b_grey: "#6a5a4a",
        bright_black: "#6a5a4a",
        b_black: "#6a5a4a",
        gray: "#6a5a4a",
        grey: "#6a5a4a",
        bright_red: "#ff7b72",
        b_red: "#ff7b72",
        bright_green: "#7ee787",
        b_green: "#7ee787",
        bright_yellow: "#d4a84b",
        b_yellow: "#d4a84b",
        bright_blue: "#79c0ff",
        b_blue: "#79c0ff",
        bright_magenta: "#d2a8ff",
        b_magenta: "#d2a8ff",
        bright_cyan: "#79c0ff",
        b_cyan: "#79c0ff",
        bright_white: "#e8ddd0",
    };

    function applyColorStack(text, stack) {
        var wrapped = text;
        for (var k = stack.length - 1; k >= 0; k--) {
            var css = COLOR_MAP[stack[k]];
            if (css) {
                wrapped =
                    '<span style="color:' + css + '">' + wrapped + "</span>";
            }
        }
        return wrapped;
    }

    function escapeHtml(s) {
        return s
            .replace(/&/g, "&amp;")
            .replace(/</g, "&lt;")
            .replace(/>/g, "&gt;");
    }

    function processLine(line) {
        var chars = Array.from(line);
        var result = "";
        var colorStack = [];
        var i = 0;

        while (i < chars.length) {
            var c = chars[i];

            // Escape sequences \\* \\< \\> \\` \\_
            if (
                c === "\\" &&
                i + 1 < chars.length &&
                ["*", "<", ">", "`", "_"].indexOf(chars[i + 1]) !== -1
            ) {
                result += escapeHtml(chars[i + 1]);
                i += 2;
                continue;
            }

            // Color tags [[color]] / [[/]]
            if (c === "[" && chars[i + 1] === "[") {
                var tagEnd = -1;
                for (var t = i + 2; t + 1 < chars.length; t++) {
                    if (chars[t] === "]" && chars[t + 1] === "]") {
                        tagEnd = t;
                        break;
                    }
                }
                if (tagEnd !== -1) {
                    var tag = chars.slice(i + 2, tagEnd).join("");
                    if (tag === "/") colorStack.pop();
                    else colorStack.push(tag);
                    i = tagEnd + 2;
                    continue;
                }
            }

            // **bold**
            if (c === "*" && chars[i + 1] === "*") {
                var bEnd = findMatch(chars, i + 2, "**");
                if (bEnd !== -1) {
                    var inner = processLine(chars.slice(i + 2, bEnd).join(""));
                    result += applyColorStack(
                        "<b>" + inner + "</b>",
                        colorStack,
                    );
                    i = bEnd + 2;
                    continue;
                }
            }

            // *italic*
            if (c === "*") {
                var iEnd = findMatch(chars, i + 1, "*");
                if (iEnd !== -1) {
                    var italic = processLine(chars.slice(i + 1, iEnd).join(""));
                    result += applyColorStack(
                        "<i>" + italic + "</i>",
                        colorStack,
                    );
                    i = iEnd + 1;
                    continue;
                }
            }

            // _underline_
            if (c === "_") {
                var uEnd = findMatch(chars, i + 1, "_");
                if (uEnd !== -1) {
                    var uText = processLine(chars.slice(i + 1, uEnd).join(""));
                    result += applyColorStack(
                        "<u>" + uText + "</u>",
                        colorStack,
                    );
                    i = uEnd + 1;
                    continue;
                }
            }

            // <angle> cyan
            if (c === "<") {
                var cEnd = findMatch(chars, i + 1, ">");
                if (cEnd !== -1) {
                    var angle = chars.slice(i, cEnd + 1).join("");
                    result += applyColorStack(
                        '<span class="t-cyan">' + escapeHtml(angle) + "</span>",
                        colorStack,
                    );
                    i = cEnd + 1;
                    continue;
                }
            }

            // `code` green
            if (c === "`") {
                var gEnd = findMatch(chars, i + 1, "`");
                if (gEnd !== -1) {
                    var codeText = chars.slice(i, gEnd + 1).join("");
                    result += applyColorStack(
                        '<span class="t-green">' +
                            escapeHtml(codeText) +
                            "</span>",
                        colorStack,
                    );
                    i = gEnd + 1;
                    continue;
                }
            }

            // Regular character
            result += applyColorStack(escapeHtml(c), colorStack);
            i += 1;
        }
        return result;
    }

    function processLineWithQuote(line) {
        var chars = Array.from(line);
        if (chars.length && chars[0] === ">") {
            if (chars.length > 1 && chars[1] === "\\") {
                return processLine(line);
            }
            var rest = chars.length > 1 ? chars.slice(1).join("") : "";
            return '<span class="t-quote"> </span>' + processLine(rest);
        }
        return processLine(line);
    }

    /**
     * Parse display.rs-flavored markup into HTML.
     * @param {string} text - Raw markup text.
     * @returns {string} HTML with t-cyan / t-green / t-heading / t-quote classes
     *                   and inline colors for [[color]] tags.
     */
    function parse(text) {
        var lines = String(text).split("\n");
        var result = "";
        var contentIndent = 0;
        for (var n = 0; n < lines.length; n++) {
            var line = lines[n];
            var trimmed = line.trim();
            var ls = line.trimStart();
            var lineResult = "";

            if (ls.startsWith("#")) {
                var level = 0;
                while (level < ls.length && level < 7 && ls[level] === "#") {
                    level++;
                }
                var effective = level > 6 ? 6 : level;
                var start = level;
                while (
                    start < ls.length &&
                    (ls[start] === " " || ls[start] === "\t")
                ) {
                    start++;
                }
                var content = start < ls.length ? ls.slice(start) : "";
                var heading =
                    '<span class="t-heading"> ' +
                    processLine(content) +
                    " </span>";
                var indent = " ".repeat(effective > 0 ? effective - 1 : 0);
                lineResult = indent + heading;
                contentIndent = effective;
            } else if (trimmed !== "") {
                lineResult =
                    " ".repeat(contentIndent) + processLineWithQuote(trimmed);
            } else {
                lineResult = " ";
            }
            result += lineResult + "\n";
        }
        return result;
    }

    window.MlingDisplay = { parse: parse };
})();
