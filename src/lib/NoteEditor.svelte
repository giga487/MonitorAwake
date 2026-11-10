<script lang="ts">
  import NoteSharing from './NoteSharing.svelte';
  import { onMount } from 'svelte';
  import { getCurrentWindow } from '@tauri-apps/api/window';
  import { invoke } from '@tauri-apps/api/core';
  import {
    dispatch,
    remaining,
    type Note,
    type NoteColor,
    type Snapshot,
    type Action,
  } from './api';
  import {
    durationUnits,
    notificationSeconds,
    type DurationUnit,
  } from './widget';
  import Icon from './Icon.svelte';
  import { createSaveQueue } from './save-queue';
  let {
    note,
    snapshot,
    native = false,
    onstate,
  }: {
    note: Note;
    snapshot: Snapshot;
    native?: boolean;
    onstate: (state: Snapshot) => void;
  } = $props();
  let title = $state('');
  let body = $state('');
  let loadedId = $state('');
  let revision = $state(-1);
  let dirty = $state(false);
  let saving = $state(false);
  let error = $state('');
  let deleting = $state(false);
  let reminder = $state(false);
  let amount = $state<number | undefined>(5);
  let unit = $state<DurationUnit>('minutes');
  let timer: ReturnType<typeof setTimeout>;
  let geometryTimer: ReturnType<typeof setTimeout>;
  let geometryDirty = false;
  let geometryGeneration = 0;
  const contentQueue = createSaveQueue(saveContent, () => dirty);
  const geometryQueue = createSaveQueue(
    writeGeometry,
    () => native && geometryDirty,
  );
  const linked = $derived(
    snapshot.timers.find((timer) => timer.id === note.reminder_id),
  );
  const activeReminder = $derived(
    linked && (linked.status === 'running' || linked.status === 'expired'),
  );
  $effect(() => {
    if (
      loadedId !== note.id ||
      (!dirty && !saving && revision !== note.content_revision)
    ) {
      loadedId = note.id;
      revision = note.content_revision;
      title = note.title;
      body = note.body;
      dirty = false;
    }
  });
  async function send(action: Action) {
    const result = await dispatch(action);
    onstate(result.state);
    return result.state;
  }
  async function saveContent(): Promise<boolean> {
    const sentTitle = title,
      sentBody = body;
    saving = true;
    error = '';
    try {
      const next = await send({
        type: 'update_note',
        id: note.id,
        expected_revision: revision,
        title: sentTitle,
        body: sentBody,
      });
      const saved = next.notes.find((item) => item.id === note.id);
      if (saved) revision = saved.content_revision;
      dirty = title !== sentTitle || body !== sentBody;
      return true;
    } catch (e) {
      error = String(e);
      return false;
    } finally {
      saving = false;
    }
  }
  function flush(): Promise<boolean> {
    clearTimeout(timer);
    return contentQueue();
  }
  function changed() {
    dirty = true;
    clearTimeout(timer);
    timer = setTimeout(() => {
      void flush();
    }, 350);
  }
  function saveGeometry(): Promise<boolean> {
    clearTimeout(geometryTimer);
    return geometryQueue();
  }
  async function writeGeometry(): Promise<boolean> {
    const generation = geometryGeneration;
    try {
      const win = getCurrentWindow();
      if (await win.isMinimized()) {
        geometryDirty = false;
        return true;
      }
      const [position, size, scale] = await Promise.all([
        win.outerPosition(),
        win.innerSize(),
        win.scaleFactor(),
      ]);
      await send({
        type: 'set_note_bounds',
        id: note.id,
        bounds: {
          x: position.x,
          y: position.y,
          width: Math.max(240, Math.min(1600, size.width / scale)),
          height: Math.max(200, Math.min(1600, size.height / scale)),
        },
      });
      geometryDirty = generation !== geometryGeneration;
      return true;
    } catch (e) {
      error = String(e);
      return false;
    }
  }
  async function action(value: Action) {
    if (!(await flush())) return;
    try {
      error = '';
      await send(value);
      deleting = false;
      reminder = false;
    } catch (e) {
      error = String(e);
    }
  }
  async function drag(event: PointerEvent) {
    if (
      !native ||
      event.button !== 0 ||
      (event.target instanceof Element && event.target.closest('button'))
    )
      return;
    try {
      await getCurrentWindow().startDragging();
    } catch (e) {
      error = String(e);
    }
  }
  onMount(() => {
    let stopped = false;
    const unlisten: (() => void)[] = [];
    const attach = async () => {
      if (!native) return;
      const win = getCurrentWindow();
      const boundsChanged = () => {
        geometryGeneration++;
        geometryDirty = true;
        clearTimeout(geometryTimer);
        geometryTimer = setTimeout(() => {
          void saveGeometry();
        }, 400);
      };
      const registrations = await Promise.all([
        win.onMoved(boundsChanged),
        win.onResized(boundsChanged),
        win.listen<string>('save-before-quit', async (event) => {
          const saved = (await flush()) && (await saveGeometry());
          await invoke('note_flush_done', { token: event.payload, saved });
        }),
      ]);
      if (stopped) registrations.forEach((fn) => fn());
      else unlisten.push(...registrations);
      // Persist a restored position after the native monitor fallback has clamped it.
      boundsChanged();
    };
    void attach().catch((e) => (error = String(e)));
    return () => {
      stopped = true;
      clearTimeout(timer);
      clearTimeout(geometryTimer);
      unlisten.forEach((fn) => fn());
    };
  });
