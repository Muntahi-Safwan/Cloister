import '@fontsource/ibm-plex-sans/400.css';
import '@fontsource/ibm-plex-sans/500.css';
import '@fontsource/ibm-plex-sans/600.css';
import '@fontsource/ibm-plex-mono/500.css';
import './styles/tokens.css';
import './styles/themes.css';
import './styles/base.css';

import { mount } from 'svelte';

import App from './App.svelte';
import { applyStoredTheme } from './lib/theme';

// Apply the last theme before the first paint, so light themes do not flash dark.
applyStoredTheme();

const target = document.getElementById('app');
if (!target) throw new Error('Missing #app element');

export default mount(App, { target });
