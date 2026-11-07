<script lang="ts">
  import { Window } from '@tauri-apps/api/window';
  import { dispatch, desktop, type Snapshot, type Action } from './api';
  import Icon from './Icon.svelte';
  import NoteEditor from './NoteEditor.svelte';
  let {
    snapshot,
    onstate,
    compact = false,
  }: {
    snapshot: Snapshot;
    onstate: (state: Snapshot) => void;
    compact?: boolean;
  } = $props();
  let selected = $state<string | null>(null);
  let error = $state('');
  let busy = $state(false);
  const note = $derived(snapshot.notes.find((item) => item.id === selected));
  async function act(action: Action) {
    if (busy) return;
    busy = true;
    error = '';
    try {
      const result = await dispatch(action);
      onstate(result.state);
      if (
        desktop &&
        action.type === 'set_all_notes_visible' &&
        action.visible
      ) {
        for (const item of result.state.notes) {
          const win = await Window.getByLabel(`note-${item.id}`);
          await win?.unminimize();
          await win?.show();
        }
      }
      if (action.type === 'create_note')
        selected = result.state.notes.at(-1)?.id ?? null;
    } catch (e) {
      error = String(e);
    } finally {
      busy = false;
    }
  }
  async function open(id: string) {
    selected = id;
    await act({ type: 'set_note_visible', id, visible: true });
    if (desktop) {
      try {
        const win = await Window.getByLabel(`note-${id}`);
        await win?.show();
        await win?.unminimize();
        await win?.setFocus();
      } catch (e) {
        error = String(e);
      }
    }
  }
</script>

<section class="notes-panel" class:compact aria-label="My notes">
  <div class="notes-heading">
    <h2>My notes</h2>
    <button
      class="primary"
      disabled={busy}
      onclick={() => act({ type: 'create_note' })}
      ><Icon name="plus" size={16} /> New note</button
    >
  </div>
  <div class="notes-visibility">
    <button
      class="text-button"
      disabled={busy}
      onclick={() => act({ type: 'set_all_notes_visible', visible: true })}
      >Show all notes</button
    ><button
      class="text-button"
      disabled={busy}
      onclick={() => act({ type: 'set_all_notes_visible', visible: false })}
      >Hide all notes</button
    >
  </div>
  {#if error}<p class="notice error" role="alert">{error}</p>{/if}
  {#if !snapshot.notes.length}<p class="notes-empty">
      Keep a thought on your desktop. Create your first note.
    </p>{/if}
  <div class="notes-list">
    {#each snapshot.notes as item (item.id)}<article
        class="note-row"
        data-color={item.color}
      >
        <div>
          <strong>{item.title || 'Untitled note'}</strong>
          <p>
            {item.body.slice(0, 80) ||
              'An empty page, ready for your thoughts.'}
          </p>
          <small
            >{item.visible ? 'On desktop' : 'Hidden'}{item.pinned
              ? ' · Pinned'
              : ''}</small
          >
        </div>
        <button class="secondary" disabled={busy} onclick={() => open(item.id)}
          >Open</button
        >
      </article>{/each}
  </div>
  {#if !desktop && note}<div class="web-note-editor">
      {#key note.id}<NoteEditor {note} {snapshot} {onstate} />{/key}
    </div>{/if}
  <p class="footnote">
    Notes save automatically. Closing a note hides it; deletion is a separate
    action.
  </p>
</section>

<style>
  .notes-heading {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 12px;
    flex-wrap: wrap;
  }
  .notes-visibility {
    display: flex;
    gap: 18px;
    margin: 12px 0;
    flex-wrap: wrap;
  }
  .notes-empty {
    font-size: 12px;
    color: #7b837c;
    padding: 15px 0;
  }
  .notes-list {
    display: grid;
    gap: 10px;
  }
  .note-row {
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 13px;
    background: #fff4ce;
    border-left: 3px solid #d9b84a;
    border-radius: 7px;
  }
  .note-row[data-color='sage'] {
    background: #edf3e5;
    border-color: #89a46c;
  }
  .note-row[data-color='blue'] {
    background: #eaf2f7;
    border-color: #7bafc5;
  }
  .note-row[data-color='rose'] {
    background: #f9ecea;
    border-color: #ce958b;
  }
  .note-row > div {
    flex: 1;
    min-width: 0;
  }
  .note-row strong {
    font-size: 13px;
    overflow-wrap: anywhere;
  }
  .note-row p {
    font-size: 11px;
    color: #67705f;
    margin-top: 4px;
    overflow-wrap: anywhere;
    white-space: pre-wrap;
  }
  .note-row small {
    font-size: 10px;
    color: #7b837c;
  }
  .web-note-editor {
    max-width: 500px;
    margin-top: 22px;
  }
  .compact h2 {
    font-size: 14px;
  }
  .compact .primary {
    padding: 7px 9px;
  }
  .compact .note-row {
    padding: 9px;
  }
  .compact .note-row p {
    display: none;
  }
</style>