</script>

<article
  class="sticky-note"
  class:native
  data-color={note.color}
  aria-label="Sticky note"
>
  <div
    class="note-bar"
    onpointerdown={drag}
    role="toolbar"
    aria-label="Note controls"
    tabindex="0"
  >
    <span>{note.pinned ? 'Pinned' : 'Note'}</span>
    <button
      aria-label="Keep note on top"
      aria-pressed={note.pinned}
      onclick={() =>
        action({
          type: 'set_note_appearance',
          id: note.id,
          color: note.color,
          pinned: !note.pinned,
        })}><Icon name="pin" size={16} /></button
    >
    {#if native}<button
        aria-label="Minimize note"
        onclick={() =>
          getCurrentWindow()
            .minimize()
            .catch((e) => (error = String(e)))}
        ><Icon name="minus" size={16} /></button
      >{/if}
    <button
      aria-label="Hide note"
      onclick={() =>
        action({ type: 'set_note_visible', id: note.id, visible: false })}
      ><Icon name="close" size={16} /></button
    >
  </div>
  <input
    class="note-title"
    aria-label="Note title"
    bind:value={title}
    oninput={changed}
    maxlength="80"
    placeholder="Untitled note"
  />
  <textarea
    class="note-body"
    aria-label="Note text"
    bind:value={body}
    oninput={changed}
    maxlength="10000"
    placeholder="Write something to remember…"></textarea>
  {#if snapshot.desktop_error}<div class="note-error" role="alert">
      {snapshot.desktop_error}
    </div>{/if}
  {#if error}<div class="note-error" role="alert">
      {error}
      <div class="note-error-actions">
        <button
          onclick={() => {
            revision = note.content_revision;
            title = note.title;
            body = note.body;
            dirty = false;
            error = '';
          }}>Reload saved note</button
        ><button
          onclick={() => {
            revision = note.content_revision;
            void flush();
          }}>Save my version</button
        >
      </div>
    </div>{/if}
  {#if activeReminder && linked}<div class="note-reminder">
      {linked.status === 'expired'
        ? 'Reminder ready'
        : `Reminder: ${remaining(linked, snapshot.now_ms)}`}<button
        onclick={() =>
          action({
            type:
              linked.status === 'expired'
                ? 'acknowledge_timer'
                : 'cancel_timer',
            id: linked.id,
          })}>{linked.status === 'expired' ? 'Dismiss' : 'Cancel'}</button
      >
    </div>{/if}
  {#if reminder && !activeReminder}<form
      class="note-reminder-form"
      onsubmit={(event) => {
        event.preventDefault();
        const seconds = notificationSeconds(amount, unit);
        if (seconds)
          void action({ type: 'set_note_reminder', id: note.id, seconds });
      }}
    >
      <label
        >Remind me in<input
          type="number"
          min="1"
          max="100"
          step="1"
          required
          bind:value={amount}
        /></label
      ><select aria-label="Reminder unit" bind:value={unit}
        >{#each durationUnits as choice}<option value={choice.value}
            >{choice.label}</option
          >{/each}</select
      ><button disabled={!notificationSeconds(amount, unit)}
        >Set reminder</button
      >
    </form>{/if}
  {#if deleting}<div class="note-delete" role="alert">
      {snapshot?.settings.notes_storage === 'firebase'
        ? 'Delete this note for everyone sharing it? Its local reminder will also be removed.'
        : 'Delete this note and its reminder permanently?'}
      <div>
        <button onclick={() => (deleting = false)}>Keep note</button><button
          onclick={() => action({ type: 'delete_note', id: note.id })}
          >Delete permanently</button
        >
      </div>
    </div>{/if}
  <NoteSharing {note} {snapshot} {onstate} beforeAction={flush} />
  <footer class="note-footer">
    <div class="note-colors" aria-label="Note color">
      {#each ['yellow', 'sage', 'blue', 'rose'] as color}<button
          data-color={color}
          aria-label={`${color} paper`}
          aria-pressed={note.color === color}
          onclick={() =>
            action({
              type: 'set_note_appearance',
              id: note.id,
              color: color as NoteColor,
              pinned: note.pinned,
            })}
        ></button>{/each}
    </div>
    <button
      aria-label="Add a reminder"
      disabled={!!activeReminder}
      onclick={() => (reminder = !reminder)}
      ><Icon name="timer" size={16} /></button
    ><button aria-label="Delete note" onclick={() => (deleting = !deleting)}
      ><Icon name="trash" size={16} /></button
    ><span class="note-save-status" aria-live="polite"
      >{saving
        ? 'Saving…'
        : dirty
          ? 'Unsaved'
          : snapshot?.settings.notes_storage === 'firebase'
            ? 'Saved locally'
            : 'Saved'}</span
    >
  </footer>
</article>

<style>
  .sticky-note {
    --paper: #fff0b8;
    --bar: #f8e7a4;
    --ink: #51472c;
    display: flex;
    flex-direction: column;
    min-height: 320px;
    height: 100%;
    background: var(--paper);
    color: var(--ink);
    border-radius: 10px;
    overflow: auto;
    box-shadow: 0 5px 20px #30251b1a;
    font-size: 13px;
  }
  .sticky-note.native {
    height: 100vh;
    min-height: 0;
    border-radius: 0;
    box-shadow: none;
  }
  [data-color='sage'] {
    --paper: #e1ebd1;
    --bar: #d4e3bf;
    --ink: #3c5137;
  }
  [data-color='blue'] {
    --paper: #dcebf4;
    --bar: #cce1ed;
    --ink: #334d5a;
  }
  [data-color='rose'] {
    --paper: #f5dfdd;
    --bar: #edcfcb;
    --ink: #644442;
  }
  .sticky-note button {
    display: inline-flex;
    justify-content: center;
    align-items: center;
    background: transparent;
    color: inherit;
    border: 0;
    border-radius: 5px;
    padding: 5px;
    cursor: pointer;
    font-size: 11px;
  }
  .sticky-note button:hover {
    background: #0000000c;
  }
  .sticky-note button:disabled {
    opacity: 0.45;
  }
  .note-bar {
    flex-shrink: 0;
    display: flex;
    align-items: center;
    gap: 3px;
    padding: 5px 8px;
    background: var(--bar);
    cursor: grab;
    user-select: none;
  }
  .note-bar > span {
    margin-right: auto;
    font-size: 11px;
  }
  .note-bar [aria-pressed='true'] {
    background: #ffffff77;
  }
  .sticky-note .note-title {
    flex-shrink: 0;
    border: 0;
    border-radius: 0;
    background: transparent;
    color: inherit;
    padding: 13px 16px 7px;
    width: 100%;
    font-size: 17px;
    font-weight: 600;
    outline-offset: -3px;
  }
  .note-body {
    flex: 1;
    min-height: 50px;
    resize: none;
    border: 0;
    background: transparent;
    color: inherit;
    padding: 4px 16px 12px;
    font: inherit;
    line-height: 1.8;
    outline-offset: -3px;
  }
  .note-footer {
    flex-shrink: 0;
    color: var(--ink);
    display: flex;
    align-items: center;
    gap: 5px;
    margin: 0;
    padding: 7px 10px;
    border: 0;
  }
  .note-colors {
    display: flex;
    gap: 6px;
    margin-right: auto;
  }
  .note-colors button {
    width: 16px;
    height: 16px;
    padding: 0;
    border-radius: 50%;
    background: #f6dc83;
    border: 1px solid #7b704f44;
  }
  .note-colors [data-color='sage'] {
    background: #b9d497;
  }
  .note-colors [data-color='blue'] {
    background: #a6cfdf;
  }
  .note-colors [data-color='rose'] {
    background: #dfaca4;
  }
  .note-colors [aria-pressed='true'] {
    outline: 2px solid var(--ink);
    outline-offset: 2px;
  }
  .note-save-status {
    font-size: 10px;
  }
  .note-error,
  .note-delete,
  .note-reminder,
  .note-reminder-form {
    padding: 7px 12px;
    font-size: 11px;
    border-top: 1px solid #00000015;
  }
  .note-error {
    color: #8b3027;
    max-height: 130px;
    overflow: auto;
  }
  .note-error-actions,
  .note-delete > div {
    display: flex;
    gap: 6px;
    flex-wrap: wrap;
  }
  .note-reminder {
    display: flex;
    justify-content: space-between;
    align-items: center;
  }
  .note-reminder-form {
    display: flex;
    align-items: end;
    gap: 6px;
    flex-wrap: wrap;
  }
  .note-reminder-form label {
    font-size: 11px;
  }
  .note-reminder-form input {
    display: block;
    width: 65px;
    padding: 5px;
    border: 1px solid #0002;
    background: #fff8;
  }
  .note-reminder-form select {
    font: inherit;
    padding: 5px;
    border: 1px solid #0002;
    border-radius: 5px;
    background: #fff8;
  }
</style>
