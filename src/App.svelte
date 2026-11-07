<script lang="ts">
  import { onMount } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';
  import {
    getCurrentWindow,
    currentMonitor,
    monitorFromPoint,
    Window,
    PhysicalPosition,
  } from '@tauri-apps/api/window';
  import Icon from './lib/Icon.svelte';
  import Mascot from './lib/Mascot.svelte';
  import NoteEditor from './lib/NoteEditor.svelte';
  import NotesPanel from './lib/NotesPanel.svelte';
  import TimersPanel from './lib/TimersPanel.svelte';
  import PhraseSettings from './lib/PhraseSettings.svelte';
  import SpeechBubble from './lib/SpeechBubble.svelte';
  import { mascots, type MascotKind } from './lib/mascot';
  import { selectRecurringPhrase } from './lib/mascot';
  import {
    durationUnits,
    notificationSeconds,
    menuPosition,
    type DurationUnit,
  } from './lib/widget';
  import {
    desktop,
    initializeSession,
    getState,
    dispatch,
    remaining,
    modes,
    type Snapshot,
    type Action,
  } from './lib/api';

  let snapshot = $state<Snapshot | null>(null);
  let message = $state('Ready when you are. Tell me when to remind you.');
  let error = $state('');
  let connectionError = $state('');
  let authenticated = $state(true);
  let busy = $state(false);
  let command = $state('');
  let name = $state('Notification');
  let amount = $state<number | undefined>(5);
  let unit = $state<DurationUnit>('minutes');
  let notificationStep = $state<'menu' | 'details' | 'duration'>('menu');
  const durationSeconds = $derived(notificationSeconds(amount, unit));
  let keepAwake = $state(false);
  let now = $state(Date.now());
  let clockOffset = 0;
  let configUrl = $state('');
  const isPanel = desktop && getCurrentWindow().label === 'controls';
  const isSpeech = desktop && getCurrentWindow().label === 'speech';
  const noteId =
    desktop && getCurrentWindow().label.startsWith('note-')
      ? getCurrentWindow().label.slice(5)
      : null;
  const currentNote = $derived(
    snapshot?.notes.find((note) => note.id === noteId),
  );
  let panelSection = $state<'home' | 'notes' | 'timers'>('home');
  const selectedMascotName = $derived(
    mascots.find((mascot) => mascot.id === snapshot?.settings.mascot)?.name ??
      'Chicken',
  );
  let changingMenu = false;

  async function showPhrase(text: string | null) {
    if (!desktop || isPanel || isSpeech || noteId) return;
    try {
      const bubble = await Window.getByLabel('speech');
      if (!bubble) return;
      if (!text) {
        await invoke('set_speech_visible', { visible: false });
        return;
      }
      const mascot = getCurrentWindow();
      const [position, size, bubbleSize, monitor, scale] = await Promise.all([
        mascot.outerPosition(),
        mascot.outerSize(),
        bubble.outerSize(),
        currentMonitor(),
        mascot.scaleFactor(),
      ]);
      const target = menuPosition(
        position,
        size,
        bubbleSize,
        monitor?.workArea ?? null,
        Math.round(5 * scale),
      );
      try {
        await bubble.setPosition(new PhysicalPosition(target.x, target.y));
      } catch {
        /* Compositor chooses a fallback. */
      }
      const panel = await Window.getByLabel('controls');
      if (await panel?.isVisible()) return;
      await bubble.emitTo('speech', 'mascot-phrase', text);
      await invoke('set_speech_visible', { visible: true });
    } catch (e) {
      error = String(e);
    }
  }

  async function selectMascot(mascot: MascotKind) {
    if (snapshot)
      await act({
        type: 'update_settings',
        settings: { ...snapshot.settings, mascot },
      });
  }

  async function setMenu(open: boolean, restoreFocus = true) {
    if (!desktop || changingMenu) return;
    changingMenu = true;
    try {
      const panel = await Window.getByLabel('controls');
      const mascot = await Window.getByLabel('mascot');
      if (!panel || !mascot)
        throw new Error('Desktop menu unavailable. Restart the app.');
      if (open) {
        await invoke('set_speech_visible', { visible: false });
        try {
          const [position, size, panelSize, scale] = await Promise.all([
            mascot.outerPosition(),
            mascot.outerSize(),
            panel.outerSize(),
            mascot.scaleFactor(),
          ]);
          const monitor = await monitorFromPoint(
            position.x + size.width / 2,
            position.y + size.height / 2,
          );
          const target = menuPosition(
            position,
            size,
            panelSize,
            monitor?.workArea ?? null,
            Math.round(6 * scale),
          );
          await panel.setPosition(new PhysicalPosition(target.x, target.y));
        } catch (e) {
          error = `Automatic menu positioning is unavailable: ${String(e)}`;
        }
        await panel.show();
        await panel.setFocus();
      } else {
        await panel.hide();
        notificationStep = 'menu';
        configUrl = '';
        if (restoreFocus) await mascot.setFocus();
      }
    } catch (e) {
      error = String(e);
    } finally {
      changingMenu = false;
    }
  }

  function contextMenu(event: MouseEvent) {
    event.preventDefault();
    void setMenu(true);
  }

  function widgetKey(event: KeyboardEvent) {
    if (event.key === 'Escape') {
      event.preventDefault();
      void setMenu(false);
    } else if (
      event.key === 'ContextMenu' ||
      (event.shiftKey && event.key === 'F10')
    ) {
      event.preventDefault();
      void setMenu(true);
    }
  }

  async function dockMascot() {
    try {
      const win = getCurrentWindow();
      const [monitor, size] = await Promise.all([
        currentMonitor(),
        win.outerSize(),
      ]);
      if (!monitor) return;
      const area = monitor.workArea;
      await win.setPosition(
        new PhysicalPosition(
          Math.max(
            area.position.x,
            area.position.x + area.size.width - size.width - 16,
          ),
          Math.max(
            area.position.y,
            area.position.y + area.size.height - size.height,
          ),
        ),
      );
    } catch (e) {
      error = `Automatic positioning is unavailable: ${String(e)}`;
    }
  }
  let section = $state<'timers' | 'notes' | 'settings'>('timers');
  const running = $derived(
    snapshot?.timers
      .filter((t) => t.status === 'running')
      .sort((a, b) => a.deadline_ms - b.deadline_ms) ?? [],
  );
  const expired = $derived(
    snapshot?.timers.filter((t) => t.status === 'expired') ?? [],
  );
  const history = $derived(
    snapshot?.timers.filter(
      (t) => t.status === 'acknowledged' || t.status === 'cancelled',
    ) ?? [],
  );
  const statusText = $derived(
    !snapshot
      ? 'Connecting…'
      : snapshot.backend_error
        ? 'Request failed'
        : snapshot.applied_mode !== snapshot.requested_mode
          ? 'Updating…'
          : snapshot.applied_mode === 'display'
            ? 'System and display awake'
            : snapshot.applied_mode === 'system'
              ? 'System awake'
              : 'Normal power saving',
  );

  function accept(next: Snapshot) {
    if (!snapshot || next.now_ms >= snapshot.now_ms) {
      snapshot = next;
      clockOffset = next.now_ms - Date.now();
      now = Date.now() + clockOffset;
    }
  }

  onMount(() => {
    document.documentElement.classList.toggle('desktop', desktop);
    if (desktop && !isPanel && !isSpeech && !noteId) void dockMascot();
    authenticated = initializeSession();
    let stopped = false;
    let removePreview = () => {};
    let previewTimer: ReturnType<typeof setTimeout>;
    if (desktop && !isPanel && !isSpeech && !noteId) {
      void getCurrentWindow()
        .listen('preview-phrase', async () => {
          clearTimeout(previewTimer);
          await setMenu(false, false);
          const text = selectRecurringPhrase(
            snapshot?.settings.phrases ?? [],
            null,
          );
          if (text) {
            await showPhrase(text);
            previewTimer = setTimeout(() => {
              void showPhrase(null);
            }, 6000);
          }
        })
        .then((fn) => {
          if (stopped) fn();
          else removePreview = fn;
        })
        .catch((e) => (error = String(e)));
    }
    let timeout: ReturnType<typeof setTimeout>;
    const poll = async () => {
      try {
        const next = await getState();
        if (!stopped) {
          accept(next);
          connectionError = '';
        }
      } catch {
        if (!stopped)
          connectionError =
            'Cannot connect to the app. If you restarted it, reopen settings from the mascot.';
      }
      if (!stopped) timeout = setTimeout(poll, 1000);
    };
    if (authenticated) void poll();
    const clock = setInterval(() => {
      now = Date.now() + clockOffset;
    }, 250);
    return () => {
      stopped = true;
      clearTimeout(timeout);
      clearInterval(clock);
      removePreview();
      clearTimeout(previewTimer);
    };
  });

  async function act(action: Action): Promise<boolean> {
    if (busy) return false;
    busy = true;
    error = '';
    try {
      const result = await dispatch(action);
      accept(result.state);
      message = result.message;
      return true;
    } catch (e) {
      error = e instanceof Error ? e.message : String(e);
      return false;
    } finally {
      busy = false;
    }
  }

  async function sendCommand(event: SubmitEvent) {
    event.preventDefault();
    if (command.trim() && (await act({ type: 'command', text: command })))
      command = '';
  }

  async function addTimer(event: SubmitEvent) {
    event.preventDefault();
    if (durationSeconds === null) return;
    const added = await act({
      type: 'add_timer',
      name,
      seconds: durationSeconds,
      keep_awake: keepAwake,
    });
    if (added) notificationStep = 'menu';
  }

  async function desktopAction(action: string) {
    error = '';
    try {
      await invoke(action);
    } catch (e) {
      error = String(e);
    }
  }

  async function showUrl() {
    try {
      configUrl = await invoke<string>('configuration_url');
    } catch (e) {
      error = String(e);
    }
  }

  async function drag(event: PointerEvent) {
    if (event.button !== 0) return;
    void showPhrase(null);
    try {
      await getCurrentWindow().startDragging();
    } catch (e) {
      error = String(e);
    }
  }
