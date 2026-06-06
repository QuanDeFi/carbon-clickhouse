#!/usr/bin/env node
const fs = require("fs");
const path = require("path");
const { pathToFileURL } = require("url");
const { createRequire } = require("module");
const { execFileSync } = require("child_process");

const demoRequire = createRequire(path.join(__dirname, "playwright", "package.json"));
const { chromium } = demoRequire("@playwright/test");

const ROOT = path.resolve(__dirname, "../..");
const SCREENSHOT_DIR = path.join(ROOT, "demo-artifacts", "screenshots");
const REPORT_DIR = path.join(ROOT, "demo-artifacts", "review-human", "logs");
const PID_DIR = path.join(ROOT, "demo-artifacts", "pids");
const TRANSITION_DIR = path.join(ROOT, "demo-artifacts", "window-transitions");
const CHROME_ARGS = [
  "--no-first-run",
  "--no-default-browser-check",
  "--no-sandbox",
  "--disable-gpu",
  "--disable-dev-shm-usage",
  "--disable-extensions",
  "--disable-infobars",
  "--disable-backgrounding-occluded-windows",
  "--disable-renderer-backgrounding",
  "--default-background-color=000000",
  "--test-type",
  "--force-dark-mode",
  "--enable-features=WebUIDarkMode",
  "--disable-save-password-bubble",
  "--disable-password-generation",
  "--password-store=basic",
  "--disable-features=AutofillServerCommunication,PasswordLeakDetection,PasswordManagerOnboarding,PasswordManagerRedesign,PasswordCheck,CalculateNativeWinOcclusion",
  "--ozone-platform=x11",
];
const GRAFANA_USER = process.env.GRAFANA_DEMO_USER || "admin";
const GRAFANA_PASSWORD = process.env.GRAFANA_DEMO_PASSWORD || "carbon";
const CLICKHOUSE_HTTP_USER = process.env.CLICKHOUSE_DEMO_USER || "carbon";
const CLICKHOUSE_HTTP_PASSWORD = process.env.CLICKHOUSE_DEMO_PASSWORD || "carbon";
let workflowTimelineStartedAt = 0;
let workflowTimelineEvents = [];
const GRAFANA_DASHBOARDS = {
  overview: {
    uid: "carbon-clickhouse-overview",
    slug: "carbon-clickhouse-overview",
    title: /Carbon ClickHouse Overview|Carbon Updates|ClickHouse Buffered Rows/i,
  },
  jupiter: {
    uid: "carbon-clickhouse-jupiter",
    slug: "carbon-clickhouse-jupiter",
    title: /Carbon ClickHouse Jupiter|Carbon Updates|ClickHouse Buffered Rows/i,
  },
  token: {
    uid: "carbon-clickhouse-token-program",
    slug: "carbon-clickhouse-token-program",
    title: /Carbon ClickHouse Token Program|Carbon Updates|ClickHouse Buffered Rows/i,
  },
};

const DARK_BROWSER_CSS = `
  :root {
    color-scheme: dark !important;
    --demo-bg: #080b0f;
    --demo-panel: #111821;
    --demo-panel-2: #17212b;
    --demo-border: #2b3a46;
    --demo-text: #e8edf2;
    --demo-muted: #a9b4bf;
    --demo-accent: #faff69;
    --demo-blue: #65d4ff;
    --demo-green: #63e6be;
  }
  html, body, #root, [data-testid="root"], .ant-layout, .ant-layout-content {
    background: var(--demo-bg) !important;
    color: var(--demo-text) !important;
  }
  body {
    background: var(--demo-bg) !important;
  }
  main, section, article, form, table, thead, tbody, tr,
  .ant-card, .ant-modal-content, .ant-drawer-content, .ant-popover-inner,
  .ant-table, .ant-table-container, .ant-table-content, .ant-table-thead > tr > th,
  .ant-table-tbody > tr > td, .ant-tabs-content-holder, .ant-select-dropdown,
  .ant-picker-panel-container, .ant-menu, .ant-layout-sider,
  [class*="card"], [class*="panel"], [class*="container"], [class*="surface"] {
    background-color: var(--demo-panel) !important;
    color: var(--demo-text) !important;
    border-color: var(--demo-border) !important;
  }
  div, span, p, label, li, td, th, h1, h2, h3, h4, h5, h6, strong, small {
    color: inherit;
  }
  input, textarea, select,
  .ant-input, .ant-select-selector, .ant-picker, .cm-editor, .cm-scroller, .cm-gutters,
  [contenteditable="true"], [role="textbox"] {
    background: #0d131a !important;
    color: var(--demo-text) !important;
    border-color: var(--demo-border) !important;
  }
  textarea, pre, code, .cm-line, .cm-content {
    color: #d9f99d !important;
    caret-color: var(--demo-accent) !important;
  }
  button, .ant-btn {
    background: #18222d !important;
    color: var(--demo-text) !important;
    border-color: #3c4d5c !important;
  }
  button:hover, .ant-btn:hover {
    color: var(--demo-accent) !important;
    border-color: var(--demo-accent) !important;
  }
  a, .ant-tabs-tab-active, .ant-tabs-tab-active * {
    color: var(--demo-blue) !important;
  }
  svg, canvas {
    color-scheme: dark !important;
  }
  ::selection {
    background: rgba(250, 255, 105, 0.28) !important;
    color: var(--demo-text) !important;
  }
`;

function size() {
  const raw = process.env.DEMO_SCREEN_SIZE || "1920x1080";
  const [width, height] = raw.split("x").map((part) => Number(part));
  return { width, height };
}

function singleFrame() {
  const { width, height } = size();
  const margin = Number(process.env.DEMO_WINDOW_MARGIN || "60");
  const frameWidth = Number(process.env.DEMO_WINDOW_WIDTH || String(width - margin * 2));
  const frameHeight = Number(process.env.DEMO_WINDOW_HEIGHT || String(height - margin * 2));
  return {
    x: Math.max(0, Math.floor((width - frameWidth) / 2)),
    y: Math.max(0, Math.floor((height - frameHeight) / 2)),
    width: Math.min(frameWidth, width),
    height: Math.min(frameHeight, height),
  };
}

function overviewFrames(count) {
  const { width, height } = size();
  const margin = Number(process.env.DEMO_OVERVIEW_MARGIN || "80");
  const gap = Number(process.env.DEMO_OVERVIEW_GAP || "42");
  const cols = count <= 4 ? 2 : 3;
  const rows = Math.max(1, Math.ceil(count / cols));
  const frameWidth = Math.floor((width - margin * 2 - gap * (cols - 1)) / cols);
  const frameHeight = Math.floor((height - margin * 2 - gap * (rows - 1)) / rows);
  return Array.from({ length: count }, (_, index) => ({
    x: margin + (index % cols) * (frameWidth + gap),
    y: margin + Math.floor(index / cols) * (frameHeight + gap),
    width: frameWidth,
    height: frameHeight,
  }));
}

function sh(command, args, options = {}) {
  return execFileSync(command, args, {
    cwd: ROOT,
    env: process.env,
    encoding: "utf8",
    stdio: options.stdio || ["ignore", "pipe", "ignore"],
  });
}

function clickHouseTsv(query) {
  return sh("curl", [
    "-fsS",
    "-u",
    "carbon:carbon",
    "--data-binary",
    `${query} FORMAT TSV`,
    "http://localhost:8123/",
  ]).trim();
}

