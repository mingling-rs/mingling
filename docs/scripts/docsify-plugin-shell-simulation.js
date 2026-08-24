/*!
 * docsify-plugin-shell-simulation
 *
 * Turns fenced code blocks declared with the `shell-simulation` language (or
 * `bash,simulation`, which lets editors highlight the fence as bash) into
 * animated terminal demos:
 *
 *     ```shell-simulation
 *     ~# cargo run -- greet
 *     Hello, World!
 *     ~# cargo run -- great
 *     <<3
 *     ```
 *
 * - Lines starting with `~#` are input lines. The `~# ` prompt is shown
 *   immediately and the command is typed character by character. Every
 *   non-letter character (space, `-`, `.`, ...) is typed at twice the speed of
 *   a letter. When the line is finished it waits for the duration of 5 letters
 *   and then the following output lines are typed character by character at
 *   twice the input speed (outputSpeed), followed by another pause of 5
 *   letters before the next `~#` line or the end of the block.
 * - Command text keeps Prism bash highlighting.
 * - An output line ending with `<<N` (an exit code, e.g. `<<3`) is shown as
 *   `<-- N` in red at the end of the line.
 * - The demo loops: when the animation finishes it restarts after a pause
 *   (3 seconds by default). The block height is reserved up front, so the
 *   animation never shifts the surrounding layout.
 * - The terminal colors follow the docs light/dark theme: plain `.shell-sim`
 *   rules in `css/light.css` and `css/dark.css` (same mechanism as the rest
 *   of the docs); the Prism token colors come from the theme files too.
 *   The plugin only injects layout styles; the `:where()` fallbacks keep the
 *   demo usable on sites that load it without those theme rules.
 *
 * Configuration (optional), e.g.:
 *
 *     window.$docsify.shellSimulation = {
 *         interval: 60,          // ms per letter
 *         nonLetterFactor: 2,    // non-letter interval = interval * factor
 *         endWaitLetters: 5,     // letters to wait after an input line
 *         outputSpeed: 2,        // output typing speed vs. input speed
 *         playOnView: true,      // start the animation when scrolled into view
 *         loop: true,            // restart the demo after it finishes
 *         loopDelay: 3000,       // ms to wait before restarting
 *         exitMark: "<--"        // prefix used for the exit-code marker
 *     };
 */
