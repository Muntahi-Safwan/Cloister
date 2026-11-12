// Cloister landing page: points download buttons at the latest GitHub release,
// runs the small demos (Veil, themes, memory) and reveals sections as they
// scroll in. Works without JavaScript too: every button already links to the
// releases page and every section is visible.

const REPO = 'Muntahi-Safwan/Cloister';
const RELEASES = `https://github.com/${REPO}/releases/latest`;

/** File name patterns for each download, as produced by the release workflow. */
const ASSETS = {
  windows: /_x64-setup\.exe$/,
  'windows-msi': /_x64_en-US\.msi$/,
  mac: /_universal\.dmg$/,
  linux: /_amd64\.AppImage$/,
  'linux-deb': /_amd64\.deb$/,
  'linux-rpm': /\.x86_64\.rpm$/,
};

const PLATFORM_LABEL = { windows: 'Windows', mac: 'macOS', linux: 'Linux' };

/** Best guess at the visitor's system: 'windows', 'mac', 'linux' or null (phones, unknown). */
function detectOs() {
  const ua = navigator.userAgent;
  if (/Android|iPhone|iPad|iPod/i.test(ua)) return null;
  const platform = (navigator.userAgentData && navigator.userAgentData.platform) || navigator.platform || ua;
  if (/win/i.test(platform)) return 'windows';
  if (/mac/i.test(platform)) return 'mac';
  if (/linux|x11/i.test(platform)) return 'linux';
  return null;
}

function formatSize(bytes) {
  return `${Math.round(bytes / (1024 * 1024))} MB`;
}

async function latestRelease() {
  const response = await fetch(`https://api.github.com/repos/${REPO}/releases/latest`, {
    headers: { Accept: 'application/vnd.github+json' },
  });
  if (!response.ok) throw new Error(`GitHub answered ${response.status}`);
  return response.json();
}

function setUpDownloads() {
  const os = detectOs();
  const mainButtons = document.querySelectorAll('[data-download]');
  const heroMeta = document.querySelector('[data-download-meta]');
  const status = document.querySelector('[data-release-status]');

  if (os) {
    document
      .querySelectorAll('[data-download-label]')
      .forEach((label) => (label.textContent = `Download for ${PLATFORM_LABEL[os]}`));
    document.querySelector(`.platform[data-os="${os}"]`)?.classList.add('is-yours');
    document.body.classList.add('knows-os');
  }

  latestRelease()
    .then((release) => {
      const version = release.tag_name;
      document.querySelectorAll('[data-version]').forEach((el) => (el.textContent = version));
      status.textContent = `${version}, released ${new Date(release.published_at).toLocaleDateString(
        undefined,
        {
          day: 'numeric',
          month: 'short',
          year: 'numeric',
        },
      )}.`;

      const found = {};
      for (const [key, pattern] of Object.entries(ASSETS)) {
        const asset = release.assets.find((a) => pattern.test(a.name));
        if (!asset) continue;
        found[key] = asset;
        document.querySelectorAll(`[data-asset="${key}"]`).forEach((link) => {
          link.href = asset.browser_download_url;
          link.title = `${asset.name} · ${formatSize(asset.size)}`;
        });
      }

      if (os && found[os]) {
        mainButtons.forEach((button) => (button.href = found[os].browser_download_url));
        heroMeta.textContent = `${version} · ${formatSize(found[os].size)} · ${
          os === 'mac'
            ? 'Apple Silicon and Intel'
            : os === 'windows'
              ? 'Windows 10 and 11'
              : 'AppImage, also .deb and .rpm'
        }`;
      }
    })
    .catch(() => {
      // No release yet, offline or rate limited: the links still go to the releases page.
      if (!os) mainButtons.forEach((button) => (button.href = RELEASES));
    });
}

function setUpVeilDemo() {
  const toggle = document.querySelector('[data-veil-toggle]');
  const chats = document.querySelector('[data-chats]');
  const label = document.querySelector('[data-veil-label]');
  if (!toggle || !chats) return;
  toggle.addEventListener('click', () => {
    const on = chats.classList.toggle('is-veiled');
    toggle.setAttribute('aria-pressed', String(on));
    label.textContent = on ? 'Veil on' : 'Veil off';
  });
}

/**
 * Service tiles show each site's own icon. If an icon cannot load (offline,
 * blocked, or the site changed it), the tile falls back to the site's initial.
 */
