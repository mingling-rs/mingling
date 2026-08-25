/*!
 * docsify-plugin-file-tree
 *
 * Renders ```file-tree fenced blocks as a text file tree (inside the pre,
 * like the original version):
 *
 *     ```file-tree
 *     /Cargo.toml # Cargo 配置文件
 *     /Cargo.lock # Cargo 锁文件
 *     /src/main.rs # 项目主入口
 *     ```
 *
 * Paths start at the root `/`; the text after `#` is the comment shown after
 * the entry. The tree is sorted with directories first, then by name
 * (digits < uppercase < lowercase).
 */
(function () {
    "use strict";

    function escapeHtml(str) {
        return str
            .replace(/&/g, "&amp;")
            .replace(/</g, "&lt;")
            .replace(/>/g, "&gt;");
    }

    // Builds the tree from the raw block text.
    function parse(source) {
        var root = { name: "/", isDir: true, comment: "", children: [] };
        source
            .replace(/\r\n?/g, "\n")
            .split("\n")
            .forEach(function (line) {
                var text = line.trim();
                if (!text) return;

                var comment = "";
                var m = text.match(/^(.*?)\s*#\s*(.*)$/);
                if (m && m[1].trim()) {
                    text = m[1].trim();
                    comment = m[2].trim();
                }

                var parts = text.replace(/^\/+/, "").split("/").filter(Boolean);
                var node = root;
                parts.forEach(function (part, i) {
                    var last = i === parts.length - 1;
                    var child = node.children.find(function (c) {
                        return c.name === part;
                    });
                    if (!child) {
                        child = {
                            name: part,
                            isDir: false,
                            comment: "",
                            children: [],
                        };
                        node.children.push(child);
                    }
                    if (last) {
                        // A leaf is a directory when other paths descend into it.
                        child.isDir = child.children.length > 0;
                        child.comment = comment;
                    } else {
                        child.isDir = true;
                    }
                    node = child;
                });
                if (!parts.length) root.comment = comment;
            });

        sortTree(root);
        return root;
    }

    // Directories first, then name order (0-9 < A-Z < a-z).
    function sortTree(node) {
        node.children.sort(function (a, b) {
            if (a.isDir !== b.isDir) return a.isDir ? -1 : 1;
            return a.name < b.name ? -1 : a.name > b.name ? 1 : 0;
        });
        node.children.forEach(sortTree);
    }

    function commentHtml(comment) {
        return comment
            ? ' <span class="token comment"># ' +
                  escapeHtml(comment) +
                  "</span>"
            : "";
    }

    // Monospace display width: CJK counts 2 cells, box-drawing and ASCII 1.
    function charWidth(str) {
        var w = 0;
        for (var i = 0; i < str.length; i++) {
            var c = str.charCodeAt(i);
            w += c > 255 && !(c >= 0x2500 && c <= 0x257f) ? 2 : 1;
        }
        return w;
    }

    // Collects every rendered line (without its comment) so comments can be
    // aligned afterwards.
    function collectLines(node, isLast, prefix, out) {
        var conn = isLast ? "\u2514\u2500\u2500 " : "\u251c\u2500\u2500 ";
        var name = node.name + (node.isDir ? "/" : "");
        out.push({
            prefix: prefix,
            conn: conn,
            name: name,
            comment: node.comment,
            width: charWidth(prefix + conn + name),
        });
        var childPrefix = prefix + (isLast ? "    " : "\u2502   ");
        node.children.forEach(function (child, i) {
            collectLines(
                child,
                i === node.children.length - 1,
                childPrefix,
                out,
            );
        });
    }

    function transformBlock(pre) {
        if (pre.__fileTreeTransformed) return;
        pre.__fileTreeTransformed = true;

        var root = parse(pre.textContent);
        var lines = [];
        // Root line, then its children.
        lines.push({
            prefix: "",
            conn: "",
            name: root.name,
            comment: root.comment,
            width: charWidth(root.name),
        });
        root.children.forEach(function (child, i) {
            collectLines(child, i === root.children.length - 1, "", lines);
        });

        // Align comments to the rightmost comment column.
        var max = 0;
        lines.forEach(function (l) {
            if (l.comment) max = Math.max(max, l.width);
        });

        var out = lines.map(function (l) {
            var pad = l.comment ? " ".repeat(max - l.width + 2) : "";
            return (
                l.prefix +
                l.conn +
                escapeHtml(l.name) +
                pad +
                commentHtml(l.comment)
            );
        });

        pre.classList.add("file-tree");
        pre.innerHTML = "<code>" + out.join("\n") + "</code>";
    }

    function plugin(hook) {
        hook.doneEach(function () {
            var blocks = document.querySelectorAll(
                'pre[data-lang="file-tree"]',
            );
            [].forEach.call(blocks, transformBlock);
        });
    }

    if (window.$docsify) {
        window.$docsify.plugins = (window.$docsify.plugins || []).concat(
            plugin,
        );
    }
})();
