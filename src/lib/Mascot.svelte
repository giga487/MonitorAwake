<script lang="ts">
  import { onMount } from 'svelte';
  import type { Timer } from './api';
  import { remaining } from './api';
  import {
    mascots,
    mascotState,
    notificationEggs,
    idleActionPlan,
    type MascotKind,
    type IdleAction,
  } from './mascot';

  let {
    timers = [],
    now,
    animations = true,
    kind = 'chicken',
  }: {
    timers?: Timer[];
    now: number;
    animations?: boolean;
    kind?: MascotKind;
  } = $props();
  let action = $state<IdleAction | null>(null);
  let mounted = $state(false);
  let reducedMotion = $state(false);
  let visible = $state(true);
  const character = $derived(
    mascots.find((mascot) => mascot.id === kind) ?? mascots[0],
  );
  const eggs = $derived(notificationEggs(timers, now));
  const hasNotifications = $derived(eggs.length > 0);
  const animate = $derived(animations && !reducedMotion);
  const canPlay = $derived(mounted && visible && animate && !hasNotifications);
  const activity = $derived(mascotState(timers, now, animate ? action : null));
  const pose = $derived(
    activity === 'waiting-for-notification'
      ? 'brood'
      : activity === 'random-action'
        ? action
        : 'waiting',
  );
  const stateLabel = $derived(
    activity === 'notification-ready'
      ? 'Notification ready'
      : activity === 'waiting-for-notification'
        ? 'Waiting for notification'
        : activity === 'random-action'
          ? 'Random action'
          : 'Waiting',
  );

  onMount(() => {
    const media = window.matchMedia('(prefers-reduced-motion: reduce)');
    const motion = () => (reducedMotion = media.matches);
    const visibility = () => (visible = document.visibilityState !== 'hidden');
    motion();
    visibility();
    mounted = true;
    media.addEventListener('change', motion);
    document.addEventListener('visibilitychange', visibility);
    return () => {
      media.removeEventListener('change', motion);
      document.removeEventListener('visibilitychange', visibility);
    };
  });

  $effect(() => {
    if (!canPlay) {
      action = null;
      return;
    }
    let timer: ReturnType<typeof setTimeout>;
    const schedule = () => {
      const plan = idleActionPlan();
      timer = setTimeout(() => {
        action = plan.action;
        timer = setTimeout(() => {
          action = null;
          schedule();
        }, plan.duration);
      }, plan.delay);
    };
    schedule();
    return () => clearTimeout(timer);
  });
</script>