(function () {
    "use strict";

    var PROMPT_RE = /^~# ?/;
    var EXIT_RE = /^(.*?)<<[ \t]*(-?\d+)[ \t]*$/;

    var reduceMotion =
        window.matchMedia &&
        window.matchMedia("(prefers-reduced-motion: reduce)").matches;

    /* ── configuration ─────────────────────────────────────── */

    function getConfig() {
        var cfg = (window.$docsify && window.$docsify.shellSimulation) || {};
        return {
            interval: cfg.interval != null ? cfg.interval : 60,
            nonLetterFactor:
                cfg.nonLetterFactor != null ? cfg.nonLetterFactor : 2,
            endWaitLetters: cfg.endWaitLetters != null ? cfg.endWaitLetters : 5,
            outputSpeed: cfg.outputSpeed != null ? cfg.outputSpeed : 2,
            playOnView: cfg.playOnView !== false,
            loop: cfg.loop !== false,
            loopDelay: cfg.loopDelay != null ? cfg.loopDelay : 3000,
            exitMark: cfg.exitMark != null ? cfg.exitMark : "<--",
        };
    }

    /* ── helpers ───────────────────────────────────────────── */

    function escapeHtml(str) {
        return str
            .replace(/&/g, "&amp;")
            .replace(/</g, "&lt;")
            .replace(/>/g, "&gt;");
    }

    // Letters and digits (any script) are typed at normal speed; everything
    // else (space, dash, dot, ...) at twice that speed.
    function isLetter(ch) {
        return /[\p{L}\p{N}]/u.test(ch);
    }

    function highlightBash(code) {
        try {
            if (window.Prism && Prism.languages && Prism.languages.bash) {
                return Prism.highlight(code, Prism.languages.bash, "bash");
            }
        } catch (e) {
            /* keep the escaped plain text */
        }
        return escapeHtml(code);
    }

    /* ── parsing ───────────────────────────────────────────── */

    // Returns a list of steps: { type: "input", cmd } for `~#` lines and
    // { type: "output", lines: [...] } for the output that follows them.
    // Trailing blank lines (e.g. an empty line before the closing fence) are
    // trimmed so they do not render as an extra empty line.
    function parse(source) {
        var lines = source.replace(/\r\n?/g, "\n").split("\n");
        while (lines.length && lines[lines.length - 1].trim() === "") {
            lines.pop();
        }

        var steps = [];
        var output = [];

        lines.forEach(function (line) {
            if (/^~#/.test(line)) {
                if (output.length) {
                    steps.push({ type: "output", lines: output });
                    output = [];
                }
                steps.push({ type: "input", cmd: line.replace(PROMPT_RE, "") });
            } else {
                output.push(line);
            }
        });

        if (output.length) steps.push({ type: "output", lines: output });
        return steps;
    }

    /* ── DOM builders ──────────────────────────────────────── */

    function buildInputLine(cmd) {
        var line = document.createElement("div");
        line.className = "shell-sim-line shell-sim-input";

        var prompt = document.createElement("span");
        prompt.className = "shell-sim-prompt";
        prompt.textContent = "~# ";

        var reveal = document.createElement("span");
        reveal.className = "shell-sim-reveal";

        var cmdSpan = document.createElement("span");
        cmdSpan.className = "shell-sim-cmd";
        cmdSpan.innerHTML = highlightBash(cmd);
        reveal.appendChild(cmdSpan);

        var cursor = document.createElement("span");
        cursor.className = "shell-sim-cursor";

        line.appendChild(prompt);
        line.appendChild(reveal);
        line.appendChild(cursor);
        return line;
    }

    // Builds an output line. Returns { el, text }: `el` contains a reveal
    // span holding the rendered content (exit codes become a red `<-- N` mark)
    // and `text` is the plain text used for typing/measuring.
    function buildOutputLine(text) {
        var line = document.createElement("div");
        line.className = "shell-sim-line shell-sim-output";

        var reveal = document.createElement("span");
        reveal.className = "shell-sim-reveal";

        var plain = text;
        var match = text.match(EXIT_RE);
        if (match) {
            var prefix = match[1].replace(/[ \t]+$/, "");
            plain =
                (prefix ? prefix + " " : "") + CONFIG.exitMark + " " + match[2];
            reveal.innerHTML =
                escapeHtml(prefix) +
                (prefix ? " " : "") +
                '<span class="shell-sim-exit">' +
                escapeHtml(CONFIG.exitMark + " " + match[2]) +
                "</span>";
        } else {
            reveal.textContent = text;
        }

        line.appendChild(reveal);
        return { el: line, text: plain };
    }

    /* ── animation ─────────────────────────────────────────── */

    // Types the text inside `lineEl` character by character. The full content
    // is rendered once; a measured width progressively reveals it (keeps
    // Prism highlighting on commands). `speed` is a multiplier relative to
    // CONFIG.interval (output lines type at outputSpeed, commands at 1).
    function typeCommand(pre, lineEl, cmd, speed, done) {
        var reveal = lineEl.querySelector(".shell-sim-reveal");
        var cursor = lineEl.querySelector(".shell-sim-cursor");
        var measurer = pre.querySelector(".shell-sim-measure");
        var letterMs = CONFIG.interval / speed;

        if (!cmd) {
            if (cursor) cursor.style.display = "none";
            done();
            return;
        }

        var i = 0;

        function show(n) {
            measurer.textContent = cmd.slice(0, n);
            reveal.style.width = measurer.offsetWidth + "px";
        }

        function tick() {
            if (!pre.isConnected) return;
            if (i >= cmd.length) {
                if (cursor) cursor.style.display = "none";
                done();
                return;
            }
            i++;
            show(i);
            var ch = cmd.charAt(i - 1);
            var ms = isLetter(ch)
                ? letterMs
                : letterMs * CONFIG.nonLetterFactor;
            setTimeout(tick, ms);
        }

        show(0);
        setTimeout(tick, letterMs);
    }

    function runBlock(pre, steps) {
        var measurer = document.createElement("span");
        measurer.className = "shell-sim-measure";
        pre.appendChild(measurer);

        var idx = 0;

        // Types the output lines one by one at outputSpeed, with a short beat
        // between lines.
        function typeOutputs(lines, done) {
            var lineIdx = 0;
            var beat = CONFIG.interval / CONFIG.outputSpeed;

            function nextLine() {
                if (lineIdx >= lines.length) {
                    done();
                    return;
                }
                var built = buildOutputLine(lines[lineIdx]);
                pre.appendChild(built.el);
                lineIdx++;
                typeCommand(
                    pre,
                    built.el,
                    built.text,
                    CONFIG.outputSpeed,
                    function () {
                        setTimeout(nextLine, beat);
                    },
                );
            }

            nextLine();
        }

        function cleanup() {
            if (measurer.parentNode) measurer.parentNode.removeChild(measurer);
        }

        function finish() {
            cleanup();
            if (!CONFIG.loop || !pre.isConnected || steps.length === 0) return;

            // Blinking cursor at the end of the last line while waiting to
            // restart (reuse the input line's cursor if there is one).
            var last = pre.children[pre.children.length - 1];
            if (last) {
                var cursor = last.querySelector(".shell-sim-cursor");
                if (cursor) {
                    cursor.style.display = "";
                } else {
                    cursor = document.createElement("span");
                    cursor.className = "shell-sim-cursor";
                    last.appendChild(cursor);
                }
            }

            if (CONFIG.loopDelay > 0) {
                setTimeout(function () {
                    if (!pre.isConnected) return;
                    pre.innerHTML = "";
                    runBlock(pre, steps);
                }, CONFIG.loopDelay);
            } else {
                pre.innerHTML = "";
                runBlock(pre, steps);
            }
        }

        function next() {
            if (!pre.isConnected) {
                cleanup();
                return;
            }
            if (idx >= steps.length) {
                finish();
                return;
            }
            var step = steps[idx++];

            if (step.type === "output") {
                typeOutputs(step.lines, function () {
                    setTimeout(next, CONFIG.interval * CONFIG.endWaitLetters);
                });
                return;
            }

            var lineEl = buildInputLine(step.cmd);
            pre.appendChild(lineEl);

            var wait = CONFIG.interval * CONFIG.endWaitLetters;
            if (!step.cmd) {
                setTimeout(next, wait);
                return;
            }
            typeCommand(pre, lineEl, step.cmd, 1, function () {
                setTimeout(next, wait);
            });
        }

        next();
    }

    // Static rendering for users who prefer reduced motion.
    function renderStatic(pre, steps) {
        steps.forEach(function (step) {
            if (step.type === "input") {
                var lineEl = buildInputLine(step.cmd);
                lineEl.querySelector(".shell-sim-reveal").style.width = "auto";
                lineEl.querySelector(".shell-sim-cursor").style.display =
                    "none";
                pre.appendChild(lineEl);
            } else {
                step.lines.forEach(function (text) {
                    var built = buildOutputLine(text);
                    built.el.querySelector(".shell-sim-reveal").style.width =
                        "auto";
                    pre.appendChild(built.el);
                });
            }
        });
    }

    // Renders the final state once, reads its height, then clears it. The
    // measured height is set as a min-height on the block so lines appearing
    // during the animation never push the rest of the article down.
    function measureFinalHeight(pre, steps) {
        renderStatic(pre, steps);
        var h = pre.offsetHeight;
        pre.innerHTML = "";
        if (h > 0) pre.style.minHeight = h + "px";
    }

    function transformBlock(pre) {
        var source = pre.textContent;
        var steps = parse(source);

        pre.classList.add("shell-sim");
        pre.innerHTML = "";

        // Reserve the final height before starting so the layout is stable
        // while the demo plays.
        if (steps.length) measureFinalHeight(pre, steps);

        function start() {
            if (pre.__shellSimStarted) return;
            pre.__shellSimStarted = true;
            if (reduceMotion) {
                renderStatic(pre, steps);
            } else {
                runBlock(pre, steps);
            }
        }

        if (
            reduceMotion ||
            !CONFIG.playOnView ||
            !("IntersectionObserver" in window)
        ) {
            start();
            return;
        }

        var io = new IntersectionObserver(
            function (entries) {
                entries.forEach(function (entry) {
                    if (entry.isIntersecting) {
                        io.unobserve(pre);
                        start();
                    }
                });
            },
            { threshold: 0.1 },
        );
        io.observe(pre);
    }

    /* ── styles ────────────────────────────────────────────── */

    // Colors are owned by css/light.css and css/dark.css so they switch in
    // real time with the theme; only layout is injected here. The `:where()`
    // rules are zero-specificity fallbacks for sites without those theme
    // files, so they never fight the theme rules.
    var CSS = [
        ":where(.shell-sim){background-color:#1b1410;color:#d5cdc2;",
        "border-color:#3a2e24;}",
        ".shell-sim{position:relative;}",
        ".shell-sim-line{display:block;white-space:pre;min-height:1.5em;line-height:1.55;}",
        ".shell-sim-input{white-space:nowrap;}",
        ":where(.shell-sim-prompt){color:#7d6f5f;}",
        ".shell-sim-prompt{font-weight:600;-webkit-user-select:none;user-select:none;}",
        ".shell-sim-reveal{display:inline-block;overflow:hidden;white-space:pre;",
        "vertical-align:text-bottom;width:0;}",
        ":where(.shell-sim-cursor){background-color:#d5cdc2;}",
        ".shell-sim-cursor{display:inline-block;width:.55em;height:1.05em;",
        "margin-left:1px;vertical-align:text-bottom;",
        "animation:shell-sim-blink 1s steps(1) infinite;}",
        "@keyframes shell-sim-blink{50%{opacity:0;}}",
        ":where(.shell-sim-exit){color:#e05545;}",
        ".shell-sim-exit{font-weight:600;}",
        ".shell-sim-measure{position:absolute;left:0;top:0;visibility:hidden;",
        "white-space:pre;pointer-events:none;z-index:-1;}",
    ].join("\n");

    function injectCSS() {
        if (document.getElementById("shell-sim-style")) return;
        var style = document.createElement("style");
        style.id = "shell-sim-style";
        style.textContent = CSS;
        document.head.appendChild(style);
    }

    /* ── plugin ────────────────────────────────────────────── */

    var CONFIG = getConfig();

    function plugin(hook) {
        injectCSS();
        hook.doneEach(function () {
            var blocks = document.querySelectorAll(
                'pre[data-lang="shell-simulation"], ' +
                    'pre[data-lang="bash,simulation"]',
            );
            [].forEach.call(blocks, function (pre) {
                if (pre.classList.contains("shell-sim")) return;
                transformBlock(pre);
            });
        });
    }

    if (window.$docsify) {
        window.$docsify.plugins = (window.$docsify.plugins || []).concat(
            plugin,
        );
    }
})();
