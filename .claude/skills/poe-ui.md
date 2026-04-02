---
description: Generate PoE-themed UI components for Path of AI. Use when creating or modifying HTML/CSS for the app UI, adding new tabs, cards, panels, or interactive elements. TRIGGER when: user asks to create UI components, improve UI, add tabs, or make things look more like PoE.
---

# PoE UI Component Generator

You are generating UI for **Path of AI**, a Path of Exile build advisor tool.

## CRITICAL: Read These Files First

Before generating ANY UI, read these reference files:
1. `docs/DESIGN-DECISIONS.md` — naming conventions, color palette, fonts, vocabulary
2. `ui/pob-advisor.html` — current UI implementation to match existing patterns

## DESIGN RULES

### Theme: Warm Dark (matching PoE's actual UI — brown-black, NOT purple-black)
- Background: `#0c0a08` (warm dark brown-black)
- Cards: `#1c1914` (warm brown-black)
- Borders: `#3a3028` (warm brown), `#5a1a1a` (blood)
- Active accent: `#d03030` (blood bright)
- Gold: `#9a7a2a` / `#dab040`
- Text: `#e8e0d0` (high contrast parchment), `#f8f2e8` (bright headings)
- Never use purple-tinted backgrounds. Never use Tailwind colors. Always PoE's warm earthy palette.

### Fonts (MUST use these)
- **Cinzel** — headers, tab names, section titles, labels
- **Crimson Text** — body text, descriptions
- **JetBrains Mono** — numbers, stats, DPS values, prices, scores

### Naming (MUST use PoE vocabulary)
- NEVER say "Submit" → say **"Invoke"**
- NEVER say "Overview" → say **"Omen"**
- NEVER say "Items" → say **"Arsenal"**
- NEVER say "Suggestions" → say **"Prophecy"**
- NEVER say "Issues" → say **"Harbinger Warnings"**
- NEVER say "Checklist" → say **"Blood Pact"**
- NEVER say "Quick Actions" → say **"Blood Rituals"**
- NEVER say "Fix:" → say **"Remedy:"**
- NEVER say "Apply" → say **"Invoke"**
- NEVER say "AI Assistant" → say **"The Seer"**
- Address user as **"Exile"**

### Colors (PoE-exact)
```
Fire: #cf3c18    Cold: #3674c2    Lightning: #d4aa00    Chaos: #9955dd
Life: #c41e1e    ES: #5090d0     Success: #3ec41e      Danger: #ef4444
Unique: #af6025  Rare: #ffff77   Magic: #8888ff        Normal: #c8c8c8
T1: #e8d44d (gold)  T2: #3ec41e (green)  T3: #3674c2 (blue)  T4: #7a6b55 (gray)  T5: #c41e1e (red)
```

### Section Label Pattern
```html
<div class="section-label">&#x2620; Section Name &mdash; Subtitle</div>
```
Style: 9px Cinzel, uppercase, letter-spacing 2px, with decorative line extending right.

### Card Pattern
```html
<div style="background:var(--bg-card);border:1px solid var(--border);border-radius:4px;padding:12px;margin-bottom:8px;">
  <!-- content -->
</div>
```
For highlighted cards, add `border-left:3px solid {color}`.

### Stat Display Pattern
```html
<div class="stat-card fire">
  <div class="stat-icon">&#x1F525;</div>
  <div class="stat-label">DPS</div>
  <div class="stat-value" style="color:var(--fire);">2.84M</div>
</div>
```

### Issue Card Pattern
```html
<div class="issue-card warning">
  <div class="issue-icon">&#x26A0;</div>
  <div class="issue-text">
    <div class="issue-title">Warning title</div>
    <div class="issue-fix">Remedy: how to fix</div>
  </div>
</div>
```
Severities: `critical` (red), `warning` (yellow), `info` (blue).

### Button Pattern
```html
<button style="padding:8px 16px;background:var(--border-blood);border:1px solid var(--blood-bright);border-radius:4px;color:var(--blood-bright);font-family:'Cinzel',serif;font-size:11px;font-weight:700;cursor:pointer;">Invoke</button>
```

### Equipment Grid Pattern (PoE character screen layout)
```
Row 1:  [Weapon]  [Helmet]   [Shield]
Row 2:  [Gloves]  [Body]     [Boots]
Row 3:  [Ring 1]  [Belt]     [Ring 2]
Row 4:     —      [Amulet]      —
```

## OUTPUT FORMAT

Always output complete, self-contained HTML that can be inserted into `pob-advisor.html`.
Use CSS custom properties (var(--fire), var(--bg-card), etc.) defined in the existing file.
Match the existing code style (inline styles for one-offs, CSS classes for reusable patterns).
