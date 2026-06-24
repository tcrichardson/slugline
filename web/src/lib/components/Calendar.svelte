<script lang="ts">
  import { app } from '../appState.svelte';
  import { monthGrid, todayISO } from '../dates';

  const weeks = $derived(monthGrid(app.calendar.year, app.calendar.month));
  const today = todayISO();
  const fileSet = $derived(new Set(app.notesWithFiles));
  const monthLabel = $derived(
    new Date(Date.UTC(app.calendar.year, app.calendar.month - 1, 1)).toLocaleDateString(undefined, {
      month: 'long',
      year: 'numeric',
      timeZone: 'UTC',
    }),
  );

  function onCellClick(e: MouseEvent, date: string) {
    if (e.metaKey || e.ctrlKey) app.openInNewTab(date);
    else app.goToDate(date);
  }
</script>

<div class="calendar">
  <div class="cal-head">
    <button onclick={() => app.prevMonth()} aria-label="Previous month">‹</button>
    <span class="month">{monthLabel}</span>
    <button onclick={() => app.nextMonth()} aria-label="Next month">›</button>
  </div>
  <div class="cal-grid">
    {#each ['S', 'M', 'T', 'W', 'T', 'F', 'S'] as d, i (i)}
      <div class="dow">{d}</div>
    {/each}
    {#each weeks as week, wi (wi)}
      {#each week as cell (cell.date)}
        <button
          class="cell"
          class:out={!cell.inMonth}
          class:today={cell.date === today}
          class:selected={cell.date === app.activeDate}
          onclick={(e) => onCellClick(e, cell.date)}
        >
          <span class="num">{Number(cell.date.slice(8, 10))}</span>
          {#if fileSet.has(cell.date)}<span class="dot"></span>{/if}
        </button>
      {/each}
    {/each}
  </div>
</div>

<style>
  .calendar { padding: 0.75rem; }
  .cal-head { display: flex; align-items: center; justify-content: space-between; margin-bottom: 0.5rem; }
  .cal-head button { border: none; background: transparent; color: var(--fg); cursor: pointer; font-size: 1rem; }
  .month { font-size: 0.85rem; font-weight: 600; }
  .cal-grid { display: grid; grid-template-columns: repeat(7, 1fr); gap: 2px; }
  .dow { text-align: center; font-size: 0.7rem; color: var(--muted); padding-bottom: 0.25rem; }
  .cell {
    position: relative; aspect-ratio: 1; border: none; cursor: pointer;
    background: transparent; color: var(--fg); border-radius: 6px; font: inherit; font-size: 0.8rem;
  }
  .cell:hover { background: var(--edit-line-bg); }
  .cell.out { color: var(--muted); opacity: 0.5; }
  .cell.today { outline: 1px solid var(--accent); }
  .cell.selected { background: var(--accent); color: #fff; }
  .dot {
    position: absolute; bottom: 4px; left: 50%; transform: translateX(-50%);
    width: 4px; height: 4px; border-radius: 50%; background: var(--accent);
  }
  .cell.selected .dot { background: #fff; }
</style>
