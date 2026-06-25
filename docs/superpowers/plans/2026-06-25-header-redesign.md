# Header Redesign Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Remove the app name, increase the clock font size to 1.1rem, and flatten tab styling so the active tab matches the editor's edit-bar background.

**Architecture:** Pure CSS/template changes in two Svelte component files. No logic changes, no new tokens, no new files.

**Tech Stack:** Svelte 5, scoped `<style>` blocks, CSS custom properties (`--edit-bar-bg`, `--fg`, `--muted`).

---

### Task 1: Remove app name from Header

**Files:**
- Modify: `web/src/lib/components/Header.svelte`

- [ ] **Step 1: Remove the `.name` div from the template**

In `web/src/lib/components/Header.svelte`, delete line 16:

```diff
 <header class="header">
-  <div class="name">Slugline</div>
   <Tabs />
```

- [ ] **Step 2: Remove the `.name` CSS rule**

In the `<style>` block, delete the `.name` rule (currently line 30):

```diff
-  .name { font-weight: 700; font-size: 1.1rem; color: var(--heading-1); letter-spacing: 0.02em; }
```

- [ ] **Step 3: Verify in browser**

Run the dev server (`npm run dev` in `web/`) and confirm the header shows only the tabs and clock — no "Slugline" text.

- [ ] **Step 4: Commit**

```bash
git add web/src/lib/components/Header.svelte
git commit -m "style: remove app name from header"
```

---

### Task 2: Increase clock font size

**Files:**
- Modify: `web/src/lib/components/Header.svelte`

- [ ] **Step 1: Update clock font-size**

In `web/src/lib/components/Header.svelte`, in the `.clock` CSS rule, change `font-size` from `0.78rem` to `1.1rem`:

```diff
   .clock {
     margin-left: auto; display: flex; flex-direction: column;
-    text-align: right; font-size: 0.78rem; color: var(--muted); line-height: 1.15;
+    text-align: right; font-size: 1.1rem; color: var(--muted); line-height: 1.15;
   }
```

- [ ] **Step 2: Verify in browser**

Confirm the stacked date and time in the top-right of the header are noticeably larger but still fit within the header height alongside the tabs.

- [ ] **Step 3: Commit**

```bash
git add web/src/lib/components/Header.svelte
git commit -m "style: increase header clock font size to 1.1rem"
```

---

### Task 3: Flatten tabs and apply edit-bar active style

**Files:**
- Modify: `web/src/lib/components/Tabs.svelte`

- [ ] **Step 1: Remove border-radius from `.tab`**

In `web/src/lib/components/Tabs.svelte`, remove `border-radius: 6px 6px 0 0` from the `.tab` rule:

```diff
   .tab {
     display: inline-flex; align-items: center; gap: 0.4rem;
     border: none; cursor: pointer; padding: 0.3rem 0.6rem;
-    border-radius: 6px 6px 0 0; background: transparent; color: var(--muted);
+    background: transparent; color: var(--muted);
     font: inherit; font-size: 0.85rem; white-space: nowrap;
   }
```

- [ ] **Step 2: Replace `.tab.active` styles**

Replace the `.tab.active` rule — remove the white background and blue box-shadow underline, apply the edit-bar background instead:

```diff
-  .tab.active { background: var(--bg); color: var(--fg); box-shadow: inset 0 -2px 0 var(--accent); }
+  .tab.active { background: var(--edit-bar-bg); color: var(--fg); }
```

- [ ] **Step 3: Verify in browser**

Confirm:
- All tabs have straight (rectangular) corners
- Inactive tabs remain transparent with muted text
- The active tab shows the light blue edit-bar background (`#e2ebff`) with no underline shadow
- Switching between tabs correctly moves the active highlight

- [ ] **Step 4: Commit**

```bash
git add web/src/lib/components/Tabs.svelte
git commit -m "style: flat tabs with edit-bar active background"
```
