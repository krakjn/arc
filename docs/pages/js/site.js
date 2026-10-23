(function () {
  var still = matchMedia("(prefers-reduced-motion: reduce)").matches;
  var shLine = "curl -fsSL https://raw.githubusercontent.com/krakjn/arc/main/install/get_arc.sh | sh";
  var psLine = "irm https://raw.githubusercontent.com/krakjn/arc/main/install/get_arc.ps1 | iex";

  if (window.hljs) {
    document.querySelectorAll(".readout pre code").forEach(function (block) {
      window.hljs.highlightElement(block);
    });
  }

  document.querySelectorAll(".readout pre").forEach(function (pre) {
    if (!pre.querySelector("code")) return;
    var button = document.createElement("button");
    button.type = "button";
    button.className = "copy";
    button.textContent = "Copy";
    button.addEventListener("click", function () {
      var text = pre.querySelector("code").innerText;
      var done = function (label) {
        button.textContent = label;
        setTimeout(function () { button.textContent = "Copy"; }, 1400);
      };
      if (navigator.clipboard) navigator.clipboard.writeText(text).then(function () { done("Copied"); }, function () { done("Select it"); });
      else done("Select it");
    });
    pre.appendChild(button);
  });

  var links = {};
  document.querySelectorAll(".readout a[href^='#']").forEach(function (a) {
    links[decodeURIComponent(a.getAttribute("href").slice(1))] = a;
  });
  var headings = document.querySelectorAll(".readout h2[id], .readout h3[id]");
  if (headings.length && "IntersectionObserver" in window) {
    var observer = new IntersectionObserver(function (entries) {
      entries.forEach(function (entry) {
        if (!entry.isIntersecting) return;
        Object.keys(links).forEach(function (id) {
          links[id].classList.toggle("active", id === entry.target.id);
        });
      });
    }, { rootMargin: "0px 0px -70% 0px" });
    headings.forEach(function (h) { observer.observe(h); });
  }

  function assetName(tag, suffix) {
    var ver = String(tag || "").replace(/^v/, "");
    var name = "arc-" + ver + "-" + suffix;
    return suffix.indexOf("windows") === 0 ? name + ".exe" : name;
  }

  function pointFittings(tag) {
    if (!tag) return;
    document.querySelectorAll(".fitting[data-asset]").forEach(function (card) {
      var suffix = card.getAttribute("data-asset");
      var file = assetName(tag, suffix);
      card.href = "https://github.com/krakjn/arc/releases/download/" + tag + "/" + file;
      card.setAttribute("download", suffix.indexOf("windows") === 0 ? "arc.exe" : "arc");
      var code = card.querySelector("code");
      if (code) code.textContent = file;
    });
  }

  var source = document.querySelector(".source");
  if (source) {
    var fill = function (facts) {
      if (facts.tag) source.querySelector("[data-release]").textContent = facts.tag;
      if (facts.stars != null) source.querySelector("[data-stars]").textContent = facts.stars + " stars";
      if (facts.forks != null) source.querySelector("[data-forks]").textContent = facts.forks + " forks";
      pointFittings(facts.tag);
    };
    var cached = sessionStorage.getItem("arc-source");
    if (cached) fill(JSON.parse(cached));
    else {
      var api = "https://api.github.com/repos/krakjn/arc";
      var json = function (r) { return r.ok ? r.json() : {}; };
      Promise.all([fetch(api).then(json), fetch(api + "/releases/latest").then(json)]).then(function (res) {
        var facts = { stars: res[0].stargazers_count, forks: res[0].forks_count, tag: res[1].tag_name };
        fill(facts);
        sessionStorage.setItem("arc-source", JSON.stringify(facts));
      }).catch(function () {});
    }
  }

  var installCmd = document.querySelector("[data-install-cmd]");
  var installCopy = document.querySelector("[data-install-copy]");
  var installSwap = document.querySelector("[data-install-swap]");
  var installOs = document.querySelector("[data-install-os]");
  var showingWindows = /Windows/.test(navigator.userAgent);

  function showInstall(windows) {
    showingWindows = windows;
    if (installCmd) installCmd.textContent = windows ? psLine : shLine;
    if (installOs) installOs.textContent = windows ? "Windows" : "Linux / macOS";
    if (installSwap) installSwap.textContent = windows ? "Linux or macOS instead" : "Windows instead";
  }
  showInstall(showingWindows);
  if (installCopy && installCmd) {
    installCopy.addEventListener("click", function () {
      var done = function (label) {
        installCopy.textContent = label;
        setTimeout(function () { installCopy.textContent = "Copy"; }, 1400);
      };
      if (navigator.clipboard) navigator.clipboard.writeText(installCmd.textContent).then(function () { done("Copied"); }, function () { done("Select it"); });
      else done("Select it");
    });
  }
  if (installSwap) installSwap.addEventListener("click", function () { showInstall(!showingWindows); });

  var ua = navigator.userAgent;
  var platform = /Windows/.test(ua) ? "windows" : /Mac/.test(ua) ? "macos" : "linux";
  var arch = /arm64|aarch64/i.test(ua) ? "arm64" : "amd64";
  var yours = document.querySelector('.fitting[data-platform="' + platform + "-" + arch + '"]');
  if (yours) yours.classList.add("is-yours");

  var canvas = document.getElementById("table");
  if (!canvas) return;
  var ctx = canvas.getContext("2d");

  var notes = {
    status: "Height is commits ahead. An amber cap is a dirty worktree. Beams are dependencies.",
    checkout: "The dependency tree moves onto the branch. A repo without that branch stays where it is.",
    sync: "Every repo checks out its default branch and pulls. The towers settle and the dirty caps clear.",
    g: "One pass across every repo. -c runs a shell command. -g runs git.",
    config: "One arc.toml names the paths, the remotes, and who depends on whom.",
    flow: "Push a branch, open the request, then checkout carries the dependencies with it."
  };

  function modeFromPage() {
    try {
      var stored = sessionStorage.getItem("arc-table-mode");
      if (stored && notes[stored]) return stored;
    } catch (err) {}
    var path = location.pathname;
    if (path.indexOf("/config") !== -1) return "config";
    if (path.indexOf("/workflow") !== -1) return "flow";
    return "status";
  }

  var repos = [
    { path: "path/app",   x:  0.0,  z:  0.0,  ahead: 3, behind: 0, dirty: true,  branch: "main", deps: [1, 3] },
    { path: "path/web",   x: -0.62, z: -0.38, ahead: 1, behind: 2, dirty: false, branch: "main", deps: [2] },
    { path: "path/api",   x: -0.55, z:  0.55, ahead: 0, behind: 0, dirty: true,  branch: "main", deps: [] },
    { path: "path/other", x:  0.62, z: -0.42, ahead: 2, behind: 0, dirty: false, branch: "main", deps: [4] },
    { path: "path/arc",   x:  0.6,  z:  0.5,  ahead: 0, behind: 1, dirty: false, branch: "main", deps: [], noFeat: true }
  ];
  var base = repos.map(function (r) { return { ahead: r.ahead, behind: r.behind, dirty: r.dirty, branch: r.branch }; });
  repos.forEach(function (r) { r.h = 0.18 + r.ahead * 0.1; r.th = r.h; r.glow = 0; });
  var edges = [];
  repos.forEach(function (r, i) { r.deps.forEach(function (d) { edges.push({ from: i, to: d, lit: 0 }); }); });
  var checkoutOrder = [[0, 1], [0, 3], [1, 2], [3, 4]];
  var mode = "status";
  var modeStart = performance.now();

  function markNav(m) {
    document.querySelectorAll(".modes button").forEach(function (button) {
      var on = button.dataset.mode === m;
      button.setAttribute("aria-pressed", on ? "true" : "false");
    });
    var note = document.querySelector("[data-mode-note]");
    if (note) note.textContent = notes[m] || "";
  }

  function setMode(m) {
    mode = m;
    modeStart = performance.now();
    markNav(m);
    try { sessionStorage.setItem("arc-table-mode", m); } catch (err) {}
    repos.forEach(function (r, i) {
      var b = base[i];
      r.branch = b.branch; r.dirty = b.dirty; r.ahead = b.ahead; r.behind = b.behind;
      r.th = 0.18 + r.ahead * 0.1;
      r.glow = 0;
    });
    edges.forEach(function (e) { e.lit = 0; });
    if (m === "sync") repos.forEach(function (r) { r.th = 0.22; r.dirty = false; r.ahead = 0; r.behind = 0; });
  }
  setMode(modeFromPage());
  document.querySelectorAll(".modes button").forEach(function (button) {
    button.addEventListener("click", function () {
      setMode(button.getAttribute("data-mode"));
    });
  });
  document.querySelectorAll(".pages a").forEach(function (a) {
    var path = a.pathname.replace(/\/$/, "") || "/";
    var here = location.pathname.replace(/\/$/, "") || "/";
    if (path === here) a.setAttribute("aria-current", "page");
  });

  var W = 0, H = 0, S = 0, CX = 0, CY = 0;
  function size() {
    var dpr = window.devicePixelRatio || 1;
    W = canvas.clientWidth; H = canvas.clientHeight;
    if (!W || !H) return;
    canvas.width = Math.round(W * dpr);
    canvas.height = Math.round(H * dpr);
    ctx.setTransform(dpr, 0, 0, dpr, 0, 0);
    var plate = document.querySelector(".plate");
    var top = plate ? plate.offsetTop + plate.offsetHeight + 24 : 120;
    var room = Math.max(160, H - top - 40);
    S = Math.min(W * 0.42, room * 0.62);
    CX = W / 2;
    CY = top + room * 0.62;
  }
  addEventListener("resize", size);
  size();

  var yaw = 0.6, vel = still ? 0 : 0.0016, dragging = false, lastX = 0;
  canvas.addEventListener("pointerdown", function (e) {
    dragging = true; lastX = e.clientX; canvas.classList.add("drag");
    try { canvas.setPointerCapture(e.pointerId); } catch (err) {}
  });
  canvas.addEventListener("pointermove", function (e) {
    if (!dragging) return;
    var dx = e.clientX - lastX; lastX = e.clientX;
    yaw += dx * 0.008;
    vel = still ? 0 : dx * 0.0004;
  });
  function stopDrag() { dragging = false; canvas.classList.remove("drag"); }
  canvas.addEventListener("pointerup", stopDrag);
  canvas.addEventListener("pointercancel", stopDrag);

  function project(x, z, h) {
    var c = Math.cos(yaw), s = Math.sin(yaw);
    var rx = x * c - z * s;
    var rz = x * s + z * c;
    return { x: CX + rx * S, y: CY + rz * S * 0.5 - h * S, d: rz };
  }

  function poly(pts, fill, stroke) {
    ctx.beginPath();
    ctx.moveTo(pts[0].x, pts[0].y);
    for (var i = 1; i < pts.length; i++) ctx.lineTo(pts[i].x, pts[i].y);
    ctx.closePath();
    if (fill) { ctx.fillStyle = fill; ctx.fill(); }
    if (stroke) { ctx.strokeStyle = stroke; ctx.stroke(); }
  }

  function drawTable(now) {
    ctx.lineWidth = 1;
    var ring, a, t, p, g, v, len, a1, a2, b1, b2;
    for (ring = 1; ring <= 4; ring++) {
      ctx.beginPath();
      for (a = 0; a <= 64; a++) {
        t = a / 64 * Math.PI * 2;
        p = project(Math.cos(t) * ring * 0.28, Math.sin(t) * ring * 0.28, 0);
        if (a === 0) ctx.moveTo(p.x, p.y); else ctx.lineTo(p.x, p.y);
      }
      ctx.strokeStyle = "rgba(57,255,182," + (ring === 4 ? 0.45 : 0.12) + ")";
      ctx.stroke();
    }
    for (g = -5; g <= 5; g++) {
      v = g * 0.2;
      len = Math.sqrt(Math.max(0, 1.12 * 1.12 - v * v));
      a1 = project(v, -len, 0); a2 = project(v, len, 0);
      b1 = project(-len, v, 0); b2 = project(len, v, 0);
      ctx.strokeStyle = "rgba(57,255,182,0.05)";
      ctx.beginPath();
      ctx.moveTo(a1.x, a1.y); ctx.lineTo(a2.x, a2.y);
      ctx.moveTo(b1.x, b1.y); ctx.lineTo(b2.x, b2.y);
      ctx.stroke();
    }
    if (still) return;
    var sweep = (now / 3000) % (Math.PI * 2);
    ctx.beginPath();
    var o = project(0, 0, 0);
    ctx.moveTo(o.x, o.y);
    for (var k = 0; k <= 12; k++) {
      var tt = sweep - k * 0.03;
      var q = project(Math.cos(tt) * 1.12, Math.sin(tt) * 1.12, 0);
      ctx.lineTo(q.x, q.y);
    }
    ctx.closePath();
    ctx.fillStyle = "rgba(57,255,182,0.06)";
    ctx.fill();
  }

  function tower(r) {
    var w = 0.1;
    var corners = [[-w, -w], [w, -w], [w, w], [-w, w]];
    var bottom = corners.map(function (c) { return project(r.x + c[0], r.z + c[1], 0); });
    var top = corners.map(function (c) { return project(r.x + c[0], r.z + c[1], r.h); });
    var faces = [];
    for (var i = 0; i < 4; i++) {
      var j = (i + 1) % 4;
      faces.push({ pts: [bottom[i], bottom[j], top[j], top[i]], d: (bottom[i].d + bottom[j].d) / 2, shade: i });
    }
    faces.sort(function (a, b) { return a.d - b.d; });
    faces.forEach(function (f) {
      var alpha = 0.1 + (f.shade % 2) * 0.06 + r.glow * 0.25;
      poly(f.pts, "rgba(57,255,182," + alpha + ")", "rgba(57,255,182," + (0.45 + r.glow * 0.5) + ")");
    });
    poly(top, r.dirty ? "rgba(255,176,0,0.75)" : "rgba(57,255,182," + (0.28 + r.glow * 0.5) + ")", r.dirty ? "#ffb000" : "#39ffb6");
    return project(r.x, r.z, r.h);
  }

  function beam(a, b, lit) {
    var pa = project(repos[a].x, repos[a].z, repos[a].h);
    var pb = project(repos[b].x, repos[b].z, repos[b].h);
    var mx = (pa.x + pb.x) / 2, my = Math.min(pa.y, pb.y) - S * 0.18;
    ctx.lineWidth = 1.5;
    ctx.strokeStyle = "rgba(57,255,182,0.28)";
    ctx.setLineDash([4, 6]);
    ctx.beginPath(); ctx.moveTo(pa.x, pa.y); ctx.quadraticCurveTo(mx, my, pb.x, pb.y); ctx.stroke();
    ctx.setLineDash([]);
    if (lit <= 0) return;
    ctx.lineWidth = 3;
    ctx.strokeStyle = "#39ffb6";
    ctx.shadowColor = "#39ffb6"; ctx.shadowBlur = 14;
    ctx.beginPath();
    ctx.moveTo(pa.x, pa.y);
    var steps = 24;
    for (var i = 1; i <= Math.round(steps * lit); i++) {
      var t = i / steps;
      ctx.lineTo(
        (1 - t) * (1 - t) * pa.x + 2 * (1 - t) * t * mx + t * t * pb.x,
        (1 - t) * (1 - t) * pa.y + 2 * (1 - t) * t * my + t * t * pb.y
      );
    }
    ctx.stroke();
    ctx.shadowBlur = 0;
  }

  function label(p, lines, color) {
    var fs = W < 520 ? 10 : 12;
    ctx.font = "600 " + fs + 'px "JetBrains Mono", monospace';
    var w = Math.max.apply(null, lines.map(function (l) { return ctx.measureText(l).width; })) + 14;
    var h = lines.length * (fs + 4) + 8;
    var x = p.x - w / 2, y = p.y - h - 18;
    ctx.fillStyle = "rgba(2,16,15,0.82)";
    ctx.fillRect(x, y, w, h);
    ctx.strokeStyle = color; ctx.lineWidth = 1;
    ctx.strokeRect(x, y, w, h);
    ctx.beginPath(); ctx.moveTo(p.x, y + h); ctx.lineTo(p.x, p.y - 4); ctx.stroke();
    ctx.textAlign = "left";
    lines.forEach(function (l, i) {
      ctx.fillStyle = i === 0 ? "#e8fff6" : color;
      ctx.fillText(l, x + 7, y + (i + 1) * (fs + 4));
    });
  }

  function update(now) {
    var el = (now - modeStart) / 1000;
    repos.forEach(function (r) { r.h += (r.th - r.h) * (still ? 1 : 0.06); r.glow *= 0.94; });
    if (mode === "checkout") {
      checkoutOrder.forEach(function (pair, i) {
        var e = edges.filter(function (x) { return x.from === pair[0] && x.to === pair[1]; })[0];
        var t = still ? 1 : Math.max(0, Math.min(1, (el - 0.4 - i * 0.7) / 0.6));
        if (e) e.lit = t;
        if (t >= 1) {
          var target = repos[pair[1]];
          if (!target.noFeat && target.branch !== "feat") { target.branch = "feat"; target.glow = 1; }
        }
      });
      if (repos[0].branch !== "feat" && (still || el > 0.3)) { repos[0].branch = "feat"; repos[0].glow = 1; }
    }
    if (mode === "g") {
      var pos = still ? 2 : ((el % 2.6) / 2.6) * 2.6 - 1.3;
      repos.forEach(function (r) { if (Math.abs(r.x - pos) < 0.08) r.glow = 1; });
      return pos;
    }
    return null;
  }

  function drawScan(pos) {
    if (pos === null || still) return;
    var a = project(pos, -1.12, 0), b = project(pos, 1.12, 0);
    var a2 = project(pos, -1.12, 0.9), b2 = project(pos, 1.12, 0.9);
    var g = ctx.createLinearGradient(a.x, a.y, a2.x, a2.y);
    g.addColorStop(0, "rgba(57,255,182,0.35)");
    g.addColorStop(1, "rgba(57,255,182,0)");
    poly([a, b, b2, a2], g, null);
    ctx.strokeStyle = "#39ffb6"; ctx.lineWidth = 2;
    ctx.beginPath(); ctx.moveTo(a.x, a.y); ctx.lineTo(b.x, b.y); ctx.stroke();
  }

  function frame(now) {
    if (!W) size();
    if (!dragging) { yaw += vel; if (!still) vel += (0.0016 - vel) * 0.02; }
    var scan = update(now);
    ctx.clearRect(0, 0, W, H);
    drawTable(now);
    var order = repos.map(function (r, i) { return { i: i, d: project(r.x, r.z, 0).d }; });
    order.sort(function (a, b) { return a.d - b.d; });
    var tops = [];
    order.forEach(function (o) { tops[o.i] = tower(repos[o.i]); });
    if (mode === "checkout" || mode === "config" || mode === "flow") edges.forEach(function (e) { beam(e.from, e.to, e.lit); });
    drawScan(scan);
    order.forEach(function (o) {
      var r = repos[o.i];
      var p = tops[o.i];
      if (mode === "status" || mode === "sync") {
        var meta = r.branch + (r.ahead ? "  ↑" + r.ahead : "") + (r.behind ? " ↓" + r.behind : "") + (r.dirty ? "  *" : (mode === "sync" ? "  clean" : ""));
        label(p, [r.path, meta], r.dirty ? "#ffb000" : "#39ffb6");
      } else if (mode === "checkout") {
        var stay = r.noFeat && edges.some(function (e) { return e.to === o.i && e.lit >= 1; });
        label(p, [r.path, r.branch + (stay ? "  (no feat, stays)" : "")], r.branch === "feat" ? "#39ffb6" : "#7fb3a3");
      } else if (mode === "g") {
        label(p, [r.path, r.glow > 0.5 ? "git fetch --prune" : "·"], "#39ffb6");
      } else {
        label(p, [r.path], "#39ffb6");
      }
    });
    requestAnimationFrame(frame);
  }

  if (document.fonts && document.fonts.ready) document.fonts.ready.then(size);
  requestAnimationFrame(frame);

  var clock = document.getElementById("clock");
  if (clock) {
    setInterval(function () {
      clock.textContent = "TABLE ONLINE · " + new Date().toISOString().slice(11, 19) + "Z";
    }, 1000);
  }
})();