</script>

<svelte:window
  onkeydown={desktop && !noteId && !isSpeech ? widgetKey : undefined}
  onblur={() => {
    if (isPanel) void setMenu(false, false);
  }}
/>

{#snippet notificationWizard()}
  <form
    class="notification-wizard"
    onsubmit={(event) => {
      if (notificationStep !== 'duration') {
        event.preventDefault();
        if (name.trim()) notificationStep = 'duration';
      } else void addTimer(event);
    }}
  >
    {#if notificationStep !== 'duration'}
      <p class="eyebrow">NOTIFICATION · 1 OF 2</p>
      <label for="notification-name">What should I remind you about?</label>
      <input
        id="notification-name"
        bind:value={name}
        maxlength="60"
        required
        placeholder="For example: take a break"
      />
      <button class="primary full" disabled={!name.trim()}
        >Next <Icon name="arrow" size={16} /></button
      >
    {:else}
      <p class="eyebrow">NOTIFICATION · 2 OF 2</p>
      <strong class="notification-name">{name}</strong>
      <label for="notification-amount">Notify me in</label>
      <div class="notification-duration">
        <input
          id="notification-amount"
          type="number"
          bind:value={amount}
          min="0"
          max="100"
          step="1"
          required
          aria-describedby="duration-hint"
        />
        <select aria-label="Time unit" bind:value={unit}>
          {#each durationUnits as choice}
            <option value={choice.value}>{choice.label}</option>
          {/each}
        </select>
      </div>
      <p id="duration-hint" class="duration-hint">
        {durationSeconds === null
          ? 'Choose a whole number from 1 to 100.'
          : `Notification in ${amount} ${unit}.`}
      </p>
      <label class="checkbox-row"
        ><input
          type="checkbox"
          bind:checked={keepAwake}
          disabled={!snapshot?.capabilities.system}
        /><span>Keep the computer awake</span></label
      >
      <div class="notification-actions">
        <button
          type="button"
          class="text-button"
          onclick={() => (notificationStep = 'details')}>Back</button
        >
        <button
          class="primary"
          disabled={busy ||
            !snapshot ||
            durationSeconds === null ||
            !name.trim()}>Start notification</button
        >
      </div>
    {/if}
  </form>
{/snippet}

{#if noteId}
  {#if snapshot && currentNote}<NoteEditor
      note={currentNote}
      {snapshot}
      native
      onstate={accept}
    />{:else}<p class="native-loading">
      {connectionError || 'Loading note…'}
    </p>{/if}
{:else if isSpeech}
  <SpeechBubble />
{:else if desktop}
  <main
    class={isPanel ? 'controls-window' : 'widget'}
    class:still={snapshot && !snapshot.settings.animations}
  >
    {#if isPanel}
      <div
        class="widget-menu"
        role="dialog"
        aria-label="Commands and timers"
        tabindex="-1"
      >
        <div class="widget-top">
          <span>MONITORAWAKE</span>
          <div class="row">
            <button
              class="icon-button small"
              aria-label="Minimize to taskbar"
              onclick={() =>
                Window.getByLabel('mascot')
                  .then((mascot) => mascot?.minimize())
                  .then(() => setMenu(false, false))
                  .catch((e) => (error = String(e)))}
              ><Icon name="minus" size={15} /></button
            >
            <button
              class="icon-button small"
              aria-label="Quit and stop keeping the computer awake"
              onclick={() => desktopAction('quit')}
              ><Icon name="close" size={15} /></button
            >
          </div>
        </div>
        <div class="speech" aria-live="polite">
          {#if expired.length}
            <strong>Time’s up! ✨</strong>
            <span>«{expired[0].name}» has finished.</span>
            <button
              class="text-button"
              disabled={busy}
              onclick={() =>
                act({ type: 'acknowledge_timer', id: expired[0].id })}
              >Dismiss <Icon name="check" size={14} /></button
            >
          {:else if running[0]}
            <span
              >{running[0].name}{running.length > 1
                ? ` · +${running.length - 1} timer`
                : ''}</span
            >
            <strong class="widget-countdown"
              >{remaining(running[0], now)}</strong
            >
          {:else}<span>{message}</span>{/if}
        </div>
        {#if panelSection === 'notes' && snapshot}
          <NotesPanel {snapshot} onstate={accept} compact />
          <button class="text-button" onclick={() => (panelSection = 'home')}
            >Back to menu</button
          >
        {:else if panelSection === 'timers' && snapshot}
          <TimersPanel {snapshot} {now} onstate={accept} />
          <button class="text-button" onclick={() => (panelSection = 'home')}
            >Back to menu</button
          >
        {:else}
          <div class="widget-controls">
            {#if notificationStep === 'menu'}
              <button
                class="menu-item"
                onclick={() => act({ type: 'create_note' })}
                ><Icon name="plus" size={18} /><span>New note</span></button
              >
              <button class="menu-item" onclick={() => (panelSection = 'notes')}
                ><Icon name="note" size={18} /><span>My notes</span><span
                  >{snapshot?.notes.length ?? 0}</span
                ></button
              >
              <button
                class="menu-item"
                onclick={() => (notificationStep = 'details')}
                ><Icon name="timer" size={18} /><span>Notification</span><Icon
                  name="arrow"
                  size={16}
                /></button
              >
              <button
                class="menu-item"
                onclick={() => (panelSection = 'timers')}
                ><Icon name="timer" size={18} /><span>My timers</span><span
                  >{snapshot?.timers.length ?? 0}</span
                ></button
              >
              <details class="advanced-command">
                <summary>Text command</summary>
                <form class="command-form compact" onsubmit={sendCommand}>
                  <input
                    aria-label="Assistant command"
                    bind:value={command}
                    maxlength="300"
                    placeholder="Timer tea 5 minutes…"
                    disabled={!snapshot}
                  />
                  <button
                    class="icon-button"
                    disabled={busy || !command.trim() || !snapshot}
                    aria-label="Send command"
                    ><Icon name="arrow" size={17} /></button
                  >
                </form>
              </details>
            {:else}
              {@render notificationWizard()}
              <button
                class="text-button"
                onclick={() => (notificationStep = 'menu')}>Back to menu</button
              >
            {/if}
            <div class="widget-toolbar">
              <button
                class="text-button awake-button"
                disabled={busy || !snapshot?.capabilities.system}
                onclick={() =>
                  act({
                    type: 'set_awake',
                    mode: snapshot?.manual_mode === 'off' ? 'system' : 'off',
                  })}
                aria-label={snapshot?.manual_mode === 'off'
                  ? 'Keep the system awake'
                  : 'Turn off manual keep-awake'}
              >
                <span
                  class="status-dot"
                  class:active={snapshot?.applied_mode !== 'off' && !!snapshot}
                ></span>
                {snapshot?.applied_mode === 'off' ? 'Paused' : 'Awake'}
              </button>
              <button
                class="icon-button"
                aria-label="Open settings in your browser"
                onclick={() => desktopAction('open_configuration')}
                ><Icon name="settings" size={17} /></button
              >
              <button class="text-button url-button" onclick={showUrl}
                >URL</button
              >
            </div>
          </div>
        {/if}
        <label class="mascot-picker compact-picker" for="desktop-mascot">
          <span>Mascot</span>
          <select
            id="desktop-mascot"
            value={snapshot?.settings.mascot ?? 'chicken'}
            disabled={busy || !snapshot}
            onchange={(event) =>
              selectMascot(event.currentTarget.value as MascotKind)}
          >
            {#each mascots as mascot}<option value={mascot.id}
                >{mascot.name}</option
              >{/each}
          </select>
        </label>
        <p class="egg-legend">
          Eggs: green · plenty of time, yellow · halfway, red · due soon. A
          cracked egg is ready.
        </p>
        <button
          class="text-button"
          disabled={!snapshot?.settings.phrases.length}
          onclick={() =>
            getCurrentWindow()
              .emitTo('mascot', 'preview-phrase')
              .catch((e) => (error = String(e)))}
          ><Icon name="chat" size={15} /> Preview phrase</button
        >
        {#if error || connectionError || snapshot?.backend_error || snapshot?.desktop_error}<p
            class="widget-error"
            role="alert"
          >
            {error ||
              connectionError ||
              snapshot?.backend_error ||
              snapshot?.desktop_error}
          </p>{/if}
        {#if configUrl}<input
            class="url-field"
            aria-label="Private settings URL, select and copy"
            readonly
            value={configUrl}
            onclick={(event) => event.currentTarget.select()}
          />{/if}
        <button class="text-button menu-close" onclick={() => setMenu(false)}
          >Close menu (Esc)</button
        >
      </div>
    {/if}
    {#if !isPanel}
      <button
        class="mascot-drag"
        aria-label={`${selectedMascotName}: ${running.length} pending notifications, ${expired.length} ready. Drag to move, right-click or press Enter for commands.`}
        aria-haspopup="dialog"
        title="Drag to move · Right-click for commands and timers"
        onpointerdown={drag}
        oncontextmenu={contextMenu}
        onclick={(event) => {
          if (event.detail === 0) void setMenu(true);
        }}
      >
        <Mascot
          timers={snapshot?.timers ?? []}
          {now}
          animations={snapshot?.settings.animations ?? true}
          kind={snapshot?.settings.mascot ?? 'chicken'}
          phrases={snapshot?.settings.phrases ?? []}
          speechEnabled={snapshot?.settings.speech_enabled ?? false}
          speechMinMinutes={snapshot?.settings.speech_min_minutes ?? 5}
          speechMaxMinutes={snapshot?.settings.speech_max_minutes ?? 10}
          onSpeech={showPhrase}
        />
        {#if expired.length || error || connectionError || snapshot?.backend_error || snapshot?.desktop_error}
          <span class="mascot-alert" aria-live="polite"
            >{expired.length
              ? `${expired.length} expired ${expired.length === 1 ? 'timer' : 'timers'}`
              : 'Error: open the menu'}</span
          >
        {/if}
      </button>
    {/if}
  </main>
{:else if !authenticated}
  <main class="welcome">
    <img src="/mascot.png" width="160" height="160" alt="MonitorAwake mascot" />
    <p class="eyebrow">MONITORAWAKE</p>
    <h1>Start from the mascot.</h1>
    <p>
      Open <strong>Settings</strong> from the mascot’s gear button to connect this
      page to the app.
    </p>
    <p class="muted">The access link changes every time the app starts.</p>
  </main>
{:else}
  <div class="app-shell">
    <aside class="sidebar">
      <a href="/" class="brand"
        ><span class="brand-symbol"><Icon name="terminal" size={21} /></span
        ><span
          >MonitorAwake<span class="brand-subtitle"
            >A LITTLE HELP, ALWAYS HERE</span
          ></span
        ></a
      >
      <nav aria-label="Main navigation">
        <button
          class:chosen={section === 'timers'}
          onclick={() => (section = 'timers')}
          ><Icon name="timer" /> Timers and tasks
          <span class="nav-count">{running.length}</span></button
        >
        <button
          class:chosen={section === 'notes'}
          onclick={() => (section = 'notes')}
          ><Icon name="note" /> My notes
          <span class="nav-count">{snapshot?.notes.length ?? 0}</span></button
        >
        <button
          class:chosen={section === 'settings'}
          onclick={() => (section = 'settings')}
          ><Icon name="settings" /> Preferences</button
        >
      </nav>
      <div
        class="sidebar-companion"
        class:still={snapshot && !snapshot.settings.animations}
      >
        <img
          class="mascot"
          src="/mascot.png"
          width="150"
          height="150"
          alt="Your mascot"
        />
        <strong>You do your thing.</strong>
        <p>I'll keep track of time.</p>
      </div>
      <div class="local-label">
        <span class="status-dot" class:active={!!snapshot && !connectionError}
        ></span>{connectionError
          ? 'App unavailable'
          : 'Only on this computer'}<small>v0.1 · Everything stays here</small>
      </div>
    </aside>
    <main class="dashboard">
      <header class="page-header">
        <div>
          <p class="eyebrow">YOUR SPACE</p>
          <h1>
            {section === 'timers'
              ? 'One thing at a time.'
              : section === 'notes'
                ? 'A place for your thoughts.'
                : 'Just the way you like it.'}
          </h1>
          <p class="subtitle">
            {section === 'timers'
              ? 'A timer, a break, one less thing to remember.'
              : 'Small details for a quiet companion.'}
          </p>
        </div>
        <span class="session-badge"
          ><span
            class="status-dot"
            class:active={!!snapshot && !connectionError}
          ></span>
          {snapshot && !connectionError
            ? 'Connected to the mascot'
            : 'Connecting…'}</span
        >
      </header>
      {#if error || connectionError}<div class="notice error" role="alert">
          {error || connectionError}
        </div>{/if}
      {#if snapshot?.desktop_error}<div class="notice error" role="alert">
          {snapshot.desktop_error}
        </div>{/if}
      {#if snapshot?.backend_error}<div class="notice error" role="alert">
          {snapshot.backend_error}
        </div>{/if}
      {#if snapshot?.notification_error}<div class="notice error" role="alert">
          System notification could not be sent: {snapshot.notification_error}.
          Alerts remain visible here and on the mascot.
        </div>{/if}
      {#if section === 'timers'}
        <section class="assistant-card" aria-label="Quick commands">
          <div class="assistant-heading">
            <span class="soft-icon"><Icon name="terminal" /></span>
            <div>
              <h2>Tell me what you need</h2>
              <p aria-live="polite">{message}</p>
            </div>
            <span class="tiny-label">SIMPLE BY CHOICE</span>
          </div>
          <form class="command-form" onsubmit={sendCommand}>
            <input
              bind:value={command}
              aria-label="Assistant command"
              maxlength="300"
              placeholder="For example: remind me in 30 minutes"
              disabled={!snapshot}
            /><button
              class="primary square"
              aria-label="Send command"
              disabled={busy || !command.trim() || !snapshot}
              ><Icon name="arrow" /></button
            >
          </form>
          <div class="suggestions">
            <span>Try</span
            >{#each ['timer tea 5 minutes', 'remind me in 30 minutes', 'how much time is left?'] as suggestion}<button
                disabled={busy || !snapshot}
                onclick={() => act({ type: 'command', text: suggestion })}
                >{suggestion}</button
              >{/each}
          </div>
        </section>
        {#if expired.length}<section
            class="expired-list"
            aria-label="Expired timers"
          >
            {#each expired as timer (timer.id)}<article class="expired-card">
                <span class="soft-icon"><Icon name="check" /></span>
                <div>
                  <strong>Time’s up: {timer.name}</strong>
                  <p>Your timer has finished.</p>
                </div>
                <button
                  class="secondary"
                  disabled={busy}
                  onclick={() =>
                    act({ type: 'acknowledge_timer', id: timer.id })}
                  >Dismiss</button
                >
              </article>{/each}
          </section>{/if}
        <div class="two-columns">
          <section class="panel new-timer">
            <div class="section-title">
              <h2>A new timer</h2>
              <Icon name="plus" size={18} />
            </div>
            {@render notificationWizard()}
          </section>
          <section class="panel active-panel">
            <div class="section-title">
              <h2>Running <span class="count">{running.length}</span></h2>
              <span class="tiny-label">YOUR TIME REMAINING</span>
            </div>
            {#if !running.length}<div class="empty-state">
                <span class="empty-clock"><Icon name="timer" size={30} /></span>
                <h3>No rush.</h3>
                <p>
                  {snapshot
                    ? 'Start your first timer whenever you like.'
                    : 'Looking for your app…'}
                </p>
              </div>{:else}<div class="timer-list">
                {#each running as timer (timer.id)}<article class="timer-card">
                    <div class="row between">
                      <h3>{timer.name}</h3>
                      <button
                        class="icon-button small"
                        disabled={busy}
                        aria-label={`Cancel timer ${timer.name}`}
                        onclick={() =>
                          act({ type: 'cancel_timer', id: timer.id })}
                        ><Icon name="close" size={16} /></button
                      >
                    </div>
                    <div class="timer-time">{remaining(timer, now)}</div>
                    <progress
                      max={timer.duration_seconds * 1000}
                      value={Math.max(0, timer.deadline_ms - now)}
                      aria-label={`Time remaining for ${timer.name}`}
                    ></progress>
                    <div class="row between">
                      <small
                        >{timer.keep_awake
                          ? 'Computer stays awake during this timer'
                          : 'Does not keep the computer awake'}</small
                      ><button
                        class="text-button"
                        disabled={busy}
                        onclick={() =>
                          act({
                            type: 'extend_timer',
                            id: timer.id,
                            seconds: 300,
                          })}>+5 min</button
                      >
                    </div>
                  </article>{/each}
              </div>{/if}
          </section>
        </div>
        <section class="panel awake-panel">
          <div class="section-title">
            <div>
              <h2>Stay with me</h2>
              <p class="muted">Choose when to let your computer rest.</p>
            </div>
            <span class="state-label"
              ><span
                class="status-dot"
                class:active={snapshot?.applied_mode !== 'off' && !!snapshot}
              ></span>{statusText}</span
            >
          </div>
          <div class="mode-options">
            {#each modes as mode}<button
                class:selected={snapshot?.manual_mode === mode.value}
                disabled={busy ||
                  !snapshot ||
                  (mode.value === 'system' && !snapshot.capabilities.system) ||
                  (mode.value === 'display' && !snapshot.capabilities.display)}
                onclick={() => act({ type: 'set_awake', mode: mode.value })}
                ><span class="radio-dot"></span><span
                  ><strong>{mode.label}</strong><small>{mode.description}</small
                  ></span
                ></button
              >{/each}
          </div>
          <p class="footnote">
            Timers with “Keep awake” keep their own request active even in
            Automatic mode. This feature does not unlock your session.
          </p>
        </section>
        {#if history.length}<section class="history">
            <div class="section-title">
              <h2>History <span class="count">{history.length}</span></h2>
              <button
                class="text-button"
                disabled={busy}
                onclick={() => act({ type: 'clear_history' })}
                >Clear history</button
              >
            </div>
            {#each history
              .slice()
              .reverse()
              .slice(0, 8) as timer (timer.id)}<div class="history-row">
                <Icon
                  name={timer.status === 'cancelled' ? 'close' : 'check'}
                  size={16}
                /><span>{timer.name}</span><small
                  >{timer.status === 'cancelled'
                    ? 'Cancelled'
                    : 'Completed'}</small
                >
              </div>{/each}
          </section>{/if}
      {:else if section === 'notes'}
        <section class="panel">
          {#if snapshot}<NotesPanel {snapshot} onstate={accept} />{/if}
        </section>
      {:else}
        <section class="panel preferences">
          <h2>Your mascot</h2>
          <div class="mascot-choice">
            <Mascot
              timers={snapshot?.timers ?? []}
              {now}
              animations={snapshot?.settings.animations ?? true}
              kind={snapshot?.settings.mascot ?? 'chicken'}
            />
            <div>
              <label class="mascot-picker" for="settings-mascot"
                ><span>Choose a mascot</span>
                <select
                  id="settings-mascot"
                  value={snapshot?.settings.mascot ?? 'chicken'}
                  disabled={busy || !snapshot}
                  onchange={(event) =>
                    selectMascot(event.currentTarget.value as MascotKind)}
                >
                  {#each mascots as mascot}<option value={mascot.id}
                      >{mascot.name}</option
                    >{/each}
                </select>
              </label>
              <p>
                {mascots.find(
                  (mascot) =>
                    mascot.id === (snapshot?.settings.mascot ?? 'chicken'),
                )?.description}
              </p>
              <small>Choose a companion for your desktop.</small>
            </div>
          </div>
          <dl class="mascot-states">
            <div>
              <dt>Waiting</dt>
              <dd>Looks up and keeps you company.</dd>
            </div>
            <div>
              <dt>Random action</dt>
              <dd>Occasionally pecks or tries to fly.</dd>
            </div>
            <div>
              <dt>Waiting for notification</dt>
              <dd>Broods over one egg per pending notification.</dd>
            </div>
          </dl>
          <p class="egg-legend">
            Green: more than half the time remains. Yellow: halfway there. Red:
            the final 20%. Cracked: ready to dismiss. Each egg has a number and
            a tooltip with its notification and remaining time.
          </p>
          <label class="setting-row"
            ><span
              ><strong>Gentle animations</strong><small
                >A little movement while keeping you company.</small
              ></span
            ><input
              type="checkbox"
              checked={snapshot?.settings.animations ?? true}
              disabled={busy || !snapshot}
              onchange={(event) =>
                snapshot &&
                act({
                  type: 'update_settings',
                  settings: {
                    ...snapshot.settings,
                    animations: event.currentTarget.checked,
                  },
                })}
            /></label
          ><label class="setting-row"
            ><span
              ><strong>System notifications</strong><small
                >Notify when a timer ends, even when this page is closed.</small
              ></span
            ><input
              type="checkbox"
              checked={snapshot?.settings.notifications ?? true}
              disabled={busy || !snapshot}
              onchange={(event) =>
                snapshot &&
                act({
                  type: 'update_settings',
                  settings: {
                    ...snapshot.settings,
                    notifications: event.currentTarget.checked,
                  },
                })}
            /></label
          >
          <p class="footnote">
            Your system’s “Do not disturb” mode may hide notifications. Alerts
            remain visible on the mascot.
          </p>
        </section>
        {#if snapshot}<PhraseSettings {snapshot} onstate={accept} />{/if}
        <section class="panel preferences">
          <h2>Truly local.</h2>
          <p>
            This page only connects to the app on your computer. Timers and
            preferences are saved in your user profile.
          </p>
          <dl>
            <div>
              <dt>Application version</dt>
              <dd>{snapshot?.app_version ?? '—'}</dd>
            </div>
            <div>
              <dt>Data schema</dt>
              <dd>{snapshot?.schema_version ?? '—'}</dd>
            </div>
            <div>
              <dt>Integration</dt>
              <dd>{snapshot?.capabilities.backend ?? 'Waiting'}</dd>
            </div>
            <div>
              <dt>Connection</dt>
              <dd>{location.origin}</dd>
            </div>
            <div>
              <dt>When you close this page</dt>
              <dd>The assistant keeps working.</dd>
            </div>
          </dl>
          <p class="footnote">
            If the computer sleeps or the app is closed, expired alerts are
            restored when you return. Manual keep-awake turns off when you quit
            the app.
          </p>
        </section>
      {/if}
      <footer>
        <Icon name="terminal" size={14} /><span
          >A small assistant. A space of your own.</span
        ><span>MonitorAwake · first release</span>
      </footer>
    </main>
  </div>
{/if}
