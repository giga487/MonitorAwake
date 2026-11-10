<script lang="ts">
  import { dispatch, type Snapshot } from './api';
  let {
    snapshot,
    onstate,
  }: { snapshot: Snapshot; onstate: (state: Snapshot) => void } = $props();
  let busy = $state(false),
    error = $state('');
  async function change(storage: 'local' | 'firebase') {
    busy = true;
    error = '';
    try {
      onstate((await dispatch({ type: 'set_notes_storage', storage })).state);
    } catch (reason) {
      error = String(reason);
    } finally {
      busy = false;
    }
  }
  async function disconnect() {
    busy = true;
    error = '';
    try {
      onstate((await dispatch({ type: 'disconnect_notes_cloud' })).state);
    } catch (reason) {
      error = String(reason);
    } finally {
      busy = false;
    }
  }
</script>

<section class="notes-storage" aria-label="Notes storage">
  <strong
    >Notes storage: {snapshot.settings.notes_storage === 'firebase'
      ? 'Firebase'
      : 'Local'}</strong
  >
  {#if snapshot.settings.notes_storage === 'local'}
    <p>
      Notes stay on this computer. Enabling Firebase copies the current notes to
      your signed-in account and syncs future changes.
    </p>
    <button class="secondary" disabled={busy} onclick={() => change('firebase')}
      >Use Firebase notes</button
    >
  {:else}
    <p aria-live="polite">
      {snapshot.note_sync_status.message ||
        'Starting synchronization…'}{snapshot.note_sync_status.pending
        ? ` (${snapshot.note_sync_status.pending} queued)`
        : ''}
    </p>
    <p>Offline edits remain on this computer until synchronization succeeds.</p>
    <button class="text-button" disabled={busy} onclick={() => change('local')}
      >Use local notes</button
    >
  {/if}
  {#if snapshot.note_sync.scope}<details>
      <summary>Cloud account links</summary>
      <p>
        Disconnecting keeps local copies with new identifiers. The cloud
        originals remain in the original account. Save and close note windows
        first.
      </p>
      <button
        class="text-button"
        disabled={busy || snapshot.notes.some((note) => note.visible)}
        onclick={disconnect}>Disconnect cloud links</button
      >
    </details>{/if}
  {#if error}<p role="alert">{error}</p>{/if}
</section>

<style>
  .notes-storage {
    border: 1px solid #dbe4d1;
    border-radius: 9px;
    padding: 12px;
    margin: 14px 0;
    font-size: 12px;
  }
  p {
    font-size: 11px;
    line-height: 1.6;
    margin: 8px 0;
    color: #657269;
  }
  summary {
    cursor: pointer;
    font-size: 11px;
    margin-top: 10px;
  }
  [role='alert'] {
    color: #8b4038;
    overflow-wrap: anywhere;
  }
</style>
