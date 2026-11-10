/** URL helpers for the custom service form. Mirrors `validate_url` in Rust. */

export type UrlCheck = { ok: true; url: URL } | { ok: false; reason: string };

/** Validates a user-typed service address. Only `https` URLs with a domain are accepted. */
export function checkServiceUrl(raw: string): UrlCheck {
  const value = raw.trim();
  if (value === '') return { ok: false, reason: 'Paste the address you use in your browser.' };

  let url: URL;
  try {
    url = new URL(value);
  } catch {
    return { ok: false, reason: 'Use a full address, like https://chat.example.com' };
  }
  if (url.protocol !== 'https:')
    return { ok: false, reason: 'Only secure https:// addresses are supported.' };
  if (!url.hostname.includes('.'))
    return { ok: false, reason: 'The address needs a domain, like example.com.' };
  return { ok: true, url };
}

/** Lets people type `chat.example.com` and get `https://chat.example.com`. */
export function withHttps(raw: string): string {
  const value = raw.trim();
  if (value === '' || /^[a-z][a-z0-9+.-]*:/i.test(value)) return value;
  return `https://${value}`;
}

/**
 * Icon for a catalog entry's main domain (`whatsapp.com`, not `web.whatsapp.com`),
 * shown before the service has ever been opened.
 *
 * Uses DuckDuckGo's favicon service because several sites do not serve
 * `/favicon.ico`. Only catalog entries use it, and the catalog asks for every
 * entry at once, so it reveals nothing about which services you use. Custom
 * websites never go through it.
 */
export function catalogIconUrl(domain: string): string {
  return `https://icons.duckduckgo.com/ip3/${domain}.ico`;
}

/** URL of the site's favicon, used when a tile shows the site icon. */
export function faviconUrl(serviceUrl: string): string | null {
  try {
    return new URL('/favicon.ico', serviceUrl).toString();
  } catch {
    return null;
  }
}