<span class="mascot-stage" class:motionless={!animate} data-state={activity}>
  <span
    class="chicken-sprite"
    class:ready={activity === 'notification-ready'}
    data-pose={pose}
    style:background-image={`url('${character.atlas}')`}
    role="img"
    aria-label={`${character.name}: ${stateLabel}`}
  >
  </span>
  {#if eggs.length}
    <span
      class="egg-nest"
      role="list"
      aria-label={`${eggs.length} pending notifications`}
      onpointerdown={(event) => event.stopPropagation()}
    >
      {#each eggs as egg, index (egg.timer.id)}
        <span
          class="notification-egg"
          data-phase={egg.phase}
          class:due={egg.due}
          role="listitem"
          title={`${egg.timer.name} · ${egg.label} · ${remaining(egg.timer, now)}`}
          aria-label={`${egg.timer.name}: ${egg.label}, ${remaining(egg.timer, now)} remaining`}
        >
          <svg
            viewBox="0 0 18 24"
            aria-hidden="true"
            shape-rendering="crispEdges"
          >
            <path
              class="egg-shell"
              d="M7 1h4v2h2v3h2v4h1v8h-1v3h-3v2H6v-2H3v-3H2v-8h1V6h2V3h2Z"
            />
            <path class="egg-glint" d="M6 6h2v3H6v4H4v-3h1V8h1Z" />
            {#if egg.due}<path
                class="egg-mark"
                d="m6 3 4 5-3 4 5 4-2 5"
                fill="none"
                stroke-width="2"
              />
            {:else}<path
                class="egg-mark"
                d={egg.phase === 'green'
                  ? 'M7 17h4v2H7Z'
                  : egg.phase === 'yellow'
                    ? 'M6 14h6v2H6Zm0 4h6v2H6Z'
                    : 'M6 11h6v2H6Zm0 4h6v2H6Zm0 4h6v2H6Z'}
              />{/if}
          </svg>
          <span class="egg-index">{index + 1}</span>
        </span>
      {/each}
    </span>
  {/if}
  <span class="mascot-sr" aria-live="polite"
    >{stateLabel}. {eggs.length
      ? `${eggs.length} pending notifications.`
      : 'No pending notifications.'}</span
  >
</span>

<style>
  .mascot-stage {
    position: relative;
    display: block;
    width: 150px;
    height: 146px;
    pointer-events: none;
  }
  .chicken-sprite {
    position: absolute;
    inset: 0;
    display: block;
    background-size: 200% 200%;
    background-repeat: no-repeat;
    background-position: 0 0;
    image-rendering: pixelated;
    transform-origin: 50% 90%;
    animation: look-around 8s steps(1, end) infinite;
  }
  .chicken-sprite[data-pose='brood'] {
    background-position: 100% 100%;
    animation: brooding 5s ease-in-out infinite;
  }
  .chicken-sprite[data-pose='peck'] {
    background-position: 100% 0;
    animation: pecking 1.8s steps(1, end) 1;
  }
  .chicken-sprite[data-pose='flutter'] {
    background-position: 0 100%;
    animation: fluttering 1.4s steps(1, end) 1;
  }
  .chicken-sprite.ready {
    animation: ready 1.5s ease-in-out infinite;
  }
  .egg-nest {
    position: absolute;
    z-index: 1;
    bottom: 0;
    left: 2px;
    right: 2px;
    display: flex;
    justify-content: center;
    flex-wrap: wrap;
    align-content: flex-start;
    gap: 1px 3px;
    max-height: 48px;
    overflow-y: auto;
    padding: 1px 3px 3px;
    border-bottom: 3px solid #a97942;
    border-radius: 0 0 35% 35%;
    background: linear-gradient(transparent 50%, #af7e4333);
    pointer-events: auto;
    scrollbar-width: thin;
    scrollbar-color: #a97942 transparent;
  }
  .notification-egg {
    position: relative;
    display: block;
    width: 17px;
    height: 24px;
    flex: 0 0 17px;
    --shell: #a8d489;
    --outline: #46683b;
  }
  .notification-egg[data-phase='yellow'] {
    --shell: #f4d36b;
    --outline: #80652e;
  }
  .notification-egg[data-phase='red'] {
    --shell: #ed8e7b;
    --outline: #8b4038;
  }
  .notification-egg svg {
    width: 100%;
    height: 100%;
  }
  .egg-shell {
    fill: var(--shell);
    stroke: var(--outline);
    stroke-width: 1;
  }
  .egg-glint {
    fill: #fff8db;
    opacity: 0.8;
  }
  .egg-mark {
    fill: var(--outline);
    stroke: var(--outline);
    stroke-width: 0;
  }
  .due .egg-mark {
    stroke-width: 2;
  }
  .egg-index {
    position: absolute;
    top: 5px;
    width: 100%;
    text-align: center;
    font: 600 8px/1 monospace;
    color: #3a302b;
  }
  .notification-egg.due {
    animation: egg-ready 1.5s steps(2, end) infinite;
  }
  .mascot-sr {
    position: absolute;
    width: 1px;
    height: 1px;
    padding: 0;
    overflow: hidden;
    clip-path: inset(50%);
    white-space: nowrap;
  }
  .motionless *,
  .motionless .chicken-sprite {
    animation: none;
  }
  @keyframes look-around {
    0%,
    65%,
    100% {
      transform: translateY(0) rotate(0);
    }
    70%,
    80% {
      transform: translateY(-2px) rotate(-3deg);
    }
    85%,
    95% {
      transform: translateY(-2px) rotate(3deg);
    }
  }
  @keyframes brooding {
    0%,
    100% {
      transform: translateY(-6px) scaleY(1);
    }
    50% {
      transform: translateY(-5px) scaleY(0.98);
    }
  }
  @keyframes pecking {
    0%,
    100% {
      background-position: 0 0;
      transform: translateY(0);
    }
    15%,
    45%,
    75% {
      background-position: 100% 0;
      transform: translateY(2px);
    }
    30%,
    60%,
    90% {
      background-position: 0 0;
      transform: translateY(0);
    }
  }
  @keyframes fluttering {
    0%,
    100% {
      background-position: 0 0;
      transform: translateY(0);
    }
    15%,
    45%,
    75% {
      background-position: 0 100%;
      transform: translateY(-5px) rotate(-2deg);
    }
    30%,
    60%,
    90% {
      background-position: 0 0;
      transform: translateY(-2px) rotate(2deg);
    }
  }
  @keyframes ready {
    0%,
    100% {
      transform: rotate(-3deg);
    }
    50% {
      transform: translateY(-3px) rotate(3deg);
    }
  }
  @keyframes egg-ready {
    0%,
    100% {
      transform: rotate(-5deg);
    }
    50% {
      transform: rotate(5deg);
    }
  }
  @media (prefers-reduced-motion: reduce) {
    .mascot-stage * {
      animation: none;
    }
  }
</style>