function setUpServiceIcons() {
  document.querySelectorAll('.svc-icon').forEach((img) => {
    const fallBack = () => img.parentElement.classList.add('no-icon');
    if (img.complete && img.naturalWidth === 0) fallBack();
    else img.addEventListener('error', fallBack, { once: true });
  });
}

/**
 * Memory estimate. Each app costs its published idle figure whether it runs on
 * its own or awake in Cloister. A paused app is frozen and Windows may reuse its
 * memory; an asleep app is closed. Cloister itself adds a small fixed overhead.
 */
const CLOISTER_OVERHEAD_MB = 40;

function formatMb(mb) {
  return mb >= 1000 ? `${(mb / 1000).toFixed(1)} GB` : `${Math.round(mb)} MB`;
}

function setUpMemoryEstimate() {
  const rows = [...document.querySelectorAll('.mem-row')];
  if (!rows.length) return;
  const out = {
    saved: document.querySelector('[data-saved]'),
    oldTotal: document.querySelector('[data-total-old]'),
    newTotal: document.querySelector('[data-total-new]'),
    oldBar: document.querySelector('[data-bar-old]'),
    newBar: document.querySelector('[data-bar-new]'),
  };

  const update = () => {
    let separate = 0;
    let cloister = CLOISTER_OVERHEAD_MB;
    for (const row of rows) {
      const mb = Number(row.dataset.mb);
      separate += mb;
      if (row.querySelector('.mem-seg').dataset.state === 'awake') cloister += mb;
    }
    const saved = Math.max(0, separate - cloister);
    out.saved.textContent = saved > 0 ? formatMb(saved) : '0 MB';
    out.oldTotal.textContent = formatMb(separate);
    out.newTotal.textContent = formatMb(cloister);
    const max = Math.max(separate, cloister);
    out.oldBar.style.width = `${(separate / max) * 100}%`;
    out.newBar.style.width = `${(cloister / max) * 100}%`;
  };

  rows.forEach((row) => {
    const seg = row.querySelector('.mem-seg');
    const sync = () =>
      seg.querySelectorAll('[data-set]').forEach((button) => {
        button.setAttribute('aria-checked', String(button.dataset.set === seg.dataset.state));
      });
    seg.querySelectorAll('[data-set]').forEach((button) =>
      button.addEventListener('click', () => {
        seg.dataset.state = button.dataset.set;
        sync();
        update();
      }),
    );
    sync();
  });
  update();
}

/** Sections fade in once as they scroll into view. */
function setUpReveals() {
  const items = document.querySelectorAll('[data-reveal]');
  if (!('IntersectionObserver' in window)) {
    items.forEach((el) => el.classList.add('is-in'));
    return;
  }
  const observer = new IntersectionObserver(
    (entries) => {
      for (const entry of entries) {
        if (!entry.isIntersecting) continue;
        entry.target.classList.add('is-in');
        observer.unobserve(entry.target);
      }
    },
    { rootMargin: '0px 0px -8% 0px', threshold: 0.12 },
  );
  items.forEach((el) => observer.observe(el));
}

/** A hairline under the navigation once the page has scrolled. */
function setUpNav() {
  const nav = document.querySelector('[data-nav]');
  if (!nav) return;
  const sync = () => nav.classList.toggle('is-scrolled', window.scrollY > 8);
  window.addEventListener('scroll', sync, { passive: true });
  sync();
}

/** Show macOS key names to Mac visitors. */
function setUpKeys() {
  if (detectOs() !== 'mac') return;
  document.querySelectorAll('kbd').forEach((key) => {
    key.textContent = key.textContent.replace(/\bCtrl\b/g, '⌘').replace(/\bShift\b/g, '⇧');
  });
}

/** Theme chips recolour the preview window with the app's own palettes. */
function setUpThemePreview() {
  const preview = document.querySelector('[data-theme-preview]');
  const chips = [...document.querySelectorAll('[data-theme-pick]')];
  chips.forEach((chip) =>
    chip.addEventListener('click', () => {
      preview.dataset.theme = chip.dataset.themePick;
      chips.forEach((other) => {
        const on = other === chip;
        other.classList.toggle('is-on', on);
        other.setAttribute('aria-pressed', String(on));
      });
    }),
  );
}

setUpReveals();
setUpNav();
setUpKeys();
setUpServiceIcons();
setUpThemePreview();
setUpMemoryEstimate();
setUpDownloads();
setUpVeilDemo();
