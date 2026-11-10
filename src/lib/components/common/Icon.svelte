<!--
  @component
  Stroke icon set. All icons share one 24×24 grid and inherit `currentColor`.
  Add an icon by appending its path data to `PATHS`.
-->
<script lang="ts" module>
  const PATHS = {
    search: 'M11 4a7 7 0 1 0 0 14 7 7 0 0 0 0-14zM20 20l-3.5-3.5',
    plus: 'M12 5v14M5 12h14',
    close: 'M6 6l12 12M18 6L6 18',
    minimize: 'M5 12h14',
    maximize: 'M6 6h12v12H6z',
    eye: 'M2 12c1-2.5 5-7 10-7s9 4.5 10 7c-1 2.5-5 7-10 7S3 14.5 2 12zM12 9a3 3 0 1 0 0 6 3 3 0 0 0 0-6z',
    'eye-off':
      'M3 3l18 18M10.6 5.1A9.9 9.9 0 0 1 12 5c5 0 9 4.5 10 7a13 13 0 0 1-3 4.2M6.6 6.6C4.2 8.1 2.6 10.3 2 12c1 2.5 5 7 10 7a9.6 9.6 0 0 0 5.4-1.6',
    focus: 'M12 3.5a8.5 8.5 0 1 0 0 17 8.5 8.5 0 0 0 0-17zM12 8.8a3.2 3.2 0 1 0 0 6.4 3.2 3.2 0 0 0 0-6.4z',
    sliders: 'M4 7h10M18 7h2M4 17h4M12 17h8M16 5v4M10 15v4',
    moon: 'M20 14.5A8 8 0 0 1 9.5 4a8 8 0 1 0 10.5 10.5z',
    reload: 'M20 12a8 8 0 1 1-2.3-5.7M20 4v5h-5',
    trash: 'M4 7h16M10 11v6M14 11v6M6 7l1 12a2 2 0 0 0 2 2h6a2 2 0 0 0 2-2l1-12M9 7V4h6v3',
    check: 'M5 12l5 5 9-10',
    eraser: 'M16 3.5l4.5 4.5-10 10H6l-2.5-2.5a1.4 1.4 0 0 1 0-2L16 3.5zM9.5 9.5l5 5M10.5 18H20',
    bell: 'M6 9.5a6 6 0 0 1 12 0c0 3 .7 5 1.5 6.5h-15C5.3 14.5 6 12.5 6 9.5zM10 19.5a2 2 0 0 0 4 0',
    'bell-off':
      'M3 3l18 18M8.7 4.6A6 6 0 0 1 18 9.5c0 3 .7 5 1.5 6.5H9M6.2 8.3A6 6 0 0 0 6 9.5c0 3-.7 5-1.5 6.5H15M10 19.5a2 2 0 0 0 4 0',
    globe:
      'M12 3a9 9 0 1 0 0 18 9 9 0 0 0 0-18zM3 12h18M12 3c2.5 2.6 3.8 5.6 3.8 9s-1.3 6.4-3.8 9c-2.5-2.6-3.8-5.6-3.8-9S9.5 5.6 12 3z',
    lock: 'M6 10h12a1 1 0 0 1 1 1v8a1 1 0 0 1-1 1H6a1 1 0 0 1-1-1v-8a1 1 0 0 1 1-1zM8 10V7a4 4 0 0 1 8 0v3',
    external: 'M14 4h6v6M20 4l-9 9M18 14v5a1 1 0 0 1-1 1H5a1 1 0 0 1-1-1V7a1 1 0 0 1 1-1h5',
    alert:
      'M12 8v5M12 16.5v.5M10.3 4.2 2.6 18a2 2 0 0 0 1.7 3h15.4a2 2 0 0 0 1.7-3L13.7 4.2a2 2 0 0 0-3.4 0z',
  } as const;

  export type IconName = keyof typeof PATHS;
</script>

<script lang="ts">
  interface Props {
    name: IconName;
    size?: number;
    strokeWidth?: number;
  }

  let { name, size = 18, strokeWidth = 1.8 }: Props = $props();
</script>

<svg
  width={size}
  height={size}
  viewBox="0 0 24 24"
  fill="none"
  stroke="currentColor"
  stroke-width={strokeWidth}
  stroke-linecap="round"
  stroke-linejoin="round"
  aria-hidden="true"
>
  <path d={PATHS[name]} />
</svg>
