# Writing a service recipe

A recipe tells Cloister how to treat a well-known website. Recipes live in
[`src-tauri/src/services/catalog.rs`](../src-tauri/src/services/catalog.rs) as entries in the `RECIPES` list.

```rust
Recipe {
    id: "whatsapp",                      // stable, lowercase, unique
    name: "WhatsApp",                    // shown in the catalog and sidebar
    category: "Chat",                    // Chat, Mail, Community, Work chat, Social…
    url: "https://web.whatsapp.com/",    // start page, must be https
    hosts: &["whatsapp.com", "whatsapp.net"],
    unread: UnreadRule::TitleParentheses,
    veil_selectors: &["#pane-side", "#main"],
    user_agent: UserAgentPreset::Default,
},
```

## Fields

**`hosts`**: domains that belong to the service. Subdomains match automatically. Popups to these domains open
inside Cloister (for example a sign-in window); links anywhere else open in the default browser. Include sign-in
domains the service uses, such as `accounts.google.com` for Gmail.

**`unread`**: how the unread count is read.

- `UnreadRule::TitleParentheses` reads the first number in parentheses in the page title, such as `(3) WhatsApp`
  or `Inbox (12) - Gmail`.
- `UnreadRule::None` when the site does not expose a count in its title.

**`veil_selectors`**: CSS selectors for the regions Veil blurs: chat lists, message panes, previews, avatars. Keep
them broad and stable (prefer roles, ids and `aria-*` attributes over generated class names). An empty list
blurs the whole page, which is always safe.

**`user_agent`**: leave as `Default` unless the site refuses to load in WebView2.

## Checklist

1. Add the recipe to `RECIPES`.
2. Run `cargo test` in `src-tauri/`. The tests check that ids are unique, URLs are `https` and the start URL's
   host is covered by `hosts`.
3. Run the app, add the service from the catalog and check sign-in, notifications, the unread badge and Veil.
4. Open a pull request with a short note on what you tested.

Recipes must never read message content, automate the site or send anything on the user's behalf.
