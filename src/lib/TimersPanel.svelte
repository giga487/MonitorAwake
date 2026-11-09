<script lang="ts">
  import { dispatch, remaining, type Snapshot, type Action } from './api';
  let {
    snapshot,
    now,
    onstate,
  }: { snapshot: Snapshot; now: number; onstate: (state: Snapshot) => void } =
    $props();
  let busy = $state(false);
  let error = $state('');
  const running = $derived(
    snapshot.timers
      .filter((timer) => timer.status === 'running')
      .sort((a, b) => a.deadline_ms - b.deadline_ms),
  );
  const expired = $derived(
    snapshot.timers.filter((timer) => timer.status === 'expired'),
  );
  const history = $derived(
    snapshot.timers
      .filter(
        (timer) =>
          timer.status === 'acknowledged' || timer.status === 'cancelled',
      )
      .slice()
      .reverse(),
  );
  async function act(action: Action) {
    if (busy) return;
    busy = true;
    error = '';
    try {
      const result = await dispatch(action);
      onstate(result.state);
    } catch (e) {
      error = String(e);
    } finally {
      busy = false;
    }
  }
</script>

<section class="compact-timers" aria-label="My timers">
  <h2>My timers</h2>
  {#if error}<p class="widget-error" role="alert">{error}</p>{/if}
  {#each expired as timer (timer.id)}<article class="compact-timer due">
      <strong>{timer.name}</strong><span>Time’s up</span><button
        class="secondary"
        disabled={busy}
        onclick={() => act({ type: 'acknowledge_timer', id: timer.id })}
        >Dismiss</button
      >
    </article>{/each}
  <h3>Running · {running.length}</h3>
  {#if !running.length}<p class="timer-empty">
      No active timers. Completed and cancelled timers stay in History.
    </p>{/if}
  {#each running as timer (timer.id)}<article class="compact-timer">
      <strong>{timer.name}</strong><span class="compact-countdown"
        >{remaining(timer, now)}</span
      >
      <div>
        <button
          class="text-button"
          disabled={busy}
          onclick={() =>
            act({ type: 'extend_timer', id: timer.id, seconds: 300 })}
          >+5 min</button
        ><button
          class="text-button"
          disabled={busy}
          onclick={() => act({ type: 'cancel_timer', id: timer.id })}
          >Cancel</button
        >
      </div>
    </article>{/each}
  <details open>
    <summary>History · {history.length}</summary>{#if !history.length}<p
        class="timer-empty"
      >
        No completed timers yet.
      </p>{/if}{#each history as timer (timer.id)}<div class="compact-history">
        <strong>{timer.name}</strong><small
          >{timer.status === 'cancelled' ? 'Cancelled' : 'Completed'}</small
        >
      </div>{/each}
  </details>
</section>

<style>
  .compact-timers h2 {
    font-size: 14px;
    margin: 10px 0;
  }
  .compact-timers h3 {
    font-size: 12px;
    margin: 14px 0 8px;
  }
  .timer-empty {
    font-size: 11px;
    line-height: 1.6;
    color: #7b837c;
    margin: 8px 0 14px;
  }
  .compact-timer {
    display: flex;
    flex-direction: column;
    gap: 6px;
    background: #f1f5e9;
    border: 1px solid #dbe4d1;
    border-radius: 9px;
    padding: 11px;
    margin: 8px 0;
  }
  .compact-timer strong {
    font-size: 12px;
    overflow-wrap: anywhere;
  }
  .compact-timer > span {
    font-size: 11px;
  }
  .compact-timer .compact-countdown {
    font-size: 23px;
    font-variant-numeric: tabular-nums;
    color: #476550;
  }
  .compact-timer > div {
    display: flex;
    justify-content: space-between;
  }
  .due {
    background: #fff2df;
  }
  .due .secondary {
    align-self: start;
  }
  .compact-timers summary {
    font-size: 12px;
    cursor: pointer;
    padding: 10px 0;
  }
  .compact-history {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 12px;
    border-top: 1px solid #e6e9e1;
    padding: 9px 0;
    font-size: 11px;
  }
  .compact-history strong {
    font-weight: 500;
    overflow-wrap: anywhere;
  }
  .compact-history small {
    color: #7b837c;
  }
</style>
