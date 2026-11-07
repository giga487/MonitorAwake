<script lang="ts">
  import { onMount } from 'svelte';
  import { getCurrentWindow } from '@tauri-apps/api/window';
  let text = $state('');
  onMount(() => {
    let stopped = false;
    let unlisten: () => void = () => {};
    void getCurrentWindow()
      .listen<string>('mascot-phrase', (event) => (text = event.payload))
      .then((fn) => {
        if (stopped) fn();
        else unlisten = fn;
      });
    return () => {
      stopped = true;
      unlisten();
    };
  });
</script>

<div class="quote-window">
  <div class="quote" role="status" aria-live="polite">
    <span>MonitorAwake</span>
    <p>{text}</p>
  </div>
</div>

<style>
  .quote-window {
    height: 100vh;
    padding: 8px 8px 17px;
    display: flex;
    align-items: end;
  }
  .quote {
    position: relative;
    flex: 1;
    padding: 13px 16px;
    background: #fffffff7;
    color: #526644;
    border: 1px solid #dbe4d1;
    border-radius: 15px;
    box-shadow: 0 3px 8px #28382012;
    font-size: 13px;
    line-height: 1.6;
    overflow-wrap: anywhere;
  }
  .quote span {
    font-size: 9px;
    text-transform: uppercase;
    letter-spacing: 1px;
    color: #7b837c;
  }
  .quote p {
    margin-top: 5px;
  }
  .quote:after {
    content: '';
    position: absolute;
    bottom: -11px;
    left: 50%;
    border-top: 11px solid #dbe4d1;
    border-left: 9px solid transparent;
    border-right: 9px solid transparent;
  }
</style>
