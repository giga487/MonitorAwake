<script lang="ts">
  let {
    name,
    size = 20,
  }: {
    name:
      | 'timer'
      | 'settings'
      | 'plus'
      | 'close'
      | 'terminal'
      | 'arrow'
      | 'power'
      | 'check'
      | 'minus'
      | 'note'
      | 'pin'
      | 'trash'
      | 'chat';
    size?: number;
  } = $props();
  const paths = {
    timer:
      'M10 2h4M12 5v2M19 6l-2 2M12 11v4l3 2M21 14a9 9 0 1 1-18 0 9 9 0 0 1 18 0Z',
    settings:
      'm9 3-1 3-3 1-2 3 2 3v4l3 1 2 3h4l2-3 3-1v-4l2-3-2-3-3-1-1-3H9ZM15 12a3 3 0 1 1-6 0 3 3 0 0 1 6 0Z',
    plus: 'M12 5v14M5 12h14',
    close: 'm6 6 12 12M18 6 6 18',
    terminal: 'M4 4h16v16H4ZM7 9l3 3-3 3M13 15h4',
    arrow: 'M5 12h14m-6-6 6 6-6 6',
    power: 'M12 2v10M6 5a9 9 0 1 0 12 0',
    check: 'm5 12 4 4L19 6',
    minus: 'M5 12h14',
    note: 'M4 3h16v12l-6 6H4V3Zm10 18v-6h6M8 8h8M8 12h5',
    pin: 'm9 3 6 0-1 6 4 4v2H6v-2l4-4-1-6ZM12 15v6',
    trash: 'M4 6h16M9 6V3h6v3M6 6l1 15h10l1-15M10 10v7M14 10v7',
    chat: 'M4 4h16v12H9l-5 4V4ZM8 8h8M8 12h5',
  };
</script>

<svg
  width={size}
  height={size}
  viewBox="0 0 24 24"
  fill="none"
  stroke="currentColor"
  stroke-width="1.7"
  stroke-linecap="round"
  stroke-linejoin="round"
  aria-hidden="true"><path d={paths[name]} /></svg
>
