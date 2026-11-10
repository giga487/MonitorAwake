<script lang="ts">
  import { dispatch, type Note, type Snapshot, type Action } from './api';
  let {
    note,
    snapshot,
    onstate,
    beforeAction,
  }: {
    note: Note;
    snapshot: Snapshot;
    onstate: (state: Snapshot) => void;
    beforeAction?: () => Promise<boolean>;
  } = $props();
  let uid = $state(''),
    busy = $state(false),
    error = $state('');
  const entry = $derived(
    snapshot.note_sync.entries.find((entry) => entry.id === note.id),
  );
  const scope = $derived(snapshot.note_sync.scope);
  async function act(action: Action) {
    if (beforeAction && !(await beforeAction())) {
      error =
        'Save the note edits before changing sharing or resolving a conflict.';
      return;
    }
    busy = true;
    error = '';
    try {
      onstate((await dispatch(action)).state);
      uid = '';
    } catch (reason) {
      error = String(reason);
    } finally {
      busy = false;
    }
  }
  function invite(event: SubmitEvent) {
    event.preventDefault();
    if (entry && uid.trim())
      void act({
        type: 'set_note_members',
        id: note.id,
        member_uids: [...new Set([...entry.member_uids, uid.trim()])],
      });
  }
</script>

{#if snapshot.settings.notes_storage === 'firebase'}
  <details class="note-sharing">
    <summary>Cloud sync and sharing</summary>
    {#if entry?.access_removed}<p>
        Cloud access was removed. This cached local copy is kept and will not be
        uploaded.
      </p>
    {:else if entry?.conflict}<p role="status">
        This note changed on another device. Both versions are kept.
      </p>
      <details>
        <summary>View cloud version</summary><strong
          >{entry.conflict.title}</strong
        >
        <pre>{entry.conflict.body}</pre>
      </details>
      <div class="actions">
        <button
          disabled={busy}
          onclick={() =>
            act({
              type: 'resolve_note_conflict',
              id: note.id,
              keep_local: true,
            })}>Keep my version</button
        ><button
          disabled={busy}
          onclick={() =>
            act({
              type: 'resolve_note_conflict',
              id: note.id,
              keep_local: false,
            })}>Use cloud version</button
        >
      </div>
    {:else if entry && scope}
      <p>
        Cloud revision {entry.base.revision}. {entry.base.owner_uid ===
        scope.uid
          ? 'You own this note.'
          : 'Shared with you; you can edit its content.'}
      </p>
      {#if entry.base.owner_uid === scope.uid}
        <form onsubmit={invite}>
          <label
            >Share with account ID<input
              bind:value={uid}
              maxlength="128"
              required
              placeholder="Ask the other person for their Firebase account ID"
            /></label
          ><button
            disabled={busy || !uid.trim() || entry.member_uids.length >= 20}
            >Share note</button
          >
        </form>
        {#each entry.member_uids.filter((id) => id !== scope.uid) as member}<div
            class="member"
          >
            <code>{member}</code><button
              disabled={busy}
              onclick={() =>
                act({
                  type: 'set_note_members',
                  id: note.id,
                  member_uids: entry.member_uids.filter((id) => id !== member),
                })}>Remove access</button
            >
          </div>{/each}
      {/if}
    {:else}<p>
        Sharing becomes available after this note’s first successful upload.
      </p>{/if}
    {#if error}<p class="error" role="alert">{error}</p>{/if}
  </details>
{/if}

<style>
  .note-sharing {
    font-size: 11px;
    margin: 8px 12px;
    line-height: 1.6;
  }
  summary {
    cursor: pointer;
  }
  p {
    margin: 8px 0;
  }
  form,
  label {
    display: grid;
    gap: 7px;
  }
  input {
    min-width: 0;
    width: 100%;
    box-sizing: border-box;
    font: inherit;
    padding: 7px;
    border: 1px solid #dbe4d1;
    border-radius: 6px;
  }
  button {
    font: inherit;
    padding: 5px 8px;
    border: 1px solid #c7d0bd;
    border-radius: 6px;
    background: white;
    cursor: pointer;
  }
  .actions,
  .member {
    display: flex;
    gap: 8px;
    flex-wrap: wrap;
    margin: 8px 0;
  }
  code {
    overflow-wrap: anywhere;
  }
  pre {
    white-space: pre-wrap;
    overflow-wrap: anywhere;
    max-height: 180px;
    overflow: auto;
  }
  .error {
    color: #8b4038;
  }
</style>