function clickHouseTableRows(tableName) {
  const safeName = String(tableName).replace(/'/g, "''");
  const value = clickHouseTsv(
    `SELECT total_rows FROM system.tables WHERE database = 'default' AND name = '${safeName}' LIMIT 1`,
  );
  return value ? Number(value) : 0;
}

function isGrafanaUrl(url) {
  return String(url || "").includes("localhost:3000");
}

function isClickStackUrl(url) {
  return String(url || "").includes("/clickstack");
}

function grafanaDashboardUrl(dashboardKey = "overview") {
  const dashboard = GRAFANA_DASHBOARDS[dashboardKey] || GRAFANA_DASHBOARDS.overview;
  return `http://localhost:3000/d/${dashboard.uid}/${dashboard.slug}?orgId=1&from=now-15m&to=now&refresh=1s&theme=dark`;
}

async function seedGrafanaSession(context) {
  try {
    await context.request.post("http://localhost:3000/login", {
      data: {
        user: GRAFANA_USER,
        password: GRAFANA_PASSWORD,
      },
    });
  } catch (_) {
    // The dashboard function still checks for a login page and fails clearly if
    // Grafana did not accept the session seed.
  }
}

function hashSeed(text) {
  let hash = 2166136261;
  for (const char of String(text)) {
    hash ^= char.charCodeAt(0);
    hash = Math.imul(hash, 16777619);
  }
  return hash >>> 0;
}

function seededRng(seedText) {
  let state = hashSeed(`${process.env.DEMO_TYPING_SEED || "carbon-clickhouse-tutorial"}:${seedText}`) || 1;
  return () => {
    state |= 0;
    state = (state + 0x6d2b79f5) | 0;
    let value = Math.imul(state ^ (state >>> 15), 1 | state);
    value = (value + Math.imul(value ^ (value >>> 7), 61 | value)) ^ value;
    return ((value ^ (value >>> 14)) >>> 0) / 4294967296;
  };
}

async function humanPause(page, ms) {
  await page.waitForTimeout(Math.max(0, Math.round(ms)));
}

function moveWindow(windowId, frame) {
  execFileSync("wmctrl", ["-ir", windowId, "-e", `0,${frame.x},${frame.y},${frame.width},${frame.height}`], {
    cwd: ROOT,
    stdio: "ignore",
    env: process.env,
  });
  // The Xvfb display may not be backed by an EWMH-compliant window manager.
  // wmctrl is useful when available, but xdotool directly enforces the pixel
  // frame so browser windows keep the same edge gaps as terminal windows.
  execFileSync("xdotool", ["windowmove", windowId, String(frame.x), String(frame.y)], {
    cwd: ROOT,
    stdio: "ignore",
    env: process.env,
  });
  execFileSync("xdotool", ["windowsize", windowId, String(frame.width), String(frame.height)], {
    cwd: ROOT,
    stdio: "ignore",
    env: process.env,
  });
}

function tryWindowCommand(args) {
  try {
    execFileSync("xdotool", args, { cwd: ROOT, env: process.env, stdio: "ignore" });
  } catch (_) {
    // Some Xvfb/window-manager combinations do not support every xdotool
    // window operation. These are visual polish commands; recording should not
    // abort if lower/raise/focus fails once the target screenshot exists.
  }
}

function windowGeometry(windowId) {
  try {
    const out = sh("xdotool", ["getwindowgeometry", "--shell", windowId]);
    const values = Object.fromEntries(
      out
        .trim()
        .split("\n")
        .map((line) => line.split("="))
        .filter((parts) => parts.length === 2),
    );
    return {
      x: Number(values.X),
      y: Number(values.Y),
      width: Number(values.WIDTH),
      height: Number(values.HEIGHT),
    };
  } catch (_) {
    return null;
  }
}

async function animateWindow(windowId, target, steps = 8) {
  const start = windowGeometry(windowId);
  if (!start) {
    moveWindow(windowId, target);
    await pause(0.2);
    return;
  }
  for (let step = 1; step <= steps; step += 1) {
    const t = step / steps;
    const eased = 1 - (1 - t) * (1 - t);
    moveWindow(windowId, {
      x: Math.round(start.x + (target.x - start.x) * eased),
      y: Math.round(start.y + (target.y - start.y) * eased),
      width: Math.round(start.width + (target.width - start.width) * eased),
      height: Math.round(start.height + (target.height - start.height) * eased),
    });
    await pause(0.035);
  }
}

function demoWindows() {
  try {
    return sh("wmctrl", ["-l"])
      .split("\n")
      .map((line) => line.trim().split(/\s+/, 4))
      .filter((parts) => parts.length >= 4)
      .map((parts) => ({ id: parts[0], title: parts[3] }))
      .filter((win) => /Carbon ClickHouse|ClickHouse|Grafana|Chromium/i.test(win.title));
  } catch (_) {
    return [];
  }
}

function latestChromiumWindow() {
  try {
    const ids = sh("xdotool", ["search", "--class", "chrom"])
      .split("\n")
      .map((line) => line.trim())
      .filter((line) => /^\d+$/.test(line));
    return ids[ids.length - 1] || null;
  } catch (_) {
    return null;
  }
}

function chromiumWindowForLabel(label) {
  const normalized = String(label || "").toLowerCase();
  const patterns = [];
  if (normalized.includes("clickstack")) {
    patterns.push(/ClickHouse Dashboard/i, /ClickStack/i);
  } else if (normalized.includes("grafana")) {
    patterns.push(/Grafana/i, /Jupiter/i, /Token Program/i);
  } else if (normalized.includes("play")) {
    patterns.push(/ClickHouse Play/i, /localhost:8123/i);
  } else if (normalized.includes("blackout")) {
    patterns.push(/Carbon Demo Blackout/i);
  }
  try {
    const ids = sh("xdotool", ["search", "--class", "chrom"])
      .split("\n")
      .map((line) => line.trim())
      .filter((line) => /^\d+$/.test(line));
    for (const windowId of ids.slice().reverse()) {
      const title = sh("xdotool", ["getwindowname", windowId]).trim();
      if (patterns.some((pattern) => pattern.test(title))) {
        return windowId;
      }
    }
    return ids[ids.length - 1] || null;
  } catch (_) {
    return null;
  }
}

async function showWindowOverview(targetWindowId, label = "browser", targetImage = null) {
  if ((process.env.DEMO_WINDOW_OVERVIEW || "true").toLowerCase() === "false") {
    return;
  }
  if (!label && targetWindowId) {
    try {
      label = sh("xdotool", ["getwindowname", targetWindowId]).trim() || "browser";
    } catch (_) {
      label = "browser";
    }
  }
  const env = { ...process.env };
  if (targetImage) {
    env.DEMO_WINDOW_TRANSITION_TARGET_IMAGE = targetImage;
  }
  if (targetWindowId) {
    // Let the transition place the real target window behind its overlay so it
    // is revealed seamlessly; the caller no longer moves it on-screen first.
    env.DEMO_WINDOW_TRANSITION_TARGET_WINDOW_ID = String(targetWindowId);
  }
  execFileSync(process.env.PYTHON || path.join(ROOT, ".venv-demo", "bin", "python"), ["scripts/demo/window_transition.py", label], {
    cwd: ROOT,
    env,
    stdio: "ignore",
  });
  if (targetWindowId) {
    execFileSync("xdotool", ["windowfocus", targetWindowId], { cwd: ROOT, env: process.env, stdio: "ignore" });
  }
}

function setTransitionCurrent(label, imagePath) {
  const slot = transitionSlot(label);
  if (slot === null || !imagePath || !fs.existsSync(imagePath)) {
    return;
  }
  fs.mkdirSync(TRANSITION_DIR, { recursive: true });
  fs.copyFileSync(imagePath, path.join(TRANSITION_DIR, `slot-${slot}.png`));
  fs.copyFileSync(imagePath, path.join(TRANSITION_DIR, "previous.png"));
  fs.writeFileSync(path.join(TRANSITION_DIR, "state.json"), JSON.stringify({ slot, label }, null, 2));
}


function hideMouse() {
  const { width, height } = size();
  try {
    execFileSync("xdotool", ["mousemove", String(width - 2), String(height - 2)], { cwd: ROOT, stdio: "ignore", env: process.env });
  } catch (_) {
    // Mouse hiding is visual polish. Do not fail the browser workflow on it.
  }
}

function writeBlackoutCoverHtml(file) {
  const { width, height } = size();
  fs.writeFileSync(
    file,
    `<!doctype html>
<html>
<head>
<meta charset="utf-8">
<title>Carbon Demo Blackout</title>
<style>
html, body {
  margin: 0;
  width: ${width}px;
  height: ${height}px;
  overflow: hidden;
  background: #000;
  cursor: none;
}
</style>
</head>
<body></body>
</html>
`,
  );
}

async function openBlackoutCover(sceneId, label) {
  const display = process.env.DISPLAY || process.env.DEMO_DISPLAY || ":96";
  const { width, height } = size();
  const profile = path.join(ROOT, "demo-artifacts", "browser-profiles", `blackout-${sceneId}-${Date.now()}`);
  const html = path.join(PID_DIR, `${sceneId}-${Date.now()}-${label}-blackout.html`);
  fs.rmSync(profile, { recursive: true, force: true });
  fs.mkdirSync(profile, { recursive: true });
  fs.mkdirSync(PID_DIR, { recursive: true });
  writeBlackoutCoverHtml(html);
  const context = await chromium.launchPersistentContext(profile, {
    headless: false,
    viewport: { width, height },
    screen: { width, height },
    colorScheme: "dark",
    args: [
      ...CHROME_ARGS,
      "--window-position=-32000,-32000",
      `--window-size=${width},${height}`,
      `--app=${pathToFileURL(html).href}`,
    ],
    env: { ...process.env, DISPLAY: display },
  });
  const page = context.pages()[0] || (await context.newPage());
  await page.waitForLoadState("domcontentloaded").catch(() => {});
  await page.waitForTimeout(250);
  const windowId = chromiumWindowForLabel("blackout") || latestChromiumWindow();
  if (windowId) {
    moveWindow(windowId, { x: 0, y: 0, width, height });
    tryWindowCommand(["windowraise", windowId]);
    tryWindowCommand(["windowfocus", windowId]);
  }
  hideMouse();
  return { browser: context, windowId };
}

async function closeBlackoutCover(cover) {
  if (!cover || !cover.browser) {
    return;
  }
  await closeBrowser(cover.browser, null);
}

function ensureGrafanaDemoPassword() {
  if ((process.env.DEMO_SKIP_GRAFANA_PASSWORD_RESET || "false").toLowerCase() === "true") {
    return;
  }
  for (const args of [
    ["exec", "carbon-grafana", "grafana", "cli", "admin", "reset-admin-password", GRAFANA_PASSWORD],
    ["exec", "carbon-grafana", "grafana-cli", "admin", "reset-admin-password", GRAFANA_PASSWORD],
  ]) {
    try {
      execFileSync("docker", args, { cwd: ROOT, env: process.env, stdio: "ignore" });
      return;
    } catch (_) {
      // Try the older grafana-cli binary name before giving up.
    }
  }
  throw new Error("Unable to reset local Grafana demo password before recording");
}

function reloadGrafanaDashboards() {
  try {
    execFileSync(
      "curl",
      [
        "-fsS",
        "-u",
        `${GRAFANA_USER}:${GRAFANA_PASSWORD}`,
        "-X",
        "POST",
        "http://localhost:3000/api/admin/provisioning/dashboards/reload",
      ],
      { cwd: ROOT, env: process.env, stdio: "ignore" },
    );
  } catch (_) {
    // Grafana also polls provisioned dashboard files. Treat explicit reload as
    // a best-effort freshness nudge, not a hard dependency for recording.
  }
}

async function openBrowser(sceneId, initialUrl, transitionLabel = "browser") {
  fs.mkdirSync(REPORT_DIR, { recursive: true });
  const display = process.env.DISPLAY || process.env.DEMO_DISPLAY || ":96";
  if (process.env.DEMO_BROWSER_DEBUG === "true") {
    console.log(`browser workflow display: ${display}`);
  }
  const frame = singleFrame();
  const prepareOffscreen = (process.env.DEMO_BROWSER_PREPARE_OFFSCREEN || "true").toLowerCase() !== "false";
  const initialX = prepareOffscreen ? -32000 : frame.x;
  const initialY = prepareOffscreen ? -32000 : frame.y;
  const profile = path.join(ROOT, "demo-artifacts", "browser-profiles", `${sceneId}-${Date.now()}`);
  const useCustomDarkTheme = !isGrafanaUrl(initialUrl) && !isClickStackUrl(initialUrl);
  fs.rmSync(profile, { recursive: true, force: true });
  fs.mkdirSync(path.join(profile, "Default"), { recursive: true });
  fs.writeFileSync(
    path.join(profile, "Default", "Preferences"),
    JSON.stringify(
      {
        credentials_enable_service: false,
        profile: {
          password_manager_enabled: false,
          password_manager_leak_detection: false,
        },
        safebrowsing: {
          enabled: false,
          enhanced: false,
        },
      },
      null,
      2,
    ),
  );
  // Grafana and ClickStack are natively dark; Chrome's force-dark filter can
  // re-invert them during navigation. Only force-dark the ClickHouse Play UI.
  const darkForcingArgs = ["--force-dark-mode", "--enable-features=WebUIDarkMode"];
  const launchArgs = useCustomDarkTheme
    ? CHROME_ARGS
    : CHROME_ARGS.filter((arg) => !darkForcingArgs.includes(arg));
  const needsClickHouseHttpAuth = isClickStackUrl(initialUrl) || initialUrl.includes("/play");
  const context = await chromium.launchPersistentContext(profile, {
    headless: false,
    viewport: { width: frame.width, height: frame.height },
    screen: size(),
    colorScheme: "dark",
    httpCredentials: needsClickHouseHttpAuth
      ? {
          username: CLICKHOUSE_HTTP_USER,
          password: CLICKHOUSE_HTTP_PASSWORD,
        }
      : undefined,
    args: [
      ...launchArgs,
      `--window-position=${initialX},${initialY}`,
      `--window-size=${frame.width},${frame.height}`,
      "--app=data:text/html,<html style='background:%23030507'><body></body></html>",
    ],
    env: { ...process.env, DISPLAY: display },
  });
  const page = context.pages()[0] || (await context.newPage());
  if (isGrafanaUrl(initialUrl)) {
    await seedGrafanaSession(context);
  }
  if (isClickStackUrl(initialUrl)) {
    await context
      .addInitScript(() => {
        const installDarkBootstrap = () => {
          document.documentElement.style.setProperty("background", "#05070a", "important");
          document.documentElement.style.setProperty("color-scheme", "dark", "important");
          if (document.body) {
            document.body.style.setProperty("background", "#05070a", "important");
          }
          if (!document.getElementById("demo-clickstack-dark-bootstrap")) {
            const style = document.createElement("style");
            style.id = "demo-clickstack-dark-bootstrap";
            style.textContent = `
              html, body, #root {
                background: #05070a !important;
                color-scheme: dark !important;
              }
            `;
            document.documentElement.appendChild(style);
          }
        };
        installDarkBootstrap();
        new MutationObserver(installDarkBootstrap).observe(document.documentElement, {
          childList: true,
          subtree: true,
        });
        sessionStorage.setItem(
          "connections",
          JSON.stringify([
            {
              id: "local",
              name: "Local ClickHouse",
              host: "http://localhost:8123",
              username: "carbon",
              password: "carbon",
              hyperdxSettingPrefix: null,
            },
          ]),
        );
      })
      .catch(() => {});
  }
  if (useCustomDarkTheme) {
    await context.addInitScript((css) => {
      const style = document.createElement("style");
      style.id = "demo-dark-theme-bootstrap";
      style.textContent = css;
      document.documentElement.appendChild(style);
      const hideClickStackBanner = () => {
        const re = /not recommended for production use/i;
        for (const element of Array.from(document.querySelectorAll("body *"))) {
          if (!(element instanceof HTMLElement)) continue;
          // Match the smallest element that still contains the whole phrase so
          // we start from the banner's own line, not an outer layout wrapper.
          if (!re.test(element.textContent || "")) continue;
          if (Array.from(element.children).some((c) => re.test(c.textContent || ""))) continue;
          // Walk up to the top warning bar: pinned near the top, spanning most
          // of the width, and not tall (so we never hide the whole app body).
          let bar = element;
          for (let depth = 0; bar.parentElement && depth < 8; depth += 1) {
            const box = bar.getBoundingClientRect();
            if (box.top < 96 && box.width > window.innerWidth * 0.6 && box.height > 0 && box.height < 240) {
              break;
            }
            bar = bar.parentElement;
          }
          if (bar instanceof HTMLElement) {
            bar.style.setProperty("display", "none", "important");
            bar.style.setProperty("visibility", "hidden", "important");
          }
        }
      };
      hideClickStackBanner();
      new MutationObserver(hideClickStackBanner).observe(document.documentElement, { childList: true, subtree: true });
      // React can re-mount the banner after our display:none; re-hide on a slow
      // interval so it never lingers in the recording.
      setInterval(hideClickStackBanner, 400);
    }, DARK_BROWSER_CSS).catch(() => {});
  }
  await page.goto(initialUrl, { waitUntil: "domcontentloaded" });
  if (useCustomDarkTheme) {
    await applyDarkTheme(page);
  }
  if (isGrafanaUrl(initialUrl)) {
    await page.waitForTimeout(2500);
    await dismissGrafanaPasswordModal(page).catch(() => {});
  } else if (isClickStackUrl(initialUrl)) {
    await page.waitForTimeout(1000);
    await completeClickStackConnection(page).catch(() => {});
    await dismissClickStackBanner(page).catch(() => {});
  } else {
    await page.waitForTimeout(500);
  }
  const windowId = chromiumWindowForLabel(transitionLabel) || latestChromiumWindow();
  if (windowId) {
    await applyGrafanaZoom(page, windowId).catch(() => {});
    if (prepareOffscreen) {
      const transitionImage = path.join(PID_DIR, `${sceneId}-${Date.now()}-browser-target.png`);
      fs.mkdirSync(PID_DIR, { recursive: true });
      await page.screenshot({ path: transitionImage, fullPage: false });
      // Do NOT move the window on-screen here. Previously moveWindow()+lower
      // briefly exposed the target (and the terminal behind it) before the
      // overlay painted. The transition now places the off-screen window at the
      // frame behind its overlay, so it only becomes visible on the clean
      // reveal once the overlay finishes expanding.
      await showWindowOverview(windowId, transitionLabel, transitionImage);
      tryWindowCommand(["windowfocus", windowId]);
      tryWindowCommand(["windowraise", windowId]);
    } else {
      moveWindow(windowId, frame);
      tryWindowCommand(["windowfocus", windowId]);
      await page.waitForTimeout(300);
      await showWindowOverview(windowId, transitionLabel);
    }
  }
  hideMouse();
  return { browser: context, page, child: null, windowId };
}

async function applyDarkTheme(page) {
  if (isGrafanaUrl(page.url()) || isClickStackUrl(page.url())) {
    return;
  }
  await page.emulateMedia({ colorScheme: "dark" }).catch(() => {});
  await page.addStyleTag({ content: DARK_BROWSER_CSS }).catch(() => {});
  const banner = page.getByText(/not recommended for production use/i).first();
  if (await banner.isVisible({ timeout: 200 }).catch(() => false)) {
    await banner.evaluate((el) => {
      let node = el;
      for (let depth = 0; node && depth < 5; depth += 1) {
        const box = node instanceof HTMLElement ? node.getBoundingClientRect() : null;
        if (node instanceof HTMLElement && box && box.width > window.innerWidth * 0.5 && box.height < 140) {
          node.style.setProperty("display", "none", "important");
          return;
        }
        node = node.parentElement;
      }
      if (el instanceof HTMLElement) {
        el.style.setProperty("display", "none", "important");
      }
    });
  }
}

async function dismissClickStackBanner(page) {
  // The "not recommended for production use" bar is removed by clicking its X.
  // That X is a Mantine <button> whose only content is an svg.tabler-icon-x
  // (no text, no aria-label) and it is a *sibling* of the warning text, so it
  // must be matched by the icon and found on a shared ancestor -- not by text
  // inside the <p>. Hiding via CSS is kept only as a fallback.
  const dismissed = await page
    .evaluate(() => {
      const re = /not recommended for production use/i;
      const leaf = Array.from(document.querySelectorAll("body *"))
        .filter((el) => re.test(el.textContent || "") && !Array.from(el.children).some((c) => re.test(c.textContent || "")))
        .pop();
      if (!leaf) return false;
      const isCloseButton = (el) =>
        !!el.querySelector('svg[class*="tabler-icon-x"]') ||
        /close|dismiss/i.test(el.getAttribute("aria-label") || "") ||
        /^\s*[×✕✖x]\s*$/i.test((el.textContent || "").trim());
      // Climb to the nearest ancestor that actually contains the close control.
      let node = leaf;
      for (let depth = 0; node && depth < 10; depth += 1) {
        const btn = Array.from(node.querySelectorAll('button, [role="button"]')).find(isCloseButton);
        if (btn instanceof HTMLElement) {
          btn.click();
          return true;
        }
        node = node.parentElement;
      }
      // Fallback: hide the whole top warning bar.
      let bar = leaf;
      for (let depth = 0; bar?.parentElement && depth < 8; depth += 1) {
        const box = bar.getBoundingClientRect();
        if (box.top < 96 && box.width > window.innerWidth * 0.6 && box.height > 0 && box.height < 240) break;
        bar = bar.parentElement;
      }
      if (bar instanceof HTMLElement) {
        bar.style.setProperty("display", "none", "important");
        bar.style.setProperty("visibility", "hidden", "important");
      }
      return false;
    })
    .catch(() => false);
  if (dismissed) {
    await page.waitForTimeout(250);
  }
  await applyDarkTheme(page);
}

async function closeBrowser(browser, child) {
  if (browser) {
    await browser.close().catch(() => {});
  }
  if (!child || child.killed) {
    return;
  }
  child.kill("SIGTERM");
  for (let attempt = 0; attempt < 30; attempt += 1) {
    if (child.exitCode !== null || child.signalCode !== null) {
      return;
    }
    await new Promise((resolve) => setTimeout(resolve, 100));
  }
  child.kill("SIGKILL");
}

async function shot(page, sceneId, label) {
  fs.mkdirSync(SCREENSHOT_DIR, { recursive: true });
  const file = path.join(SCREENSHOT_DIR, `${sceneId}-${label}.png`);
  await page.screenshot({ path: file, fullPage: false });
  hideMouse();
  return file;
}

function transitionSlot(label) {
  const normalized = String(label || "").toLowerCase();
  if (normalized.includes("grafana")) return 2;
  if (normalized.includes("clickstack")) return 3;
  if (normalized.includes("async")) return 5;
  if (normalized.includes("play") || normalized.includes("jupiter") || normalized.includes("token")) return 4;
  return null;
}

function updateTransitionSlot(label, imagePath) {
  const slot = transitionSlot(label);
  if (slot === null || !imagePath || !fs.existsSync(imagePath)) {
    return;
  }
  fs.mkdirSync(TRANSITION_DIR, { recursive: true });
  fs.copyFileSync(imagePath, path.join(TRANSITION_DIR, `slot-${slot}.png`));
}

async function pause(seconds) {
  await new Promise((resolve) => setTimeout(resolve, seconds * 1000));
  hideMouse();
}

function signalBrowserDone() {
  const doneFile = process.env.DEMO_BROWSER_DONE_FILE;
  if (!doneFile) {
    return;
  }
  fs.mkdirSync(path.dirname(doneFile), { recursive: true });
  fs.writeFileSync(doneFile, `${Date.now()}\n`);
}

async function waitForCleanupSignal() {
  const cleanupFile = process.env.DEMO_BROWSER_CLEANUP_FILE;
  if (!cleanupFile) {
    return;
  }
  const deadline = Date.now() + 120_000;
  while (Date.now() < deadline) {
    if (fs.existsSync(cleanupFile)) {
      return;
    }
    await new Promise((resolve) => setTimeout(resolve, 100));
  }
  throw new Error("browser workflow cleanup signal timed out");
}

function resetWorkflowTimeline() {
  workflowTimelineStartedAt = Date.now();
  workflowTimelineEvents = [];
}

function markWorkflowEvent(label, extra = {}) {
  if (!workflowTimelineStartedAt) {
    return;
  }
  workflowTimelineEvents.push({
    at_seconds: Number(((Date.now() - workflowTimelineStartedAt) / 1000).toFixed(3)),
    label,
    ...extra,
  });
}

function workflowElapsedSeconds() {
  if (!workflowTimelineStartedAt) {
    return 0;
  }
  return (Date.now() - workflowTimelineStartedAt) / 1000;
}

async function waitForWorkflowSecond(targetSeconds) {
  const remaining = targetSeconds - workflowElapsedSeconds();
  if (remaining > 0) {
    await pause(remaining);
  }
}

async function requireText(page, regex, label) {
  const text = await page.locator("body").innerText({ timeout: 10_000 });
  if (!regex.test(text)) {
    throw new Error(`${label} did not render expected text: ${regex}`);
  }
  return text;
}

async function loginClickHousePlay(page) {
  if (!page.url().includes("/play")) {
    await page.goto("http://localhost:8123/play", { waitUntil: "domcontentloaded" });
  }
  await page.waitForTimeout(400);
  await applyDarkTheme(page);
  await enhanceClickHousePlayTableBrowser(page);
  const inputs = page.locator("input");
  await inputs.nth(1).fill("carbon");
  await inputs.nth(2).fill("carbon");
}

async function enhanceClickHousePlayTableBrowser(page) {
  await page
    .addStyleTag({
      content: `
        :root {
          --table-size-bar-color: rgba(250, 255, 105, 0.34) !important;
        }
        .table {
          background-size: 100% 100% !important;
          border-radius: 2px !important;
        }
        .table button {
          background: transparent !important;
        }
        .table.current,
        .table:hover {
          filter: brightness(1.18) !important;
        }
      `,
    })
    .catch(() => {});
}

async function waitForPlayResult(page, expected, label) {
  const deadline = Date.now() + 15_000;
  let lastText = "";
  let matchedExpectedAt = 0;
  while (Date.now() < deadline) {
    lastText = await page.locator("body").innerText({ timeout: 1000 }).catch(() => "");
    const hasExpected = expected.test(lastText);
    const hasResultSummary = /(?:\d+(?:\.\d+)?\s*ms\.\s*Read|Read\s+\d+|\b\d+\s+rows?\s+in\s+(?:set|result)\b|\b\d+\s+rows?\b|Rows read)/i.test(lastText);
    const stillLoading = /loading|running query/i.test(lastText);
    if (hasExpected && !matchedExpectedAt) {
      matchedExpectedAt = Date.now();
    }
    if (hasExpected && hasResultSummary && !stillLoading) {
      return lastText;
    }
    if (matchedExpectedAt && Date.now() - matchedExpectedAt > 2500 && !stillLoading) {
      return lastText;
    }
    await page.waitForTimeout(350);
  }
  throw new Error(`${label} did not render completed result`);
}

function escapeRegExp(text) {
  return String(text).replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
}

function playTableNameRegex(tableName) {
  return new RegExp(`^${escapeRegExp(tableName)}\\b`);
}

async function playTableButton(page, tableName) {
  const button = page.getByRole("button", { name: playTableNameRegex(tableName) }).first();
  await button.scrollIntoViewIfNeeded({ timeout: 8000 });
  return button;
}

async function scrollClickHousePlayTableList(page, sceneId) {
  await page.waitForTimeout(350);
  const box = await page
    .evaluate(() => {
      const tables = document.querySelector(".tables");
      if (!tables) return null;
      let node = tables.parentElement;
      while (node && node !== document.body) {
        if (node.scrollHeight > node.clientHeight + 40) {
          const rect = node.getBoundingClientRect();
          return {
            x: rect.left,
            y: rect.top,
            width: rect.width,
            height: rect.height,
          };
        }
        node = node.parentElement;
      }
      const rect = tables.getBoundingClientRect();
      return {
        x: rect.left,
        y: rect.top,
        width: rect.width,
        height: rect.height,
      };
    })
    .catch(() => null);
  if (box) {
    const rng = seededRng(`${sceneId}:clickhouse-play-table-scroll`);
    const x = box.x + Math.min(box.width - 12, Math.max(18, box.width * 0.68));
    const y = box.y + Math.min(box.height - 18, Math.max(42, box.height * 0.42));
    await page.mouse.move(x, y, { steps: 5 }).catch(() => {});
    const deltas = [110, 150, 185, 135, 165, 120, 95];
    for (let index = 0; index < deltas.length; index += 1) {
      const delta = Math.round(deltas[index] * (0.88 + rng() * 0.28));
      await page.mouse.wheel(0, delta).catch(() => {});
      const basePause = index === 2 ? 620 : 260;
      await humanPause(page, basePause + rng() * 260);
    }
  }
  await page.waitForTimeout(950);
  setTransitionCurrent("ClickHouse Play", await shot(page, sceneId, "clickhouse-play-table-browser-scroll"));
}

async function openClickHousePlayTableBrowser(page, sceneId = null, options = {}) {
  await loginClickHousePlay(page);
  const visibleTable = page.getByRole("button", { name: /^(jupiter_swap_|token_program_)/i }).first();
  if (!(await visibleTable.isVisible({ timeout: 500 }).catch(() => false))) {
    await page.locator("#databases-toggle").click({ force: true }).catch(() => {});
    await page.waitForTimeout(options.revealMenu ? 1400 : 450);
    if (options.revealMenu && sceneId) {
      setTransitionCurrent("ClickHouse Play", await shot(page, sceneId, "clickhouse-play-menu-expanded"));
      markWorkflowEvent("clickhouse-play:database-menu-visible");
    }
    const defaultDatabase = page.getByRole("button", { name: /^default\b/i }).first();
    if (await defaultDatabase.isVisible({ timeout: 2500 }).catch(() => false)) {
      await defaultDatabase.click({ force: true }).catch(() => {});
    }
    if (options.revealMenu) {
      await page.waitForTimeout(1400);
    }
  }
  await page.waitForTimeout(1000);
  await requireText(page, /jupiter_swap_|token_program_/, "ClickHouse Play table browser");
  if (options.revealMenu && sceneId) {
    setTransitionCurrent("ClickHouse Play", await shot(page, sceneId, "clickhouse-play-table-browser-expanded"));
    markWorkflowEvent("clickhouse-play:table-browser-visible");
    await scrollClickHousePlayTableList(page, sceneId);
    markWorkflowEvent("clickhouse-play:table-list-scrolled");
  }
  hideMouse();
}

async function hoverPlayTable(page, sceneId, tableName, label) {
  const button = await playTableButton(page, tableName);
  await button.hover({ force: true });
  await page.waitForTimeout(650);
  await requireText(page, /MergeTree|rows|bytes|KiB|MiB|GiB/i, "ClickHouse Play table metadata");
  setTransitionCurrent("ClickHouse Play", await shot(page, sceneId, `clickhouse-play-${label}`));
  markWorkflowEvent(`clickhouse-play:${label}:visible`);
  await pause(0.8);
}

async function setPlayQuery(page, query) {
  await page.evaluate((value) => {
    const queryArea = document.querySelector("textarea");
    if (!queryArea) {
      throw new Error("ClickHouse Play query textarea not found");
    }
    const descriptor = Object.getOwnPropertyDescriptor(window.HTMLTextAreaElement.prototype, "value");
    descriptor?.set?.call(queryArea, value);
    queryArea.dispatchEvent(new Event("input", { bubbles: true }));
    document.getElementById("main")?.scrollTo(0, 0);
  }, query);
  await page.waitForFunction((value) => document.querySelector("textarea")?.value === value, query, { timeout: 5000 });
}

async function selectPlayTableDefaultQuery(page, tableName) {
  const button = await playTableButton(page, tableName);
  await button.evaluate((el) => {
    const table = el.closest(".table");
    if (!table) {
      throw new Error("ClickHouse Play table row not found");
    }
    table.dispatchEvent(new MouseEvent("click", { bubbles: true, cancelable: true, view: window }));
  });
  const expected = `SELECT * FROM "default"."${tableName}" LIMIT 100`;
  await page.waitForFunction((value) => document.querySelector("textarea")?.value === value, expected, { timeout: 5000 });
}

async function inspectPlayTable(page, sceneId, tableName, expected, label, query = null) {
  await openClickHousePlayTableBrowser(page);
  const button = await playTableButton(page, tableName);
  await button.hover({ force: true }).catch(() => {});
  if (query) {
    await setPlayQuery(page, query);
  } else {
    await selectPlayTableDefaultQuery(page, tableName);
  }
  await page.waitForTimeout(450);
  await page.locator("button").filter({ hasText: /^Run$/ }).first().click({ force: true }).catch(() => {});
  hideMouse();
  await page.waitForTimeout(1200);
  const text = await waitForPlayResult(page, expected, `ClickHouse Play table ${tableName}`);
  if (/Exception|DB::Exception|Code:\s*\d+/.test(text)) {
    throw new Error(`ClickHouse /play table query failed for ${tableName}`);
  }
  setTransitionCurrent("ClickHouse Play", await shot(page, sceneId, `clickhouse-play-${label}`));
  markWorkflowEvent(`clickhouse-play:${label}:visible`);
  await pause(0.8);
}

async function resultHorizontalScrollBox(page) {
  return page
    .evaluate(() => {
      const elements = [];
      const collect = (root) => {
        for (const element of Array.from(root.querySelectorAll("*"))) {
          elements.push(element);
          if (element.shadowRoot) {
            collect(element.shadowRoot);
          }
        }
      };
      collect(document);
      const candidates = [];
      for (const element of elements) {
        const rect = element.getBoundingClientRect();
        if (rect.width < 360 || rect.height < 120) continue;
        if (rect.top < 260 || rect.bottom > window.innerHeight + 80) continue;
        if (element.scrollWidth <= element.clientWidth + 80) continue;
        const text = element.innerText || element.textContent || "";
        const resultAncestor = element.closest?.("#query-result, query-result");
        const geometryLikelyResult =
          rect.left > 350 && rect.top > 260 && element.scrollWidth - element.clientWidth > 250;
        const resultLike =
          resultAncestor ||
          element.id === "query-result" ||
          element.tagName === "QUERY-RESULT" ||
          geometryLikelyResult ||
          /program_id|family_name|instruction_type|amount|decimals|signature|instr/i.test(text);
        if (!resultLike) continue;
        let score = element.scrollWidth - element.clientWidth;
        if (resultAncestor) score += 1000;
        if (element.id === "query-result" || element.tagName === "QUERY-RESULT") score += 600;
        if (/program_id|family_name|instruction_type|amount|decimals|signature/i.test(text)) score += 350;
        if (rect.top > 300 && rect.top < window.innerHeight - 220) score += 200;
        candidates.push({
          x: rect.left,
          y: rect.top,
          width: rect.width,
          height: rect.height,
          score,
        });
      }
      candidates.sort((a, b) => b.score - a.score);
      return candidates[0] || null;
    })
    .catch(() => null);
}

async function scrollResultHorizontallyBy(page, scrollDelta) {
  return page.evaluate((delta) => {
    const elements = [];
    const collect = (root) => {
      for (const element of Array.from(root.querySelectorAll("*"))) {
        elements.push(element);
        if (element.shadowRoot) {
          collect(element.shadowRoot);
        }
      }
    };
    collect(document);
    const candidates = [];
    for (const element of elements) {
      const rect = element.getBoundingClientRect();
      if (rect.width < 360 || rect.height < 120) continue;
      if (rect.top < 260 || rect.bottom > window.innerHeight + 80) continue;
      if (element.scrollWidth <= element.clientWidth + 80) continue;
      const text = element.innerText || element.textContent || "";
      const resultAncestor = element.closest?.("#query-result, query-result");
      const geometryLikelyResult =
        rect.left > 350 && rect.top > 260 && element.scrollWidth - element.clientWidth > 250;
      const resultLike =
        resultAncestor ||
        element.id === "query-result" ||
        element.tagName === "QUERY-RESULT" ||
        geometryLikelyResult ||
        /program_id|family_name|instruction_type|amount|decimals|signature|instr/i.test(text);
      if (!resultLike) continue;
      let score = element.scrollWidth - element.clientWidth;
      if (resultAncestor) score += 1000;
      if (element.id === "query-result" || element.tagName === "QUERY-RESULT") score += 600;
      if (/program_id|family_name|instruction_type|amount|decimals|signature/i.test(text)) score += 350;
      if (rect.top > 300 && rect.top < window.innerHeight - 220) score += 200;
      candidates.push({ element, score });
    }
    candidates.sort((a, b) => b.score - a.score);
    let changed = 0;
    let maxScrollLeft = 0;
    for (const { element } of candidates.slice(0, 8)) {
      const before = element.scrollLeft;
      const limit = Math.max(0, element.scrollWidth - element.clientWidth);
      const next = Math.min(limit, before + delta);
      element.scrollLeft = next;
      element.scrollTo?.({ left: next, behavior: "auto" });
      if (element.scrollLeft !== before) {
        changed += 1;
        maxScrollLeft = Math.max(maxScrollLeft, element.scrollLeft);
      }
    }
    return { changed, maxScrollLeft };
  }, scrollDelta);
}

function easeInOutCubic(t) {
  return t < 0.5
    ? 4 * t * t * t
    : 1 - Math.pow(-2 * t + 2, 3) / 2;
}

async function scrollResultHorizontallyToRatio(page, ratio, rng = () => 0.5) {
  const durationJitterMs = (rng() - 0.5) * 400;
  return page.evaluate(
    async ({ targetRatio, durationJitter }) => {
      const easeInOutCubicInPage = (t) =>
        t < 0.5
          ? 4 * t * t * t
          : 1 - Math.pow(-2 * t + 2, 3) / 2;
      const elements = [];
      const collect = (root) => {
        for (const element of Array.from(root.querySelectorAll("*"))) {
          elements.push(element);
          if (element.shadowRoot) {
            collect(element.shadowRoot);
          }
        }
      };
      collect(document);
      const candidates = [];
      const addCandidate = (element, score) => {
        if (!element || element.scrollWidth <= element.clientWidth + 80) return;
        candidates.push({ element, score });
      };
      addCandidate(document.scrollingElement || document.documentElement, 4500);
      for (const element of elements) {
        const rect = element.getBoundingClientRect();
        if (rect.width < 320 || rect.height < 40) continue;
        if (element.scrollWidth <= element.clientWidth + 80) continue;
        const text = element.innerText || element.textContent || "";
        const resultAncestor = element.closest?.("#query-result, query-result");
        const textLikelyResult = /program_id|family_name|instruction_type|amount|decimals|signature|instr/i.test(text);
        const geometryLikelyResult =
          rect.left > 300 && rect.top > 220 && element.scrollWidth - element.clientWidth > 250;
        const resultLike =
          resultAncestor ||
          element.id === "query-result" ||
          element.tagName === "QUERY-RESULT" ||
          geometryLikelyResult ||
          textLikelyResult;
        if (!resultLike) continue;
        let score = element.scrollWidth - element.clientWidth;
        if (resultAncestor) score += 1000;
        if (element.id === "query-result" || element.tagName === "QUERY-RESULT") score += 600;
        if (textLikelyResult) score += 350;
        if (rect.top > 300 && rect.top < window.innerHeight - 220) score += 200;
        addCandidate(element, score);
      }
      candidates.sort((a, b) => b.score - a.score);
      let selected = null;
      for (const candidate of candidates.slice(0, 8)) {
        const { element } = candidate;
        const maxScrollLeft = Math.max(0, element.scrollWidth - element.clientWidth);
        if (maxScrollLeft <= 0) continue;
        const original = element.scrollLeft;
        const probe = original < maxScrollLeft ? original + 1 : original - 1;
        element.scrollLeft = probe;
        const moved = element.scrollLeft !== original;
        element.scrollLeft = original;
        element.scrollTo?.({ left: original, behavior: "auto" });
        if (moved) {
          selected = candidate;
          break;
        }
      }
      if (!selected) {
        return { changed: 0, scrollLeft: 0, maxScrollLeft: 0 };
      }

      const { element } = selected;
      const maxScrollLeft = Math.max(0, element.scrollWidth - element.clientWidth);
      const startLeft = element.scrollLeft;
      const boundedRatio = Math.max(0, Math.min(1, Number(targetRatio) || 0));
      const targetLeft = Math.round(maxScrollLeft * boundedRatio);
      const distance = Math.abs(targetLeft - startLeft);
      if (maxScrollLeft <= 0 || distance < 1) {
        return { changed: 0, scrollLeft: element.scrollLeft, maxScrollLeft };
      }

      const durationMs = Math.max(750, Math.min(1000, 700 + Math.min(300, distance * 0.08) + durationJitter * 0.75));
      const frames = Math.max(12, Math.min(20, Math.round(durationMs / 58)));
      for (let frame = 1; frame <= frames; frame += 1) {
        const eased = easeInOutCubicInPage(frame / frames);
        const next = Math.round(startLeft + (targetLeft - startLeft) * eased);
        element.scrollLeft = next;
        element.scrollTo?.({ left: next, behavior: "auto" });
        await new Promise((resolve) => setTimeout(resolve, durationMs / frames));
      }
      element.scrollLeft = targetLeft;
      element.scrollTo?.({ left: targetLeft, behavior: "auto" });
      return {
        changed: element.scrollLeft !== startLeft ? 1 : 0,
        scrollLeft: element.scrollLeft,
        maxScrollLeft,
      };
    },
    { targetRatio: ratio, durationJitter: durationJitterMs },
  );
}

async function dragResultHorizontalScrollbar(page) {
  const resultBox = await page
    .locator("query-result, #query-result")
    .first()
    .boundingBox({ timeout: 1500 })
    .catch(() => null);
  const viewport = page.viewportSize() || { width: 1920, height: 1080 };
  if (!resultBox) {
    return false;
  }
  const y = Math.max(20, Math.min(viewport.height - 10, resultBox.y + resultBox.height - 10));
  const startX = Math.max(20, Math.min(viewport.width - 40, resultBox.x + resultBox.width * 0.23));
  const firstX = Math.max(20, Math.min(viewport.width - 40, resultBox.x + resultBox.width * 0.38));
  const midX = Math.max(20, Math.min(viewport.width - 40, resultBox.x + resultBox.width * 0.53));
  const lateX = Math.max(20, Math.min(viewport.width - 40, resultBox.x + resultBox.width * 0.70));
  const endX = Math.max(20, Math.min(viewport.width - 28, resultBox.x + resultBox.width - 72));
  await page.mouse.move(startX, y, { steps: 8 }).catch(() => {});
  await page.mouse.down().catch(() => {});
  await page.mouse.move(firstX, y, { steps: 18 }).catch(() => {});
  await humanPause(page, 1110);
  await page.mouse.move(midX, y, { steps: 20 }).catch(() => {});
  await humanPause(page, 1330);
  await page.mouse.move(lateX, y, { steps: 22 }).catch(() => {});
  await humanPause(page, 1290);
  await page.mouse.move(endX, y, { steps: 24 }).catch(() => {});
  await humanPause(page, 1130);
  await page.mouse.up().catch(() => {});
  await humanPause(page, 720);
  return true;
}

async function scrollPlayResultHorizontally(page, sceneId, label) {
  await page.waitForTimeout(700);
  let box = await resultHorizontalScrollBox(page);
  if (!box) {
    box = await page
      .locator("query-result, #query-result")
      .first()
      .boundingBox({ timeout: 1500 })
      .catch(() => null);
  }
  const rng = seededRng(`${sceneId}:clickhouse-play-horizontal-scroll:${label}`);
  markWorkflowEvent("clickhouse-play:horizontal-scroll-start");
  let changed = 0;
  if (box) {
    const x = box.x + Math.min(box.width - 24, Math.max(36, box.width * 0.76));
    const y = box.y + Math.min(box.height - 22, Math.max(42, box.height * 0.56));
    await page.mouse.move(x, y, { steps: 7 }).catch(() => {});
    await humanPause(page, 900 + rng() * 400);

    const targets = [
      { ratio: 0.18, hold: 1600 },
      { ratio: 0.36, hold: 1900 },
      { ratio: 0.56, hold: 2100 },
      { ratio: 0.76, hold: 1800 },
      { ratio: 0.90, hold: 2400 },
    ];

    for (const target of targets) {
      const result = await scrollResultHorizontallyToRatio(page, target.ratio, rng).catch(() => ({ changed: 0 }));
      changed += Number(result.changed || 0);
      await humanPause(page, target.hold + (rng() - 0.5) * 700);
    }
  }
  if (changed === 0) {
    const dragged = await dragResultHorizontalScrollbar(page);
    if (!dragged) {
      throw new Error("ClickHouse Play result horizontal scroller not found");
    }
  }
  setTransitionCurrent("ClickHouse Play", await shot(page, sceneId, `clickhouse-play-${label}-horizontal-scroll`));
  markWorkflowEvent("clickhouse-play:horizontal-scroll-complete");
  await pause(1.8);
}

async function runPlayQuery(page, sceneId, query, expected, label, holdSeconds = 0.8) {
  await openClickHousePlayTableBrowser(page);
  await setPlayQuery(page, query);
  await page.waitForTimeout(450);
  await page.locator("button").filter({ hasText: /^Run$/ }).first().click({ force: true }).catch(() => {});
  hideMouse();
  await page.waitForTimeout(1200);
  const text = await waitForPlayResult(page, expected, `ClickHouse Play query ${label}`);
  if (/Exception|DB::Exception|Code:\s*\d+/.test(text)) {
    throw new Error(`ClickHouse /play query failed for ${label}`);
  }
  setTransitionCurrent("ClickHouse Play", await shot(page, sceneId, `clickhouse-play-${label}`));
  markWorkflowEvent(`clickhouse-play:${label}:visible`);
  await pause(holdSeconds);
}

async function clickHousePlayLandingValidation(page, sceneId) {
  await openClickHousePlayTableBrowser(page, sceneId, { revealMenu: true });
  updateTransitionSlot("ClickHouse Play", await shot(page, sceneId, "clickhouse-play-table-browser"));
  await runPlayQuery(
    page,
    sceneId,
    `WITH
    ['Jupiter Swap', 'Token Program'] AS decoders,
    ['account rows', 'instruction rows', 'CPI event rows'] AS families
SELECT
    decoder,
    table_family,
    ifNull(tables, 0) AS tables,
    ifNull(rows, 0) AS rows,
    formatReadableSize(ifNull(bytes_raw, 0)) AS bytes
FROM (SELECT arrayJoin(decoders) AS decoder) AS d
CROSS JOIN (SELECT arrayJoin(families) AS table_family) AS f
LEFT JOIN
(
    SELECT
        decoder,
        table_family,
        count() AS tables,
        sum(total_rows) AS rows,
        sum(total_bytes) AS bytes_raw
    FROM
    (
        SELECT
            name,
            total_rows,
            total_bytes,
            multiIf(
                startsWith(name, 'jupiter_swap_'), 'Jupiter Swap',
                startsWith(name, 'token_program_'), 'Token Program',
                'other'
            ) AS decoder,
            multiIf(
                endsWith(name, '_account_landing'), 'account rows',
                endsWith(name, '_instruction_landing'), 'instruction rows',
                endsWith(name, '_event_landing'), 'CPI event rows',
                'other'
            ) AS table_family
        FROM system.tables
        WHERE database = 'default'
          AND endsWith(name, '_landing')
          AND (
              endsWith(name, '_account_landing')
              OR endsWith(name, '_instruction_landing')
              OR endsWith(name, '_event_landing')
          )
          AND (startsWith(name, 'jupiter_swap_') OR startsWith(name, 'token_program_'))
    )
    GROUP BY decoder, table_family
) AS actual USING (decoder, table_family)
ORDER BY decoder, indexOf(families, table_family)`,
    /Jupiter Swap|Token Program|account rows|instruction rows|CPI event rows/i,
    "landing-table-families-by-decoder",
    8.0,
  );
  await waitForWorkflowSecond(30.4);
  await runPlayQuery(
    page,
    sceneId,
    `SELECT
    multiIf(
        startsWith(name, 'jupiter_swap_'), 'Jupiter Swap',
        startsWith(name, 'token_program_'), 'Token Program',
        'other'
    ) AS decoder,
    name,
    total_rows AS rows,
    formatReadableSize(total_bytes) AS bytes
FROM system.tables
WHERE database = 'default'
  AND endsWith(name, '_landing')
  AND total_rows > 0
  AND (startsWith(name, 'jupiter_swap_') OR startsWith(name, 'token_program_'))
ORDER BY rows DESC, total_bytes DESC`,
    /token_program_transfer_checked_instruction_landing|rows|bytes/i,
    "largest-populated-landing-tables",
    4.8,
  );
  await hoverPlayTable(page, sceneId, "token_program_transfer_checked_instruction_landing", "largest-table-metadata");
  await inspectPlayTable(
    page,
    sceneId,
    "token_program_transfer_checked_instruction_landing",
    /program_id|family_name|instruction_type|amount|decimals|signature/i,
    "largest-transfer-checked-rows",
  );
  await waitForWorkflowSecond(51.0);
  await scrollPlayResultHorizontally(page, sceneId, "largest-transfer-checked-rows");
  await pause(0.8);
}

async function dismissGrafanaPasswordModal(page) {
  for (let attempt = 0; attempt < 4; attempt += 1) {
    const bodyText = await page.locator("body").innerText({ timeout: 800 }).catch(() => "");
    if (!/(change|update).*password/i.test(bodyText)) {
      return;
    }
    const buttons = [
      page.locator("button, a").filter({ hasText: /skip for now/i }).first(),
      page.locator("button, a").filter({ hasText: /^skip$/i }).first(),
      page.locator("button, a").filter({ hasText: /later/i }).first(),
      page.locator("button, a").filter({ hasText: /cancel/i }).first(),
    ];
    let clicked = false;
    for (const button of buttons) {
      if (await button.isVisible({ timeout: 300 }).catch(() => false)) {
        await button.click({ force: true }).catch(() => {});
        clicked = true;
        await page.waitForTimeout(900);
        break;
      }
    }
    if (!clicked) {
      await page.keyboard.press("Escape").catch(() => {});
      await page.waitForTimeout(400);
    }
  }
  await page.evaluate(() => {
    for (const dialog of Array.from(document.querySelectorAll('[role="dialog"], [aria-modal="true"]'))) {
      const text = dialog.textContent || "";
      if (/change.*password|skip/i.test(text)) {
        dialog.remove();
      }
    }
    for (const el of Array.from(document.querySelectorAll("div"))) {
      const style = getComputedStyle(el);
      const text = el.textContent || "";
      const zIndex = Number.parseInt(style.zIndex || "0", 10);
      if (/(change|update).*password/i.test(text) || (style.position === "fixed" && zIndex >= 1000 && style.backgroundColor.includes("rgba"))) {
        el.remove();
      }
    }
  }).catch(() => {});
  await page.waitForTimeout(500);
}

async function applyGrafanaZoom(page, windowId = null) {
  if (!isGrafanaUrl(page.url())) {
    return;
  }
  if (page.__demoGrafanaChromeZoomApplied) {
    return;
  }
  const targetWindowId = windowId || chromiumWindowForLabel("Grafana") || latestChromiumWindow();
  if (!targetWindowId) {
    return;
  }
  page.__demoGrafanaChromeZoomApplied = true;
  tryWindowCommand(["windowfocus", targetWindowId]);
  tryWindowCommand(["key", "ctrl+0"]);
  tryWindowCommand(["key", "ctrl+minus"]);
  tryWindowCommand(["key", "ctrl+minus"]);
  await page.waitForTimeout(800);
  markWorkflowEvent("grafana:zoom-80");
}

async function grafanaDashboard(page, sceneId, label, dashboardKey = "overview") {
  const dashboard = GRAFANA_DASHBOARDS[dashboardKey] || GRAFANA_DASHBOARDS.overview;
  await seedGrafanaSession(page.context());
  if (!page.url().includes(`/d/${dashboard.uid}/`)) {
    await page.goto(grafanaDashboardUrl(dashboardKey), { waitUntil: "domcontentloaded" });
    await page.waitForTimeout(2500);
  } else {
    await page.waitForTimeout(700);
  }
  await dismissGrafanaPasswordModal(page);
  await applyGrafanaZoom(page);
  await shot(page, sceneId, `grafana-${label}-raw`);
  const text = await requireText(page, dashboard.title, "Grafana dashboard");
  if (/invalid username|password|login failed|email or username/i.test(text)) {
    throw new Error("Grafana login failed");
  }
  if (/(change|update) your password/i.test(text)) {
    throw new Error("Grafana password modal remained visible");
  }
  const noDataPanels = (text.match(/No data/g) || []).length;
  if (noDataPanels >= 6) {
    throw new Error("Grafana dashboard did not render enough metric data");
  }
  setTransitionCurrent("Grafana", await shot(page, sceneId, `grafana-${label}`));
  markWorkflowEvent(`grafana:${dashboardKey}:visible`, { dashboard_label: label });
  if (dashboardKey === "jupiter") {
    await pause(2.2);
    await page.mouse.wheel(0, 760).catch(() => {});
    await page.waitForTimeout(1200);
    setTransitionCurrent("Grafana", await shot(page, sceneId, `grafana-${label}-lower-panels`));
    markWorkflowEvent(`grafana:${dashboardKey}:lower-panels-visible`, { dashboard_label: label });
    await pause(1.3);
  } else {
    await pause(0.15);
  }
}

async function seedClickStackConnection(page) {
  await page.evaluate(() => {
    sessionStorage.setItem(
      "connections",
      JSON.stringify([
        {
          id: "local",
          name: "Local ClickHouse",
          host: "http://localhost:8123",
          username: "carbon",
          password: "carbon",
          hyperdxSettingPrefix: null,
        },
      ]),
    );
  });
}

async function completeClickStackConnection(page) {
  const welcome = page.getByText(/Welcome to ClickStack/i).first();
  if (!(await welcome.isVisible({ timeout: 700 }).catch(() => false))) {
    return;
  }
  await page.locator('input[placeholder="My Clickhouse Server"]').fill("Local ClickHouse").catch(() => {});
  await page.locator('input[placeholder="http://localhost:8123"]').fill("http://localhost:8123").catch(() => {});
  await page.locator('input[placeholder="Username (default: default)"]').fill(CLICKHOUSE_HTTP_USER).catch(() => {});
  await page.locator('input[placeholder="Password (default: blank)"]').fill(CLICKHOUSE_HTTP_PASSWORD).catch(() => {});
  await page.getByRole("button", { name: /Test Connection/i }).click({ force: true, timeout: 1500 }).catch(() => {});
  await page.waitForTimeout(700);
  await page
    .addStyleTag({
      content: `
        .mantine-Modal-overlay,
        .mantine-Modal-inner {
          display: none !important;
          visibility: hidden !important;
          pointer-events: none !important;
        }
      `,
    })
    .catch(() => {});
}

async function focusClickStackInsertCharts(page) {
  await page
    .addStyleTag({
      content: `
        html, body {
          overflow: hidden !important;
        }
        [data-demo-hidden-clickstack-parts="true"] {
          display: none !important;
          visibility: hidden !important;
        }
        [data-demo-hidden-clickstack-promo="true"] {
          display: none !important;
          visibility: hidden !important;
        }
      `,
    })
    .catch(() => {});
  const hidePartsPanels = async () => {
    await page
      .evaluate(() => {
        const hideClickStackBanner = () => {
          const pattern = /not recommended for production use/i;
          const leaves = Array.from(document.querySelectorAll("body *")).filter((element) => {
            if (!(element instanceof HTMLElement)) return false;
            const text = element.textContent || "";
            return pattern.test(text) && !Array.from(element.children).some((child) => pattern.test(child.textContent || ""));
          });
          for (const leaf of leaves) {
            let node = leaf;
            for (let depth = 0; node instanceof HTMLElement && node.parentElement && depth < 10; depth += 1) {
              const box = node.getBoundingClientRect();
              if (box.top < 96 && box.width > window.innerWidth * 0.6 && box.height > 0 && box.height < 240) {
                node.setAttribute("data-demo-hidden-clickstack-parts", "true");
                node.style.setProperty("display", "none", "important");
                node.style.setProperty("visibility", "hidden", "important");
                break;
              }
              node = node.parentElement;
            }
          }
        };
        const hidePanelContaining = (pattern) => {
          const leaves = Array.from(document.querySelectorAll("body *")).filter((element) => {
            if (!(element instanceof HTMLElement)) return false;
            const text = element.textContent || "";
            return pattern.test(text) && !Array.from(element.children).some((child) => pattern.test(child.textContent || ""));
          });
          for (const leaf of leaves) {
            let node = leaf;
            for (let depth = 0; node instanceof HTMLElement && node.parentElement && depth < 14; depth += 1) {
              const box = node.getBoundingClientRect();
              const text = node.textContent || "";
              const isMainInsertChart = /Insert (Rows|Bytes) Per Table/i.test(text);
              const isCandidatePanel =
                box.top > 380 &&
                box.width > window.innerWidth * 0.30 &&
                box.height > 36 &&
                box.height < window.innerHeight * 0.78;
              if (isCandidatePanel && !isMainInsertChart) {
                node.setAttribute("data-demo-hidden-clickstack-parts", "true");
                node.style.setProperty("display", "none", "important");
                node.style.setProperty("visibility", "hidden", "important");
                break;
              }
              node = node.parentElement;
            }
          }
        };
        const hidePromoPanel = () => {
          const leaves = Array.from(document.querySelectorAll("body *")).filter((element) => {
            if (!(element instanceof HTMLElement)) return false;
            const text = element.textContent || "";
            return /Ready to deploy on ClickHouse Cloud|Get Started for Free/i.test(text);
          });
          for (const leaf of leaves) {
            let node = leaf;
            for (let depth = 0; node instanceof HTMLElement && node.parentElement && depth < 8; depth += 1) {
              const box = node.getBoundingClientRect();
              if (box.left < 360 && box.top > 260 && box.width > 120 && box.width < 360 && box.height > 40 && box.height < 220) {
                node.setAttribute("data-demo-hidden-clickstack-promo", "true");
                node.style.setProperty("display", "none", "important");
                node.style.setProperty("visibility", "hidden", "important");
                break;
              }
              node = node.parentElement;
            }
          }
        };
        const run = () => {
          window.scrollTo(0, 0);
          hideClickStackBanner();
          hidePromoPanel();
          hidePanelContaining(/Max Active Parts per Partition/i);
          hidePanelContaining(/Active Parts Per Partition/i);
          hidePanelContaining(/Recommended to stay under 300/i);
          hidePanelContaining(/^Part Count$/i);
          window.scrollTo(0, 0);
        };
        run();
        if (!window.__demoClickStackInsertFocusInstalled) {
          window.__demoClickStackInsertFocusInstalled = true;
          new MutationObserver(run).observe(document.documentElement, { childList: true, subtree: true });
          window.setInterval(run, 250);
        }
      })
      .catch(() => {});
  };
  await hidePartsPanels();
  await page.waitForTimeout(300);
  await hidePartsPanels();
}

async function clickStackInsertsDashboard(page, sceneId, label) {
  const variant = { key: "rows", title: /Insert Rows Per Table/i };
  await seedClickStackConnection(page);
  if (!page.url().includes("/clickstack/clickhouse") || !page.url().includes("tab=inserts")) {
    await page.goto(`http://localhost:8123/clickstack/clickhouse?tab=inserts&insertsBy=${variant.key}`, {
      waitUntil: "domcontentloaded",
    });
    await page.waitForTimeout(2000);
  } else {
    await page.waitForTimeout(800);
  }
  await completeClickStackConnection(page).catch(() => {});
  await dismissClickStackBanner(page);
  await requireText(page, variant.title, `ClickStack Inserts ${variant.key}`);
  setTransitionCurrent("ClickStack", await shot(page, sceneId, `clickstack-inserts-${variant.key}-${label}`));
  markWorkflowEvent("clickstack:inserts-visible", { dashboard_label: label });
  await pause(12.5);
}

async function runWorkflow(sceneId, workflow) {
  ensureGrafanaDemoPassword();
  reloadGrafanaDashboards();
  if (workflow === "live-observability") {
    const checks = [];
    const grafana = await openBrowser(sceneId, grafanaDashboardUrl("jupiter"), "Grafana");
    resetWorkflowTimeline();
    markWorkflowEvent("grafana:jupiter-window-visible");
    try {
      await grafanaDashboard(grafana.page, sceneId, "jupiter-live", "jupiter");
      checks.push("grafana-jupiter-live");
      await pause(24.4);
      await grafanaDashboard(grafana.page, sceneId, "token-live", "token");
      checks.push("grafana-token-live");
      await pause(0.05);
    } catch (err) {
      await closeBrowser(grafana.browser, grafana.child);
      throw err;
    }
    let clickstack;
    try {
      markWorkflowEvent("clickstack:prepare-offscreen-start");
      clickstack = await openBrowser(sceneId, "http://localhost:8123/clickstack/clickhouse?tab=inserts&insertsBy=rows", "ClickStack");
      markWorkflowEvent("clickstack:window-visible");
    } catch (err) {
      await closeBrowser(grafana.browser, grafana.child);
      throw err;
    }
    await closeBrowser(grafana.browser, grafana.child);
    try {
      await clickStackInsertsDashboard(clickstack.page, sceneId, "live-examples");
      checks.push("clickstack-inserts-live-examples");
      signalBrowserDone();
      await waitForCleanupSignal();
    } finally {
      await closeBrowser(clickstack.browser, clickstack.child);
    }
    fs.mkdirSync(REPORT_DIR, { recursive: true });
    fs.writeFileSync(
      path.join(REPORT_DIR, `${sceneId}-browser-workflow.json`),
      JSON.stringify({ scene: sceneId, workflow, checks, timeline: workflowTimelineEvents }, null, 2),
    );
    return;
  }

  if (workflow !== "table-inspection") {
    throw new Error(`unknown workflow: ${workflow}`);
  }
  const { browser, page, child } = await openBrowser(sceneId, "http://localhost:8123/play", "ClickHouse Play");
  resetWorkflowTimeline();
  markWorkflowEvent("clickhouse-play:window-visible");
  const checks = [];
  try {
    await clickHousePlayLandingValidation(page, sceneId);
    checks.push("clickhouse-play-largest-landing-table");
    signalBrowserDone();
    await waitForCleanupSignal();
  } finally {
    await closeBrowser(browser, child);
  }

  fs.mkdirSync(REPORT_DIR, { recursive: true });
  fs.writeFileSync(
    path.join(REPORT_DIR, `${sceneId}-browser-workflow.json`),
    JSON.stringify({ scene: sceneId, workflow, checks, timeline: workflowTimelineEvents }, null, 2),
  );
}

async function main() {
  const sceneId = process.argv[2];
  const workflow = process.argv[3];
  if (!sceneId || !workflow) {
    console.error("usage: browser_workflow.js <scene-id> <live-observability|table-inspection>");
    process.exit(2);
  }
  await runWorkflow(sceneId, workflow);
  if (process.env.DEMO_KEEP_BROWSER_OPEN !== "false") {
    process.exit(0);
  }
}

main().catch((err) => {
  console.error(err && err.stack ? err.stack : String(err));
  process.exit(1);
});
