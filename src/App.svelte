<script lang="ts">
  import { onMount } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';
  import { getCurrentWindow } from '@tauri-apps/api/window';
  import Icon from './lib/Icon.svelte';
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
  let message = $state('Sono qui. Dimmi quando avvisarti.');
  let error = $state('');
  let connectionError = $state('');
  let authenticated = $state(true);
  let busy = $state(false);
  let command = $state('');
  let name = $state('Pausa');
  let minutes = $state(30);
  let keepAwake = $state(false);
  let now = $state(Date.now());
  let clockOffset = 0;
  let configUrl = $state('');
  let section = $state<'timers' | 'settings'>('timers');
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
      ? 'Connessione…'
      : snapshot.backend_error
        ? 'Richiesta non riuscita'
        : snapshot.applied_mode !== snapshot.requested_mode
          ? 'Aggiornamento…'
          : snapshot.applied_mode === 'display'
            ? 'Sistema e schermo svegli'
            : snapshot.applied_mode === 'system'
              ? 'Sistema sveglio'
              : 'Risparmio energetico normale',
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
    authenticated = initializeSession();
    let stopped = false;
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
            'Non riesco a raggiungere l’app. Se l’hai riavviata, riapri la configurazione dalla mascotte.';
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
    await act({
      type: 'add_timer',
      name,
      seconds: Math.round(minutes * 60),
      keep_awake: keepAwake,
    });
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
    try {
      await getCurrentWindow().startDragging();
    } catch (e) {
      error = String(e);
    }
  }
</script>

