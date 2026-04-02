# Path of AI — Design Decisions & Naming Convention

This document captures all naming, theming, and architectural decisions made during
the design phase. All UI text, labels, AI personality, and visual choices MUST follow
these conventions.

---

## WHY THE NAME "PATH OF AI"

Direct wordplay on "Path of Exile". Instantly recognizable by every PoE player.
Includes "AI" which is the tool's core differentiator. Short, memorable, no trademark conflicts.

**Rejected names:** Vaal.AI (too niche), Exile Oracle AI (Oracle Corp trademark),
Wraeclast AI (hard to spell), PathSeer AI (good but "Path of AI" was cleaner).

---

## THE DARK POE VOCABULARY

Every UI element uses language from Path of Exile's dark, bloody, cursed atmosphere.
Never use generic terms like "Submit", "Overview", "Assistant".

### Tab Names

| Generic | PoE-Themed | Why |
|---------|-----------|-----|
| Overview | **Omen** | Prophetic sign of your build's state |
| Items | **Arsenal** | Your equipped gear is your arsenal for war |
| Suggestions | **Prophecy** | Foretelling what your build could become (also a PoE league) |
| AI Chat | **Grimoire** | Book of dark magic — consult forbidden knowledge |
| Map Mods | **Curse Map** | Maps have cursed dangerous mods |
| Checklist | **Blood Pact** | Binding oath — goals you've sworn to complete |
| Build Evolution | **Dark Path** | Choose your path forward (dark + "path" in Path of Exile) |

### AI Entity — "The Seer"

NOT Oracle (Oracle Corp trademark), NOT Assistant.

| Element | Term | Instead of |
|---------|------|-----------|
| AI name | **The Seer** | AI Assistant |
| Section header | "The Seer — Dark Counsel" | "AI Chat" |
| Open chat | "Seek the Seer" | "Ask AI" |
| Send button | **"Invoke"** | "Send" / "Submit" |
| Placeholder | "Whisper to the Seer" | "Type a message" |
| Quick action | "Why do I perish?" | "Why am I dying?" |

### Other Thematic Terms

| Term | Used For | PoE Connection |
|------|----------|---------------|
| **Harbinger** | Warnings/issues tab | Harbinger league — foretells danger |
| **Blood Rituals** | Quick apply actions | Rituals league + blood magic |
| **Blood Magic** | Undo system | PoE keystone — using life instead of mana |
| **Invoke** | Action buttons | PoE skill term — summoning power |
| **Exile** | User address | What all PoE NPCs call the player |
| **Divine** | Currency display | Divine Orb — PoE's primary currency |
| **Void** | AI references | Where the Elder dwells in PoE lore |
| **Corruption** | Purple accent, AI glow | Vaal corruption — core PoE mechanic |
| **Remedy** | Issue fix suggestions | More archaic than "Fix" |
| **The Arena** | Combat simulator | Where you fight |

### AI Personality Voice

The Seer speaks like a dark PoE NPC:
- Addresses user as **"Exile"**
- Slightly condescending but helpful (like Nessa or Sin)
- Uses dark metaphors (void, blood, corruption, death)
- Archaic language ("perish" not "die", "invoke" not "use")
- References specific PoE mechanics accurately
- Provides real numbers despite the dramatic tone

Examples:
- "I have peered into the void and examined your build, Exile."
- "Your Ring 2 is barely worthy of a white mob drop."
- "The harbinger of your death whispers two truths..."
- "Practically naked against corruption" (low chaos res)

---

## COLOR PALETTE

Colors match PoE's actual in-game colors exactly.

### Base Theme (Warm Dark — matching PoE's actual UI)

**Important:** Use warm brown-blacks, NOT purple-blacks. PoE's UI is warm/earthy.

| Color | Hex | Usage |
|-------|-----|-------|
| Background | `#0c0a08` | Main bg (warm dark brown-black) |
| Panel | `#141210` | Panel bg (warm dark) |
| Card | `#1c1914` | Card bg (warm brown-black) |
| Hover | `#252018` | Hover state |
| Border | `#3a3028` | Default border (warm brown) |
| Blood border | `#5a1a1a` | Header, active accents |
| Blood bright | `#d03030` | Active tab, header glow |
| Gold | `#9a7a2a` / `#dab040` | Currency, T1 tier, gold accents |

### Text Colors (High Contrast — must be readable)

| Color | Hex | Usage | Contrast |
|-------|-----|-------|----------|
| Text | `#e8e0d0` | Body text | High (AA compliant on dark bg) |
| Text bright | `#f8f2e8` | Headings, important values | Very high |
| Text muted | `#a09080` | Secondary info, labels | Medium |
| Text dim | `#706050` | Tertiary, subtle hints | Low (intentional) |

### Element Colors (PoE exact)

| Element | Hex | Usage |
|---------|-----|-------|
| Fire | `#960000` | Fire damage (dark blood red — PoE exact) |
| Fire bright | `#cf3c18` | Fire text on dark bg (brighter variant) |
| Cold | `#366492` | Cold damage (steel blue — PoE exact) |
| Lightning | `gold` (#ffd700) | Lightning damage (CSS gold — PoE exact) |
| Chaos | `#d02090` | Chaos damage (**magenta-pink**, NOT purple — PoE exact) |
| Physical | `#c4a882` | Armour, phys damage |
| Life | `#c41e1e` | Life pool |
| Corrupted | `#d20000` | Corrupted items, danger |
| Success | `#4ae63a` | Quest green, capped, complete |
| Crafted | `#b8daf2` | Crafted mod text (light blue) |

### Rarity Colors (PoE exact)

| Rarity | Hex |
|--------|-----|
| Normal | `#c8c8c8` |
| Magic | `#8888ff` |
| Rare | `#ffff77` |
| Unique | `#af6025` / `#e8a444` |

### Tier Colors

| Tier | Color | Hex |
|------|-------|-----|
| T1 | Gold (divine) | `#e8d44d` |
| T2 | Green | `#3ec41e` |
| T3 | Blue | `#3674c2` |
| T4 | Gray | `#7a6b55` |
| T5 | Red | `#c41e1e` |
| Special | Purple | `#9955dd` |

---

## TYPOGRAPHY

| Font | Usage | Why |
|------|-------|-----|
| **Cinzel** | Headers, tab names, section titles | Gothic serif — medieval dark fantasy inscriptions |
| **Crimson Text** | Body text, descriptions | Warm serif, "Crimson" = blood theme |
| **JetBrains Mono** | Numbers, stats, DPS, prices | Monospace precision for data |

---

## EQUIPMENT LAYOUT

The Arsenal tab shows items in PoE's character screen layout:

```
Row 1:  [Weapon]  [Helmet]   [Shield]
Row 2:  [Gloves]  [Body]     [Boots]
Row 3:  [Ring 1]  [Belt]     [Ring 2]
Row 4:     —      [Amulet]      —
```

Items use PoE CDN artwork in production (`web.poecdn.com`).
Prototype uses emoji/SVG placeholders.

---

## FEEDBACK SYSTEM

In-app bug reports post directly to GitHub Issues via the app's GitHub token.
Users don't need a GitHub account. Called "Whisper to the Void".
Supports: screenshots, video, text, PoB builds, app logs.
Privacy: build data anonymized by default.
