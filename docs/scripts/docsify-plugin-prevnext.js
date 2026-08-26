/*!
 * docsify-plugin-prevnext
 * Auto-generates previous/next page navigation based on sidebar order.
 */
(function () {
    function install(hook, vm) {
        // auto2top only applies to "navigate"-source clicks, not direct hash
        // changes, so after a prev/next jump the target page is scrolled back to
        // the top once it has rendered.
        var pendingTop = false;

        hook.doneEach(function () {
            if (pendingTop) {
                pendingTop = false;
                window.scrollTo(0, 0);
            }

            var nav = document.querySelector(".page-nav");
            if (nav) nav.remove();

            function navigate(path) {
                pendingTop = true;
                window.location.hash = "#/" + (path || "");
                window.scrollTo(0, 0);
            }

            var sidebarLinks = document.querySelectorAll(".sidebar-nav li a");
            if (!sidebarLinks.length) return;

            var currentPath = vm.route.path;
            var links = [];
            sidebarLinks.forEach(function (a) {
                var href = a.getAttribute("href");
                if (!href) return;
                if (href.startsWith("http")) return;
                if (href.indexOf("?id=") !== -1) return; // skip sub-headings
                var path = href.startsWith("#") ? href.slice(1) : href;
                if (path.startsWith("/")) path = path.slice(1);
                if (path === "" || path === "/") path = "README";
                links.push({ path: path, text: a.textContent.trim() });
            });

            function normalize(p) {
                return p.replace(/\.md$/g, "").replace(/^\/+|\/+$/g, "");
            }

            // Treat root path as README
            var cur = normalize(currentPath) || "README";
            var idx = -1;
            for (var i = 0; i < links.length; i++) {
                if (normalize(links[i].path) === cur) {
                    idx = i;
                    break;
                }
            }

            if (idx === -1) return;

            var prevLink = idx > 0 ? links[idx - 1] : null;
            var nextLink = idx < links.length - 1 ? links[idx + 1] : null;

            // Build nav HTML
            var html = "";
            if (prevLink) {
                html +=
                    '<a class="nav-prev" href="javascript:;" data-nav="' +
                    encodeURIComponent(prevLink.path) +
                    '">' +
                    '<span><div class="nav-label">‹ Previous</div><div class="nav-title">' +
                    prevLink.text +
                    "</div></span></a>";
            }
            if (nextLink) {
                html +=
                    '<a class="nav-next" href="javascript:;" data-nav="' +
                    encodeURIComponent(nextLink.path) +
                    '">' +
                    '<span><div class="nav-label">Next ›</div><div class="nav-title">' +
                    nextLink.text +
                    "</div></span></a>";
            }

            if (!html) return;

            nav = document.createElement("div");
            nav.className = "page-nav";
            nav.innerHTML = html;

            // Attach click handlers via event delegation
            nav.addEventListener("click", function (e) {
                var target = e.target.closest("a");
                if (!target) return;
                var path = target.getAttribute("data-nav");
                if (path) {
                    e.preventDefault();
                    navigate(decodeURIComponent(path));
                }
            });

            var section = document.querySelector(".markdown-section");
            if (section) section.appendChild(nav);
        });
    }

    window.$docsify = window.$docsify || {};
    window.$docsify.plugins = (window.$docsify.plugins || []).concat(install);
})();