{#if desktop}
  <main class="widget" class:still={snapshot && !snapshot.settings.animations}>
    <div class="widget-top">
      <span>MONITORAWAKE</span>
      <div class="row">
        <button
          class="icon-button small"
          aria-label="Minimizza nella barra delle applicazioni"
          onclick={() =>
            getCurrentWindow()
              .minimize()
              .catch((e) => (error = String(e)))}
          ><Icon name="minus" size={15} /></button
        >
        <button
          class="icon-button small"
          aria-label="Esci e interrompi il mantenimento in attività"
          onclick={() => desktopAction('quit')}
          ><Icon name="close" size={15} /></button
        >
      </div>
    </div>
    <div class="speech" aria-live="polite">
      {#if expired.length}
        <strong>È il momento! ✨</strong>
        <span>«{expired[0].name}» è terminato.</span>
        <button
          class="text-button"
          disabled={busy}
          onclick={() => act({ type: 'acknowledge_timer', id: expired[0].id })}
          >Ho visto <Icon name="check" size={14} /></button
        >
      {:else if running[0]}
        <span
          >{running[0].name}{running.length > 1
            ? ` · +${running.length - 1} timer`
            : ''}</span
        >
        <strong class="widget-countdown">{remaining(running[0], now)}</strong>
      {:else}<span>{message}</span>{/if}
    </div>
    <button
      class="mascot-drag"
      aria-label="Trascina la mascotte per spostarla"
      onpointerdown={drag}
    >
      <img
        class="mascot"
        class:celebrate={expired.length > 0}
        src="/mascot.png"
        alt="Una gallina robot in pixel art con cresta rossa e viso verde fosforo su uno schermo"
        draggable="false"
      />
    </button>
    <div class="widget-controls">
      <form class="command-form compact" onsubmit={sendCommand}>
        <input
          aria-label="Comando per l’assistente"
          bind:value={command}
          maxlength="300"
          placeholder="Timer tè 5 minuti…"
          disabled={!snapshot}
        />
        <button
          class="icon-button"
          disabled={busy || !command.trim() || !snapshot}
          aria-label="Invia comando"><Icon name="arrow" size={17} /></button
        >
      </form>
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
            ? 'Mantieni sveglio il sistema'
            : 'Disattiva il mantenimento manuale'}
        >
          <span
            class="status-dot"
            class:active={snapshot?.applied_mode !== 'off' && !!snapshot}
          ></span>
          {snapshot?.applied_mode === 'off' ? 'In pausa' : 'Sveglio'}
        </button>
        <button
          class="icon-button"
          aria-label="Apri la configurazione nel browser"
          onclick={() => desktopAction('open_configuration')}
          ><Icon name="settings" size={17} /></button
        >
        <button class="text-button url-button" onclick={showUrl}>URL</button>
      </div>
    </div>
    {#if error || connectionError || snapshot?.backend_error}<p
        class="widget-error"
        role="alert"
      >
        {error || connectionError || snapshot?.backend_error}
      </p>{/if}
    {#if configUrl}<input
        class="url-field"
        aria-label="URL privato della configurazione, seleziona e copia"
        readonly
        value={configUrl}
        onclick={(event) => event.currentTarget.select()}
      />{/if}
  </main>
{:else if !authenticated}
  <main class="welcome">
    <img
      src="/mascot.png"
      width="160"
      height="160"
      alt="Mascotte MonitorAwake"
    />
    <p class="eyebrow">MONITORAWAKE</p>
    <h1>Ci vediamo dalla mascotte.</h1>
    <p>
      Apri <strong>Configurazione</strong> dal suo ingranaggio per collegare questa
      pagina all’app.
    </p>
    <p class="muted">Il link di accesso cambia a ogni avvio.</p>
  </main>
{:else}
  <div class="app-shell">
    <aside class="sidebar">
      <a href="/" class="brand"
        ><span class="brand-symbol"><Icon name="terminal" size={21} /></span
        ><span
          >MonitorAwake<span class="brand-subtitle"
            >UN PICCOLO AIUTO, SEMPRE QUI</span
          ></span
        ></a
      >
      <nav aria-label="Navigazione principale">
        <button
          class:chosen={section === 'timers'}
          onclick={() => (section = 'timers')}
          ><Icon name="timer" /> Timer e attività
          <span class="nav-count">{running.length}</span></button
        >
        <button
          class:chosen={section === 'settings'}
          onclick={() => (section = 'settings')}
          ><Icon name="settings" /> Preferenze</button
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
          alt="La tua mascotte"
        />
        <strong>Tu fai le tue cose.</strong>
        <p>Al tempo ci penso io.</p>
      </div>
      <div class="local-label">
        <span class="status-dot" class:active={!!snapshot && !connectionError}
        ></span>{connectionError
          ? 'App non raggiungibile'
          : 'Solo su questo computer'}<small>v0.1 · Tutto rimane qui</small>
      </div>
    </aside>
    <main class="dashboard">
      <header class="page-header">
        <div>
          <p class="eyebrow">IL TUO SPAZIO</p>
          <h1>
            {section === 'timers' ? 'Una cosa alla volta.' : 'Come piace a te.'}
          </h1>
          <p class="subtitle">
            {section === 'timers'
              ? 'Un timer, una pausa, un pensiero in meno.'
              : 'Piccoli dettagli per un assistente discreto.'}
          </p>
        </div>
        <span class="session-badge"
          ><span
            class="status-dot"
            class:active={!!snapshot && !connectionError}
          ></span>
          {snapshot && !connectionError
            ? 'Collegato alla mascotte'
            : 'Connessione…'}</span
        >
      </header>
      {#if error || connectionError}<div class="notice error" role="alert">
          {error || connectionError}
        </div>{/if}
      {#if snapshot?.backend_error}<div class="notice error" role="alert">
          {snapshot.backend_error}
        </div>{/if}
      {#if snapshot?.notification_error}<div class="notice error" role="alert">
          Notifica di sistema non inviata: {snapshot.notification_error}. Gli
          avvisi restano visibili qui e sulla mascotte.
        </div>{/if}
      {#if section === 'timers'}
        <section class="assistant-card" aria-label="Comandi rapidi">
          <div class="assistant-heading">
            <span class="soft-icon"><Icon name="terminal" /></span>
            <div>
              <h2>Dimmi cosa ti serve</h2>
              <p aria-live="polite">{message}</p>
            </div>
            <span class="tiny-label">SEMPLICE, PER SCELTA</span>
          </div>
          <form class="command-form" onsubmit={sendCommand}>
            <input
              bind:value={command}
              aria-label="Comando per l’assistente"
              maxlength="300"
              placeholder="Ad esempio: avvisami tra 30 minuti"
              disabled={!snapshot}
            /><button
              class="primary square"
              aria-label="Invia comando"
              disabled={busy || !command.trim() || !snapshot}
              ><Icon name="arrow" /></button
            >
          </form>
          <div class="suggestions">
            <span>Prova</span
            >{#each ['timer tè 5 minuti', 'avvisami tra 30 minuti', 'quanto manca?'] as suggestion}<button
                disabled={busy || !snapshot}
                onclick={() => act({ type: 'command', text: suggestion })}
                >{suggestion}</button
              >{/each}
          </div>
        </section>
        {#if expired.length}<section
            class="expired-list"
            aria-label="Timer scaduti"
          >
            {#each expired as timer (timer.id)}<article class="expired-card">
                <span class="soft-icon"><Icon name="check" /></span>
                <div>
                  <strong>È il momento: {timer.name}</strong>
                  <p>Il tuo timer è terminato.</p>
                </div>
                <button
                  class="secondary"
                  disabled={busy}
                  onclick={() =>
                    act({ type: 'acknowledge_timer', id: timer.id })}
                  >Ho visto</button
                >
              </article>{/each}
          </section>{/if}
        <div class="two-columns">
          <section class="panel new-timer">
            <div class="section-title">
              <h2>Un nuovo timer</h2>
              <Icon name="plus" size={18} />
            </div>
            <form onsubmit={addTimer}>
              <label for="timer-name">Per cosa?</label><input
                id="timer-name"
                bind:value={name}
                maxlength="60"
                required
                placeholder="Una piccola pausa"
              />
              <div class="duration-heading">
                <label for="timer-minutes">Tra quanto?</label><span>minuti</span
                >
              </div>
              <input
                class="duration-input"
                id="timer-minutes"
                type="number"
                bind:value={minutes}
                min="1"
                max="10080"
                step="1"
                required
              />
              <div class="presets">
                {#each [5, 15, 30, 60] as preset}<button
                    type="button"
                    class:selected={minutes === preset}
                    onclick={() => (minutes = preset)}>{preset} min</button
                  >{/each}
              </div>
              <label class="checkbox-row"
                ><input
                  type="checkbox"
                  bind:checked={keepAwake}
                  disabled={!snapshot?.capabilities.system}
                /><span>Mantieni sveglio il PC fino alla scadenza</span></label
              ><button class="primary full" disabled={busy || !snapshot}
                ><Icon name="plus" size={18} /> Avvia timer</button
              >
            </form>
          </section>
          <section class="panel active-panel">
            <div class="section-title">
              <h2>In corso <span class="count">{running.length}</span></h2>
              <span class="tiny-label">IL TEMPO CHE TI RESTA</span>
            </div>
            {#if !running.length}<div class="empty-state">
                <span class="empty-clock"><Icon name="timer" size={30} /></span>
                <h3>Nessuna fretta.</h3>
                <p>
                  {snapshot
                    ? 'Quando vuoi, avvia il tuo primo timer.'
                    : 'Sto cercando la tua app…'}
                </p>
              </div>{:else}<div class="timer-list">
                {#each running as timer (timer.id)}<article class="timer-card">
                    <div class="row between">
                      <h3>{timer.name}</h3>
                      <button
                        class="icon-button small"
                        disabled={busy}
                        aria-label={`Annulla timer ${timer.name}`}
                        onclick={() =>
                          act({ type: 'cancel_timer', id: timer.id })}
                        ><Icon name="close" size={16} /></button
                      >
                    </div>
                    <div class="timer-time">{remaining(timer, now)}</div>
                    <progress
                      max={timer.duration_seconds * 1000}
                      value={Math.max(0, timer.deadline_ms - now)}
                      aria-label={`Tempo rimanente per ${timer.name}`}
                    ></progress>
                    <div class="row between">
                      <small
                        >{timer.keep_awake
                          ? 'PC sveglio durante il timer'
                          : 'Timer indipendente dal risveglio'}</small
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
              <h2>Rimani con me</h2>
              <p class="muted">Scegli quando lasciare riposare il computer.</p>
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
            I timer con “PC sveglio” mantengono la propria richiesta anche in
            modalità Automatica. Questa funzione non sblocca la sessione.
          </p>
        </section>
        {#if history.length}<section class="history">
            <div class="section-title">
              <h2>Già fatto <span class="count">{history.length}</span></h2>
              <button
                class="text-button"
                disabled={busy}
                onclick={() => act({ type: 'clear_history' })}
                >Svuota storico</button
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
                    ? 'Annullato'
                    : 'Completato'}</small
                >
              </div>{/each}
          </section>{/if}
      {:else}
        <section class="panel preferences">
          <h2>La tua mascotte</h2>
          <label class="setting-row"
            ><span
              ><strong>Animazioni leggere</strong><small
                >Un piccolo movimento mentre ti fa compagnia.</small
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
              ><strong>Notifiche di sistema</strong><small
                >Avvisa alla scadenza, anche con la pagina chiusa.</small
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
            La modalità “Non disturbare” del sistema può nascondere le
            notifiche. Gli avvisi restano visibili nella mascotte.
          </p>
        </section>
        <section class="panel preferences">
          <h2>Locale, davvero.</h2>
          <p>
            Questa pagina comunica solo con l’app sul tuo computer. Timer e
            preferenze vengono salvati nel tuo profilo utente.
          </p>
          <dl>
            <div>
              <dt>Integrazione</dt>
              <dd>{snapshot?.capabilities.backend ?? 'In attesa'}</dd>
            </div>
            <div>
              <dt>Connessione</dt>
              <dd>{location.origin}</dd>
            </div>
            <div>
              <dt>Quando chiudi questa pagina</dt>
              <dd>L’assistente continua a lavorare.</dd>
            </div>
          </dl>
          <p class="footnote">
            Se il computer dorme o l’app è chiusa, gli avvisi scaduti vengono
            recuperati al ritorno. Il mantenimento manuale si disattiva quando
            esci dall’app.
          </p>
        </section>
      {/if}
      <footer>
        <Icon name="terminal" size={14} /><span
          >Un assistente piccolo. Uno spazio tutto tuo.</span
        ><span>MonitorAwake · prima versione</span>
      </footer>
    </main>
  </div>
{/if}
