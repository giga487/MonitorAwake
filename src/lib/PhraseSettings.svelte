<script lang="ts">
  import { dispatch, type Snapshot } from './api';
  let {
    snapshot,
    onstate,
  }: { snapshot: Snapshot; onstate: (state: Snapshot) => void } = $props();
  let text = $state('');
  let enabled = $state(true);
  let dirty = $state(false);
  let busy = $state(false);
  let message = $state('');
  let error = $state('');
  $effect(() => {
    if (!dirty && !busy) {
      text = snapshot.settings.phrases.join('\n');
      enabled = snapshot.settings.speech_enabled;
    }
  });
  const phrases = $derived(
    text
      .split(/\r?\n/)
      .map((line) => line.trim())
      .filter(Boolean),
  );
  const valid = $derived(
    phrases.length <= 50 && phrases.every((line) => [...line].length <= 180),
  );
  async function save(event: SubmitEvent) {
    event.preventDefault();
    if (!valid) return;
    busy = true;
    error = '';
    message = '';
    try {
      const result = await dispatch({
        type: 'update_settings',
        settings: { ...snapshot.settings, phrases, speech_enabled: enabled },
      });
      onstate(result.state);
      dirty = false;
      message = 'Phrases saved.';
    } catch (e) {
      error = String(e);
    } finally {
      busy = false;
    }
  }
</script>

<section
  class="phrase-settings panel preferences"
  aria-label="Recurring phrases"
>
  <h2>Recurring phrases</h2>
  <p>Choose the little thoughts your mascot can share in a speech bubble.</p>
  <form onsubmit={save}>
    <label class="phrase-toggle"
      ><input
        type="checkbox"
        bind:checked={enabled}
        onchange={() => (dirty = true)}
      /><span>Let the mascot say a phrase occasionally</span></label
    >
    <label for="recurring-phrases">One phrase per line</label><textarea
      id="recurring-phrases"
      bind:value={text}
      oninput={() => (dirty = true)}
      rows="7"
      maxlength="10000"
      spellcheck="true"></textarea>
    <p class="footnote">
      {phrases.length}/50 phrases · Up to 180 characters each. Empty lines are
      ignored. Phrases pause while notifications are pending or motion is
      disabled.
    </p>
    {#if !valid}<p role="alert" class="phrase-error">
        Use at most 50 phrases with up to 180 characters each.
      </p>{/if}
    {#if error}<p role="alert" class="phrase-error">{error}</p>{/if}
    <div class="phrase-actions">
      <button class="primary" disabled={busy || !valid || !dirty}
        >Save phrases</button
      ><span aria-live="polite">{message}</span>
    </div>
  </form>
</section>

<style>
  .phrase-settings label {
    display: block;
    font-size: 12px;
    margin: 16px 0 7px;
  }
  .phrase-settings .phrase-toggle {
    display: flex;
    align-items: center;
    gap: 10px;
  }
  .phrase-settings textarea {
    display: block;
    width: 100%;
    resize: vertical;
    min-height: 150px;
    border: 1px solid #dbe4d1;
    border-radius: 9px;
    padding: 12px;
    font: inherit;
    font-size: 13px;
    line-height: 1.8;
    color: #303b34;
    background: #fff;
  }
  .phrase-actions {
    display: flex;
    align-items: center;
    gap: 15px;
    flex-wrap: wrap;
    margin-top: 15px;
  }
  .phrase-actions span {
    font-size: 12px;
    color: #476550;
  }
  .phrase-error {
    font-size: 12px;
    color: #8b4038;
    margin-top: 10px;
  }
</style>
