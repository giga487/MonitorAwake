<script lang="ts">
  import { dispatch, type Snapshot } from './api';
  import { alarmDeadline, localAlarmValue } from './alarm';
  let {
    snapshot,
    now,
    onstate,
    oncreated,
  }: {
    snapshot: Snapshot | null;
    now: number;
    onstate: (state: Snapshot) => void;
    oncreated?: (message: string) => void;
  } = $props();
  let name = $state('Alarm');
  let dateTime = $state(
    localAlarmValue(Math.ceil((Date.now() + 5 * 60_000) / 60_000) * 60_000),
  );
  let keepAwake = $state(false);
  let busy = $state(false);
  let error = $state('');
  let message = $state('');
  const deadline = $derived(alarmDeadline(dateTime, now));
  async function save(event: SubmitEvent) {
    event.preventDefault();
    if (busy || !snapshot || deadline === null || !name.trim()) return;
    busy = true;
    error = '';
    message = '';
    try {
      const result = await dispatch({
        type: 'add_alarm',
        name,
        deadline_ms: deadline,
        keep_awake: keepAwake,
      });
      onstate(result.state);
      message = result.message;
      oncreated?.(result.message);
    } catch (e) {
      error = String(e);
    } finally {
      busy = false;
    }
  }
</script>

<form class="alarm-form" onsubmit={save} aria-label="New alarm">
  <label for="alarm-name">What should I remind you about?</label>
  <input id="alarm-name" bind:value={name} maxlength="60" required />
  <label for="alarm-date">Date and time</label>
  <input
    id="alarm-date"
    type="datetime-local"
    bind:value={dateTime}
    required
    step="60"
    aria-describedby="alarm-hint"
  />
  <p class="duration-hint" id="alarm-hint">
    Your local time ({Intl.DateTimeFormat().resolvedOptions().timeZone}). Choose
    a future time within 100 days.
  </p>
  <label class="checkbox-row"
    ><input
      type="checkbox"
      bind:checked={keepAwake}
      disabled={!snapshot?.capabilities.system}
    /><span>Keep the computer awake</span></label
  >
  <p class="duration-hint">
    One-time alarm. The app must be running to alert you; overdue alarms appear
    when you reopen it.
  </p>
  {#if deadline === null}<p class="alarm-error" role="alert">
      Choose a valid future date and time within 100 days.
    </p>{/if}
  {#if error}<p class="alarm-error" role="alert">{error}</p>{/if}
  <button
    class="primary full"
    disabled={busy || !snapshot || deadline === null || !name.trim()}
    >Set alarm</button
  >
  <p class="duration-hint" aria-live="polite">{message}</p>
</form>

<style>
  .alarm-form {
    display: grid;
    gap: 7px;
  }
  .checkbox-row {
    display: flex;
    align-items: center;
    gap: 8px;
    line-height: 1.5;
  }
  .duration-hint:empty {
    display: none;
  }
  label {
    font-size: 12px;
  }
  input:not([type='checkbox']) {
    width: 100%;
    min-width: 0;
    box-sizing: border-box;
    padding: 10px;
    border: 1px solid #dbe4d1;
    border-radius: 9px;
    color: #303b34;
    background: white;
    font: inherit;
    font-size: 13px;
  }
  .duration-hint {
    margin: 0;
    font-size: 11px;
    color: #657269;
    line-height: 1.5;
  }
  .alarm-error {
    font-size: 12px;
    color: #8b4038;
  }
</style>
