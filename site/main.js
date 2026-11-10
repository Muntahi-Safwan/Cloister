// Cloister landing page: points download buttons at the latest GitHub release
// and runs the small Veil demo. Works without JavaScript too: every button
// already links to the releases page.

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
  const hero = document.querySelector('[data-download]');
  const heroLabel = document.querySelector('[data-download-label]');
  const heroMeta = document.querySelector('[data-download-meta]');
  const status = document.querySelector('[data-release-status]');

  if (os) {
    heroLabel.textContent = `Download for ${PLATFORM_LABEL[os]}`;
    document.querySelector(`.platform[data-os="${os}"]`)?.classList.add('is-yours');
  }

  latestRelease()
    .then((release) => {
      const version = release.tag_name;
      document.querySelectorAll('[data-version]').forEach((el) => (el.textContent = version));
      status.textContent = `Latest: ${version}, released ${new Date(release.published_at).toLocaleDateString()}.`;

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
        hero.href = found[os].browser_download_url;
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
      hero.href = os ? '#install' : RELEASES;
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

setUpDownloads();
setUpVeilDemo();
