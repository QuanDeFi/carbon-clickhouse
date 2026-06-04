#!/usr/bin/env node
const fs = require("fs");
const path = require("path");
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

function normalish(rng) {
  const a = Math.max(rng(), 1e-6);
  const b = Math.max(rng(), 1e-6);
  return Math.sqrt(-2 * Math.log(a)) * Math.cos(2 * Math.PI * b);
}

function clamp(value, min, max) {
  return Math.max(min, Math.min(max, value));
}

async function humanPause(page, ms) {
  await page.waitForTimeout(Math.max(0, Math.round(ms)));
}

async function typeHumanText(page, text, seedText, options = {}) {
  const rng = seededRng(seedText);
  const base = Number(options.baseDelayMs || process.env.DEMO_BROWSER_TYPE_BASE_DELAY_MS || process.env.DEMO_TYPE_BASE_DELAY_MS || "50");
  const minDelay = Number(process.env.DEMO_TYPE_MIN_DELAY_MS || "28");
  const maxDelay = Number(process.env.DEMO_TYPE_MAX_DELAY_MS || "145");
  const lengthFactor =
    text.length <= 30 ? 1 : Math.max(0.67, 1 - ((Math.min(text.length, 110) - 30) / 80) * 0.33);
  const minEffective = minDelay * lengthFactor;
  const maxEffective = maxDelay * Math.max(lengthFactor, 0.82);
  let burstRemaining = 8 + Math.floor(rng() * 11);
  let previous = "";
  for (let index = 0; index < text.length; index += 1) {
    const char = text[index];
    const next = index + 1 < text.length ? text[index + 1] : "";
    await page.keyboard.type(char, { delay: 0 });
    let delay = base * Math.exp(normalish(rng) * 0.34);
    if ("\"'`$(){}[]<>|&;:=+".includes(char)) {
      delay *= 1.45;
    } else if (/-_.\/\\/.test(char)) {
      delay *= 1.2;
    } else if (char.toUpperCase() === char && char.toLowerCase() !== char) {
      delay *= 1.12;
    }
    if (previous === char) {
      delay *= 1.18;
    }
    delay *= lengthFactor;
    await humanPause(page, clamp(delay, minEffective, maxEffective));
    if (char === " ") {
      await humanPause(page, (45 + rng() * 80) * lengthFactor);
    } else if ("|&;".includes(char) || next === "|" || next === "&" || next === ";") {
      await humanPause(page, (110 + rng() * 210) * lengthFactor);
    }
    burstRemaining -= 1;
    if (burstRemaining <= 0 && next && char !== " ") {
      await humanPause(page, (45 + rng() * 115) * lengthFactor);
      burstRemaining = 8 + Math.floor(rng() * 11);
    }
    previous = char;
  }
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
    const ids = sh("xdotool", ["search", "--onlyvisible", "--class", "chrom"])
      .split("\n")
      .map((line) => line.trim())
      .filter((line) => /^\d+$/.test(line));
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
  const context = await chromium.launchPersistentContext(profile, {
    headless: false,
    viewport: { width: frame.width, height: frame.height },
    screen: size(),
    colorScheme: "dark",
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
    await page.waitForTimeout(2800);
    await dismissClickStackBanner(page).catch(() => {});
  } else {
    await page.waitForTimeout(500);
  }
  const windowId = latestChromiumWindow();
  if (windowId) {
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

async function typeSql(page, query) {
  await typeHumanText(page, query, `sql:${query}`, {
    baseDelayMs: Number(process.env.DEMO_PLAY_QUERY_TYPE_DELAY_MS || process.env.DEMO_BROWSER_TYPE_BASE_DELAY_MS || "44"),
  });
}

async function typeUiText(page, locator, value) {
  await locator.fill("");
  await locator.click({ force: true }).catch(() => {});
  await typeHumanText(page, value, `ui:${value}`, {
    baseDelayMs: Number(process.env.DEMO_UI_TYPE_DELAY_MS || process.env.DEMO_BROWSER_TYPE_BASE_DELAY_MS || "50"),
  });
  await page.waitForTimeout(180);
}

async function runPlayQuery(page, query, expected) {
  await loginClickHousePlay(page);
  const editor = page.locator("textarea").first();
  await editor.click();
  await editor.press("Control+A");
  await editor.press("Backspace");
  await page.waitForTimeout(250);
  await typeSql(page, query);
  await page.waitForTimeout(500);
  await page.locator("button").filter({ hasText: /^Run$/ }).first().click();
  hideMouse();
  await page.waitForTimeout(1200);
  const text = await waitForPlayResult(page, expected, "ClickHouse /play query");
  if (/Exception|DB::Exception|Code:\s*\d+/.test(text)) {
    throw new Error("ClickHouse /play query failed");
  }
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
    await scrollClickHousePlayTableList(page, sceneId);
  }
  hideMouse();
}

async function hoverPlayTable(page, sceneId, tableName, label) {
  const button = await playTableButton(page, tableName);
  await button.hover({ force: true });
  await page.waitForTimeout(650);
  await requireText(page, /MergeTree|rows|bytes|KiB|MiB|GiB/i, "ClickHouse Play table metadata");
  setTransitionCurrent("ClickHouse Play", await shot(page, sceneId, `clickhouse-play-${label}`));
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
  await pause(0.8);
}

async function clickHousePlayJupiter(page, sceneId) {
  await openClickHousePlayTableBrowser(page, sceneId, { revealMenu: true });
  updateTransitionSlot("ClickHouse Play", await shot(page, sceneId, "clickhouse-play-table-browser"));
  await hoverPlayTable(page, sceneId, "jupiter_swap_swap_event_landing", "jupiter-event-metadata");
  await inspectPlayTable(
    page,
    sceneId,
    "jupiter_swap_swap_event_landing",
    /program_id|family_name|swap_event|event_type/i,
    "jupiter-event-rows",
  );
  await inspectPlayTable(
    page,
    sceneId,
    "jupiter_swap_route_instruction_landing",
    /program_id|family_name|instruction_type|quoted_out_amount|slippage_bps/i,
    "jupiter-instruction-rows",
    'SELECT program_id, family_name, instruction_type, slot, signature, in_amount, quoted_out_amount, slippage_bps, platform_fee_bps, source_name, mode FROM "default"."jupiter_swap_route_instruction_landing" LIMIT 100',
  );
}

async function clickHousePlayToken(page, sceneId) {
  await openClickHousePlayTableBrowser(page);
  await hoverPlayTable(page, sceneId, "token_program_transfer_checked_instruction_landing", "token-instruction-metadata");
  await inspectPlayTable(
    page,
    sceneId,
    "token_program_transfer_checked_instruction_landing",
    /program_id|family_name|instruction_type|amount|decimals/i,
    "token-instruction-rows",
  );
  await inspectPlayTable(
    page,
    sceneId,
    "token_program_multisig_account_landing",
    /program_id|family_name|account_type|pubkey|is_initialized|source_name|mode/i,
    "token-account-rows",
    'SELECT program_id, family_name, account_type, slot, pubkey, m, n, is_initialized, source_name, mode FROM "default"."token_program_multisig_account_landing" LIMIT 100',
  );
}

async function clickHousePlayAsyncLog(page, sceneId) {
  execFileSync("docker", ["exec", "clickhouse", "clickhouse-client", "-q", "SYSTEM FLUSH LOGS"], {
    cwd: ROOT,
    stdio: "ignore",
    env: process.env,
  });
  await loginClickHousePlay(page);
  await runPlayQuery(
    page,
    "SELECT event_time, table, rows, bytes, status, left(query_id,32) query_id, left(flush_query_id,12) flush_id\nFROM system.asynchronous_insert_log\nORDER BY event_time DESC\nLIMIT 20",
    /event_time|table|rows|bytes|status|query_id|flush_id/i,
  );
  setTransitionCurrent("ClickHouse Play", await shot(page, sceneId, "clickhouse-play-async-log"));
  await pause(1.5);
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
  if (dashboardKey === "jupiter") {
    await pause(2.2);
    await page.mouse.wheel(0, 760).catch(() => {});
    await page.waitForTimeout(1200);
    setTransitionCurrent("Grafana", await shot(page, sceneId, `grafana-${label}-lower-panels`));
    await pause(1.3);
  } else {
    await pause(1.6);
  }
}

async function clickHouseDashboards(page, sceneId, label) {
  await page.goto("http://carbon:carbon@localhost:8123/dashboards", { waitUntil: "domcontentloaded" });
  await page.waitForTimeout(800);
  const inputs = page.locator("input");
  if ((await inputs.count()) >= 3) {
    await inputs.nth(0).fill("http://localhost:8123");
    await inputs.nth(1).fill("carbon");
    await inputs.nth(2).fill("carbon");
    await page.getByRole("button", { name: /^ok$/i }).click().catch(() => {});
  }
  await page.getByRole("button", { name: /Reload/i }).click().catch(() => {});
  await page.waitForTimeout(3000);
  await shot(page, sceneId, `clickhouse-dashboard-${label}-raw`);
  await requireText(page, /Queries\/second|CPU Usage|Merges Running|Selected Bytes/i, "ClickHouse dashboard");
  await shot(page, sceneId, `clickhouse-dashboard-${label}`);
  await pause(4.0);
}

async function setHiddenInputValue(page, name, value) {
  await page.evaluate(
    ([fieldName, expected]) => {
      const input = document.querySelector(`input[name="${fieldName}"]`);
      if (!input) {
        return false;
      }
      const descriptor = Object.getOwnPropertyDescriptor(window.HTMLInputElement.prototype, "value");
      descriptor?.set?.call(input, expected);
      input.dispatchEvent(new Event("input", { bubbles: true }));
      input.dispatchEvent(new Event("change", { bubbles: true }));
      return true;
    },
    [name, value],
  );
}

async function setVisibleInputValue(locator, value) {
  await locator.evaluate((input, expected) => {
    const descriptor = Object.getOwnPropertyDescriptor(window.HTMLInputElement.prototype, "value");
    descriptor?.set?.call(input, expected);
    input.setAttribute("value", expected);
    input.dispatchEvent(new Event("input", { bubbles: true }));
    input.dispatchEvent(new Event("change", { bubbles: true }));
  }, value);
}

async function choose(page, input, value, hiddenName = null) {
  await typeUiText(page, input, value);
  const escaped = value.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
  const option = page.getByText(new RegExp(`^${escaped}$`)).last();
  if (await option.isVisible({ timeout: 700 }).catch(() => false)) {
    await option.click({ force: true }).catch(() => {});
  } else {
    await page.keyboard.press("Enter").catch(() => {});
  }
  if (hiddenName) {
    await setHiddenInputValue(page, hiddenName, value);
  }
  await setVisibleInputValue(input, value);
  await page.keyboard.press("Escape").catch(() => {});
  await page.waitForTimeout(500);
}

async function expectHiddenValue(page, name, value) {
  return page.waitForFunction(
    ([fieldName, expected]) => document.querySelector(`input[name="${fieldName}"]`)?.value === expected,
    [name, value],
    { timeout: 5000 },
  ).then(() => true).catch(() => false);
}

async function setCodeMirror(page, index, text) {
  const editor = page.locator('.cm-content[role="textbox"]').nth(index);
  await editor.fill("");
  await editor.click({ force: true }).catch(() => {});
  await typeHumanText(page, text, `codemirror:${index}:${text}`, {
    baseDelayMs: Number(process.env.DEMO_UI_TYPE_DELAY_MS || process.env.DEMO_BROWSER_TYPE_BASE_DELAY_MS || "50"),
  });
  await page.waitForTimeout(300);
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

async function seedClickStackQueryLogSource(page) {
  await seedClickStackConnection(page);
  await page.evaluate(() => {
    localStorage.setItem(
      "hdx-local-source",
      JSON.stringify([
        {
          id: "local-query-log",
          name: "ClickHouse Query Log",
          kind: "log",
          connection: "local",
          from: { databaseName: "system", tableName: "query_log" },
          timestampValueExpression: "event_time",
          defaultTableSelectExpression:
            "event_time, query_kind, query, initial_user, read_rows, written_rows, query_duration_ms",
          implicitColumnExpression: "query",
          querySettings: [],
        },
      ]),
    );
    localStorage.setItem("hdx-last-selected-source-id", "local-query-log");
  });
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
  }
  await page.waitForTimeout(2600);
  await dismissClickStackBanner(page);
  await requireText(page, variant.title, `ClickStack Inserts ${variant.key}`);
  setTransitionCurrent("ClickStack", await shot(page, sceneId, `clickstack-inserts-${variant.key}-${label}`));
  await pause(7.0);
}

async function clickStackQueryLog(page, sceneId, label) {
  if (!page.url().includes("/clickstack")) {
    await page.goto("http://localhost:8123/clickstack", { waitUntil: "domcontentloaded" });
  }
  await page.waitForTimeout(1200);
  await applyDarkTheme(page);
  await dismissClickStackBanner(page);
  const skipSourceSetup =
    label === "after-async" || (process.env.DEMO_CLICKSTACK_SKIP_SOURCE_SETUP || "false").toLowerCase() === "true";
  if (skipSourceSetup) {
    await seedClickStackQueryLogSource(page);
    await page.goto("http://localhost:8123/clickstack/search?source=local-query-log", { waitUntil: "domcontentloaded" });
    await page.waitForTimeout(1600);
    await applyDarkTheme(page);
    await dismissClickStackBanner(page);
    await page.getByRole("button", { name: /^Run$/ }).click({ force: true }).catch(() => {});
    await page.waitForTimeout(1800);
    await applyDarkTheme(page);
    await dismissClickStackBanner(page);
    await shot(page, sceneId, `clickstack-query-log-${label}-raw`);
    await requireText(page, /Results Table/i, "ClickStack query log");
    setTransitionCurrent("ClickStack", await shot(page, sceneId, `clickstack-query-log-${label}`));
    await pause(1.0);
    return;
  }
  if (await page.getByTestId("connection-username-input").isVisible().catch(() => false)) {
    await typeUiText(page, page.getByTestId("connection-name-input"), "Local ClickHouse");
    await typeUiText(page, page.getByTestId("connection-host-input"), "http://localhost:8123");
    await typeUiText(page, page.getByTestId("connection-username-input"), "carbon");
    await typeUiText(page, page.getByTestId("connection-password-input"), "carbon");
    await page.getByRole("button", { name: /test connection/i }).click({ force: true }).catch(() => {});
    await page.waitForTimeout(900);
    await page
      .locator("button")
      .filter({ hasText: "Create Connection" })
      .evaluate((button) => button.click());
    await page.waitForTimeout(700);
  }
  await page
    .locator('input[placeholder="Database"], input[name="name"]')
    .last()
    .waitFor({ state: "visible", timeout: 8000 })
    .catch(() => {});
  const sourceName = page.locator('input[name="name"], input[placeholder="Name"]').first();
  const databaseInput = page.locator('input[placeholder="Database"]').last();
  const tableInput = page.locator('input[placeholder="Table"]').last();
  if (
    (await sourceName.isVisible().catch(() => false)) &&
    (await databaseInput.isVisible().catch(() => false)) &&
    (await tableInput.isVisible().catch(() => false))
  ) {
    await typeUiText(page, sourceName, "ClickHouse Query Log");
    await choose(page, databaseInput, "system", "from.databaseName");
    if (!(await expectHiddenValue(page, "from.databaseName", "system"))) {
      console.warn("ClickStack database hidden field did not update; falling back to seeded source state");
    }
    await choose(page, tableInput, "query_log", "from.tableName");
    if (!(await expectHiddenValue(page, "from.tableName", "query_log"))) {
      console.warn("ClickStack table hidden field did not update; falling back to seeded source state");
    }
    await setCodeMirror(page, 2, "event_time");
    await setCodeMirror(
      page,
      3,
      "event_time, query_kind, query, initial_user, read_rows, written_rows, query_duration_ms",
    );
    await setVisibleInputValue(databaseInput, "system");
    await setVisibleInputValue(tableInput, "query_log");
    updateTransitionSlot("ClickStack", await shot(page, sceneId, `clickstack-source-${label}`));
    await seedClickStackQueryLogSource(page);
    await page.locator("button").filter({ hasText: "Save New Source" }).first().click({ force: true }).catch(() => {});
    await page.waitForTimeout(500);
  }
  await seedClickStackQueryLogSource(page);
  await page.goto("http://localhost:8123/clickstack/search?source=local-query-log", { waitUntil: "domcontentloaded" });
  await page.waitForTimeout(1600);
  await applyDarkTheme(page);
  await dismissClickStackBanner(page);
  await page.getByRole("button", { name: /^Run$/ }).click({ force: true }).catch(() => {});
  await page.waitForTimeout(3000);
  await applyDarkTheme(page);
  await dismissClickStackBanner(page);
  await shot(page, sceneId, `clickstack-query-log-${label}-raw`);
  await requireText(page, /Results Table/i, "ClickStack query log");
  setTransitionCurrent("ClickStack", await shot(page, sceneId, `clickstack-query-log-${label}`));
  await pause(5.0);
}

async function runWorkflow(sceneId, workflow) {
  ensureGrafanaDemoPassword();
  reloadGrafanaDashboards();
  if (workflow === "live-observability") {
    const checks = [];
    const grafana = await openBrowser(sceneId, grafanaDashboardUrl("jupiter"), "Grafana");
    try {
      await grafanaDashboard(grafana.page, sceneId, "jupiter-live", "jupiter");
      checks.push("grafana-jupiter-live");
      await grafanaDashboard(grafana.page, sceneId, "token-live", "token");
      checks.push("grafana-token-live");
    } catch (err) {
      await closeBrowser(grafana.browser, grafana.child);
      throw err;
    }
    let clickstack;
    try {
      clickstack = await openBrowser(sceneId, "http://localhost:8123/clickstack/clickhouse?tab=inserts&insertsBy=rows", "ClickStack");
    } catch (err) {
      await closeBrowser(grafana.browser, grafana.child);
      throw err;
    }
    await closeBrowser(grafana.browser, grafana.child);
    try {
      await clickStackInsertsDashboard(clickstack.page, sceneId, "live-examples");
      checks.push("clickstack-inserts-live-examples");
    } finally {
      await closeBrowser(clickstack.browser, clickstack.child);
    }
    fs.mkdirSync(REPORT_DIR, { recursive: true });
    fs.writeFileSync(
      path.join(REPORT_DIR, `${sceneId}-browser-workflow.json`),
      JSON.stringify({ scene: sceneId, workflow, checks }, null, 2),
    );
    return;
  }

  if (workflow === "async-log") {
    const checks = [];
    const play = await openBrowser(sceneId, "http://localhost:8123/play", "ClickHouse Play");
    try {
      await clickHousePlayAsyncLog(play.page, sceneId);
      checks.push("clickhouse-play-async-log");
    } catch (err) {
      await closeBrowser(play.browser, play.child);
      throw err;
    }
    let clickstack;
    try {
      clickstack = await openBrowser(sceneId, "http://localhost:8123/clickstack", "ClickStack");
    } catch (err) {
      await closeBrowser(play.browser, play.child);
      throw err;
    }
    await closeBrowser(play.browser, play.child);
    try {
      await clickStackQueryLog(clickstack.page, sceneId, "after-async");
      checks.push("clickstack-query-log-after-async");
    } finally {
      await closeBrowser(clickstack.browser, clickstack.child);
    }
    fs.mkdirSync(REPORT_DIR, { recursive: true });
    fs.writeFileSync(
      path.join(REPORT_DIR, `${sceneId}-browser-workflow.json`),
      JSON.stringify({ scene: sceneId, workflow, checks }, null, 2),
    );
    return;
  }

  const { browser, page, child } = await openBrowser(sceneId, "http://localhost:8123/play", "ClickHouse Play");
  const checks = [];
  try {
    if (workflow === "post-jupiter") {
      await clickHousePlayJupiter(page, sceneId);
      checks.push("clickhouse-play-jupiter");
      await grafanaDashboard(page, sceneId, "after-jupiter");
      checks.push("grafana-after-jupiter");
    } else if (workflow === "post-token") {
      await clickHousePlayToken(page, sceneId);
      checks.push("clickhouse-play-token");
      await grafanaDashboard(page, sceneId, "after-token");
      checks.push("grafana-after-token");
      await clickStackQueryLog(page, sceneId, "after-token");
      checks.push("clickstack-query-log-after-token");
    } else if (workflow === "table-inspection") {
      await clickHousePlayJupiter(page, sceneId);
      checks.push("clickhouse-play-jupiter");
      await clickHousePlayToken(page, sceneId);
      checks.push("clickhouse-play-token");
    } else {
      throw new Error(`unknown workflow: ${workflow}`);
    }
  } finally {
    await closeBrowser(browser, child);
  }

  fs.mkdirSync(REPORT_DIR, { recursive: true });
  fs.writeFileSync(
    path.join(REPORT_DIR, `${sceneId}-browser-workflow.json`),
    JSON.stringify({ scene: sceneId, workflow, checks }, null, 2),
  );
}

async function main() {
  const sceneId = process.argv[2];
  const workflow = process.argv[3];
  if (!sceneId || !workflow) {
    console.error("usage: browser_workflow.js <scene-id> <post-jupiter|post-token|live-observability|table-inspection|async-log>");
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
