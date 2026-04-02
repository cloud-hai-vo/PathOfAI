# Path of AI — Advanced Systems Specification

## 1. ITEM IMAGE SYSTEM

### Image Sources (Priority Order)
```
1. PoE Official CDN (web.poecdn.com)
   → Used by: PoB, poe.ninja, all community tools
   → URL pattern: https://web.poecdn.com/image/Art/2DItems/{category}/{file}
   → Examples:
     Armours/Gloves/GlovesStr3.png
     Armours/Helmets/HelmetStr7.png
     Rings/Ring5.png
     Amulets/Amulet5.png
     Weapons/OneHandWeapons/OneHandSwords/OneHandSword1.png
   → Quality: Official 2D art, perfect quality
   → Rate limits: Generous for community tools

2. PoE Wiki (poewiki.net)
   → Has every unique item art
   → URL pattern: https://www.poewiki.net/wiki/Special:FilePath/{ItemName}.png
   → Good for unique items with specific art

3. PoeDB (poedb.tw)
   → Has item art + mod database
   → Alternative source if CDN is slow

4. Local Cache (our app stores fetched images)
   → First launch: download full item art pack (~50-100MB)
   → Store in: %AppData%/PathOfAI/cache/images/
   → Subsequent launches: serve from cache, update weekly
```

### Image Fetching Architecture
```
App startup
  ↓
Check local cache age
  ↓
If stale (>7 days) or missing:
  ↓
Download item art manifest from our GitHub repo
  → manifest.json lists all item base types + image URLs
  ↓
Fetch images in background (don't block UI)
  → Show placeholder SVG icons while loading
  → Replace with real images as they download
  ↓
Cache to disk with content hash
  → Only re-download changed images
```

### Image Mapping Logic
```javascript
class ItemImageResolver {
  // Map PoB item data to image URL
  resolve(item) {
    // 1. Unique items → exact art lookup
    if (item.rarity === "UNIQUE") {
      return this.resolveUnique(item.name);
    }
    
    // 2. Rare/Magic/Normal → base type art
    return this.resolveBaseType(item.base, item.tags);
  }
  
  resolveUnique(name) {
    // Database of unique item → image path
    const uniques = {
      "Rise of the Phoenix": "Art/2DItems/Armours/Shields/ShieldStrDex4.png",
      "Aegis Aurora": "Art/2DItems/Armours/Shields/ShieldStrInt5.png",
      "Devoto's Devotion": "Art/2DItems/Armours/Helmets/HelmetDex9.png",
      "Bottled Faith": "Art/2DItems/Flasks/SulphurFlaskUnique.png",
      // ... 1000+ unique items mapped
    };
    return `https://web.poecdn.com/image/${uniques[name]}`;
  }
  
  resolveBaseType(base, tags) {
    // Map base type name to image
    const bases = {
      "Astral Plate": "Art/2DItems/Armours/BodyArmours/BodyStrDex4.png",
      "Royal Burgonet": "Art/2DItems/Armours/Helmets/HelmetStr7.png",
      "Titan Gauntlets": "Art/2DItems/Armours/Gloves/GlovesStr3.png",
      "Titan Greaves": "Art/2DItems/Armours/Boots/BootsStr3.png",
      "Ruby Ring": "Art/2DItems/Rings/RubyRing.png",
      "Opal Ring": "Art/2DItems/Rings/OpalRing.png",
      "Crystal Belt": "Art/2DItems/Belts/CrystalBelt.png",
      "Turquoise Amulet": "Art/2DItems/Amulets/TurquoiseAmulet.png",
      // ... all base types mapped
    };
    return `https://web.poecdn.com/image/${bases[base]}`;
  }
}

// Gem icons
class GemImageResolver {
  resolve(gemId) {
    // Gems have consistent naming on CDN
    const gems = {
      "RighteousFire": "Art/2DItems/Gems/RighteousFire.png",
      "Determination": "Art/2DItems/Gems/Determination.png",
      "MoltenShell": "Art/2DItems/Gems/MoltenShell.png",
      "SupportBurningDamage": "Art/2DItems/Gems/Support/BurningDamage.png",
      // ... all gems mapped
    };
    return `https://web.poecdn.com/image/${gems[gemId]}`;
  }
}

// Skill icons for active skills
class SkillIconResolver {
  resolve(skillId) {
    // Skill icons are used in the skill bar
    return `https://web.poecdn.com/image/Art/2DArt/SkillIcons/${skillId}.png`;
  }
}
```

### Caching Strategy
```
%AppData%/PathOfAI/
  cache/
    images/
      items/
        unique/
          rise-of-the-phoenix.png
          aegis-aurora.png
        base/
          astral-plate.png
          royal-burgonet.png
      gems/
        righteous-fire.png
        determination.png
      skills/
        righteous-fire-icon.png
    manifest.json        ← tracks versions + hashes
    last-updated.json    ← timestamp of last sync
```

### Offline Support
- First launch downloads all images (~50-100MB)
- After that, works fully offline
- Background weekly update checks for new items
- League launch: auto-download new unique item art

---

## 2. MARKET INTELLIGENCE — WHEN TO BUY

### Price Trend Engine
```
Data source: poe.ninja API (free, no auth needed)
  → Fetches: item prices, currency rates, build popularity
  → Updates: every 5 minutes (cached locally)
  → History: poe.ninja provides 7-day price history
  → We store locally for full-league tracking
```

### Buy Timing Advisor
```
For each suggested upgrade, the system calculates:

1. Current price
2. 7-day price trend (rising/falling/stable)
3. League phase indicator
4. Buy recommendation

Example output:
┌─────────────────────────────────────────────┐
│ Aegis Aurora                                │
│                                             │
│ Current Price: 18 divine                    │
│ 7-Day Trend:  ▼ -15% (was 21 div)         │
│ League Phase:  Week 3 (prices stabilizing)  │
│                                             │
│ 🟢 BUY NOW — price is dropping and         │
│    approaching league-low. Historically     │
│    Aegis stabilizes around 15-18 div by     │
│    week 4, then rises slightly as supply    │
│    decreases.                               │
│                                             │
│ Price History:                              │
│ 40d ████████░░░░ Day 1                     │
│ 28d ██████░░░░░░ Day 3                     │
│ 22d █████░░░░░░░ Week 1                    │
│ 18d ████░░░░░░░░ Now ← YOU ARE HERE        │
│ 15d ███░░░░░░░░░ Predicted Week 4          │
│ 17d ████░░░░░░░░ Predicted Week 6          │
└─────────────────────────────────────────────┘
```

### League Economy Phase Detection
```
Phase 1: Day 1-3 (Chaos Economy)
  → Chaos orbs are king
  → Unique prices inflated 5-10x
  → "DO NOT buy uniques now — wait 3 days"
  → "SELL chaos-valued items immediately"

Phase 2: Day 3-7 (Transition)
  → Prices crashing fast
  → Divine orb establishing value
  → "Buy leveling uniques now (90% cheaper)"
  → "Start saving divines"

Phase 3: Week 1-3 (Divine Economy)
  → Prices stabilizing
  → Build-defining uniques settling
  → "Good time to buy mid-tier upgrades"
  → "Watch for underpriced items"

Phase 4: Week 3-6 (Stable)
  → Prices mostly stable
  → Best time for big purchases
  → "BUY endgame items now — best prices"
  → "Craft vs buy comparison most accurate"

Phase 5: Week 6+ (Late League)
  → Player count dropping
  → Some prices rising (less supply)
  → Some prices falling (less demand)
  → "Mirror-tier items cheapest now"
  → "Standard-viable items hold value"
```

### Per-Item Buy Advisor
```javascript
class BuyAdvisor {
  analyzePurchase(itemSearch, budget) {
    return {
      currentPrice: this.getCurrentPrice(itemSearch),
      priceHistory: this.getPriceHistory(itemSearch, 14), // 14 days
      trend: this.calculateTrend(itemSearch),
      leaguePhase: this.detectLeaguePhase(),
      recommendation: this.generateRecommendation(itemSearch),
      alternatives: this.findCheaperAlternatives(itemSearch),
      craftVsBuy: this.compareCraftVsBuy(itemSearch),
    };
  }
  
  generateRecommendation(itemSearch) {
    const trend = this.calculateTrend(itemSearch);
    const phase = this.detectLeaguePhase();
    
    if (trend === "dropping_fast" && phase <= 2) {
      return {
        action: "WAIT",
        reason: "Price dropping rapidly — early league deflation",
        waitUntil: "Price stabilizes (usually week 2-3)",
        confidence: "high",
      };
    }
    
    if (trend === "dropping_slow" && phase >= 3) {
      return {
        action: "BUY_NOW",
        reason: "Price approaching league-low, stable economy",
        confidence: "high",
      };
    }
    
    if (trend === "rising" && phase >= 4) {
      return {
        action: "BUY_NOW",
        reason: "Price rising — supply decreasing as players leave",
        confidence: "medium",
      };
    }
    
    if (trend === "stable") {
      return {
        action: "BUY_WHEN_READY",
        reason: "Price stable — no advantage to waiting",
        confidence: "high",
      };
    }
    
    return {
      action: "MONITOR",
      reason: "Unclear trend — set a price alert",
      confidence: "low",
    };
  }
  
  compareCraftVsBuy(itemSearch) {
    // Compare expected crafting cost vs market price
    return {
      buyPrice: 12, // divine
      craftCostAverage: 8, // divine (expected)
      craftCostWorstCase: 25, // divine (unlucky)
      recommendation: "Craft if you can handle variance",
      explanation: "Average craft cost is 8 div (67% of buy price), but worst case is 25 div. If you have 30+ div budget, crafting is more efficient. If tight on currency, buying is safer.",
    };
  }
}
```

### Price Alert System
```
User sets alerts:
  "Notify me when Aegis Aurora drops below 15 div"
  "Notify me when +1 fire gem amulet appears under 10 div"
  "Notify me when divine:chaos ratio changes more than 5%"

System checks every 5 minutes:
  → Fetches poe.ninja data
  → Compares against alerts
  → Desktop notification + optional sound
  → Overlay popup if playing PoE

Alert types:
  1. Price threshold: "Item X below Y price"
  2. Price change: "Item X dropped more than Z%"
  3. Snipe alert: "Specific item posted below market"
  4. Currency rate: "Exchange rate shifted"
  5. Trend reversal: "Item was dropping, now rising"
```

---

## 3. CUSTOM LOCAL AI MODEL FOR POE

### Why Custom Local Model?
```
Problems with general AI models (Claude/GPT/etc):
  → Don't know current league mechanics
  → Outdated PoE knowledge (patches change everything)
  → Can't access your build data natively
  → API costs per query
  → Requires internet
  → Privacy: some players don't want build data sent externally

Our local model solves all of these:
  → Trained specifically on PoE data
  → Updated every patch
  → Runs 100% on user's PC
  → Zero cost per query
  → Works offline
  → Complete privacy
```

### Architecture: Hybrid RAG + Fine-Tuned Small Model
```
┌──────────────────────────────────────────────┐
│                Path of AI                     │
│                                               │
│  ┌─────────────────────────────────────────┐  │
│  │  Custom Local AI ("The Seer Engine")    │  │
│  │                                         │  │
│  │  ┌───────────┐    ┌──────────────────┐  │  │
│  │  │ Small LLM │    │ PoE Knowledge DB │  │  │
│  │  │ (Fine-    │◄───│ (RAG Retrieval)  │  │  │
│  │  │  tuned)   │    │                  │  │  │
│  │  │ ~3-7B     │    │ • Mod database   │  │  │
│  │  │ params    │    │ • Skill data     │  │  │
│  │  └─────┬─────┘    │ • Unique items   │  │  │
│  │        │          │ • Passive tree    │  │  │
│  │        ▼          │ • Build guides   │  │  │
│  │  ┌───────────┐    │ • Patch notes    │  │  │
│  │  │ Build     │    │ • Price history  │  │  │
│  │  │ Context   │    │ • Craft recipes  │  │  │
│  │  │ Injector  │    └──────────────────┘  │  │
│  │  └───────────┘                          │  │
│  └─────────────────────────────────────────┘  │
│                                               │
│  Also available (optional, internet required): │
│  ┌──────────┐ ┌──────┐ ┌──────┐ ┌──────────┐ │
│  │ Claude   │ │ GPT  │ │Gemini│ │ Grok     │ │
│  │ API      │ │ API  │ │ API  │ │ API      │ │
│  └──────────┘ └──────┘ └──────┘ └──────────┘ │
└──────────────────────────────────────────────┘
```

### Step 1: Base Model Selection
```
Recommended base models to fine-tune:

Option A: Phi-3 Mini (3.8B params)
  → Size: ~2.3 GB (quantized Q4)
  → RAM: 4-6 GB
  → Speed: 20-40 tokens/sec on modern CPU
  → Pros: Very small, runs on any gaming PC
  → Cons: Less capable for complex reasoning

Option B: Llama 3.1 8B (8B params)  ← RECOMMENDED
  → Size: ~4.5 GB (quantized Q4)  
  → RAM: 6-8 GB
  → Speed: 15-30 tokens/sec on CPU, 60+ on GPU
  → Pros: Strong reasoning, good at following instructions
  → Cons: Needs decent RAM (most gaming PCs have 16GB+)

Option C: Mistral 7B (7B params)
  → Size: ~4 GB (quantized Q4)
  → RAM: 6-8 GB
  → Speed: Similar to Llama
  → Pros: Good at structured output, efficient
  → Cons: Slightly less creative

Gaming PCs have GPUs → can run these FAST:
  → RTX 3060+: 50-100 tokens/sec
  → RTX 4070+: 100-200 tokens/sec
  → CPU only: 15-30 tokens/sec (still usable)
```

### Step 2: Fine-Tuning Data Collection
```
Training data sources:

1. PoE Game Data (structured)
   → All item mods + tiers + weights (~50K entries)
   → All passive tree nodes + connections (~1.5K nodes)
   → All skill gems + support gems + interactions (~400 gems)
   → All unique items + mods (~1,200 uniques)
   → Crafting bench recipes + costs
   → Vendor recipes
   → Map mod effects
   → Boss mechanics + damage values
   → Source: PyPoE data extraction, RePoE, poedb

2. Build Knowledge (Q&A format)
   → Convert popular build guides into Q&A pairs
   → "Why is Determination good for RF?" → explanation
   → "What's the best support gem for RF?" → ranked list
   → ~10,000-50,000 Q&A pairs
   → Source: PoE forums, Reddit r/PathOfExile, YouTube transcripts

3. Crafting Knowledge
   → Step-by-step crafting guides → Q&A format
   → "How do I craft a +2 fire gem amulet?" → steps
   → Probability calculations
   → ~5,000 crafting Q&A pairs

4. Market Knowledge
   → Price patterns per league phase
   → Item valuation logic
   → Currency conversion strategies
   → ~3,000 economy Q&A pairs

5. Build Analysis (synthetic data)
   → Generate builds with known issues
   → Create "analyze this build" → "here are the problems"
   → Train model to identify common mistakes
   → ~20,000 synthetic analysis pairs

Total training data: ~50,000-100,000 high-quality examples
```

### Step 3: Fine-Tuning Process
```
Tools needed:
  → Axolotl or Unsloth (fine-tuning frameworks)
  → 1x GPU with 24GB VRAM (RTX 4090 or A100)
  → Or use cloud: RunPod/Lambda Labs (~$1-2/hr)

Training format (conversation style):
{
  "conversations": [
    {
      "role": "system",
      "content": "You are The Seer, an AI advisor for Path of Exile. You analyze builds, suggest upgrades, and explain game mechanics. You speak with dark, atmospheric PoE-flavored language."
    },
    {
      "role": "user", 
      "content": "My RF Inquisitor has 2.8M DPS and 6400 life. I have +15% fire DoT multi on one ring but my other ring only has +45 life and resists. What should I do?"
    },
    {
      "role": "assistant",
      "content": "Your Ring 2 is the weakest link in your build, Exile. A +45 life roll is Tier 5 — barely worth the socket. For an RF Inquisitor at your level, you need:\n\n1. Replace Ring 2 with: Opal Ring base (+25% ele damage implicit), T1-T2 life (+80-99), fire DoT multiplier (+15-18%), and resistance to fill gaps.\n\n2. Fire DoT Multiplier is a MORE multiplier — at your current ~180% total DoT multi, adding +15% gives (195/180) = 8.3% MORE final DPS. This is not additive like increased damage.\n\n3. Expected cost: 5-8 divine for a good ring, 15+ for a great one.\n\nThis single upgrade should push you to ~3.1M DPS with better life."
    }
  ]
}

Fine-tuning config:
  → LoRA rank: 32-64
  → Learning rate: 2e-4
  → Epochs: 3-5
  → Training time: 4-8 hours on RTX 4090
  → Output: LoRA adapter (~100-200MB)
```

### Step 4: RAG Knowledge Base
```
Even with fine-tuning, the model needs current data.
RAG (Retrieval Augmented Generation) provides this:

Knowledge Base structure:
  poe_knowledge/
    items/
      unique_items.json      ← all uniques + mods
      base_types.json        ← all base types
      mod_tiers.json         ← all mod tiers + values
    skills/
      active_gems.json       ← all active skill gems
      support_gems.json      ← all support gems
      interactions.json      ← gem interaction rules
    tree/
      passive_nodes.json     ← all nodes + stats
      cluster_jewels.json    ← cluster jewel data
      keystones.json         ← keystone effects
    crafting/
      bench_crafts.json      ← all bench crafts
      fossil_mods.json       ← fossil modifier weights
      essence_mods.json      ← essence guaranteed mods
      harvest_crafts.json    ← harvest craft options
    economy/
      price_history.json     ← cached from poe.ninja
      currency_rates.json    ← current exchange rates
    builds/
      popular_builds.json    ← top builds from poe.ninja
      build_guides.json      ← parsed guide summaries
    patches/
      current_patch.json     ← latest patch notes
      balance_changes.json   ← nerfs/buffs

RAG flow:
  User asks: "What's the best helmet for my RF build?"
    ↓
  1. Embed query using small embedding model
  2. Search knowledge base for relevant chunks
  3. Find: helmet mods, RF-relevant stats, popular RF helmets
  4. Inject retrieved context into prompt
  5. Local model generates answer with current data
    ↓
  Answer includes: specific helmets, current prices, mod tiers

Embedding model: all-MiniLM-L6-v2 (22MB, runs on CPU)
Vector store: SQLite with vector extension (no external DB needed)
```

### Step 5: Build Context Injection
```
Before every query, inject the user's current build:

System prompt template:
"""
You are The Seer, Path of AI's build advisor.

CURRENT BUILD DATA:
Class: {class} / Ascendancy: {ascendancy} / Level: {level}
Main Skill: {main_skill} (Level {gem_level}/{quality})
DPS: {dps} | Life: {life} | ES: {es}
Resists: Fire {fire_res}% | Cold {cold_res}% | Light {light_res}% | Chaos {chaos_res}%
Armour: {armour} | Block: {block}% | Regen: {regen}/s

EQUIPPED ITEMS:
{for each item: slot, name, mods with tiers, score, value}

PASSIVE TREE: {node count}, {key keystones}
AURAS: {list}
FLASKS: {list}

ISSUES DETECTED: {list from build analyzer}
BUDGET: {currency available}

Respond with PoE-accurate advice. Reference specific items, mods,
and mechanics. Include numbers and calculations.
"""

This gives the local model FULL context of the player's build
without needing to explain anything — it already knows.
```

### Step 6: Runtime Integration in Tauri App
```
┌─────────────────────────────────────────┐
│ Tauri App                               │
│                                         │
│  ┌──────────────────────────────────┐   │
│  │ llama.cpp (bundled, ~2MB)        │   │
│  │  → Runs GGUF model locally      │   │
│  │  → Uses GPU if available        │   │
│  │  → Falls back to CPU            │   │
│  └──────────────────────────────────┘   │
│                                         │
│  ┌──────────────────────────────────┐   │
│  │ Model files (user downloads):    │   │
│  │  → Base: llama-3.1-8b.Q4.gguf  │   │
│  │     (4.5 GB, one-time download) │   │
│  │  → LoRA: poe-seer-v1.gguf      │   │
│  │     (200 MB adapter)            │   │
│  │  → Embedding: minilm.gguf      │   │
│  │     (22 MB for RAG)            │   │
│  └──────────────────────────────────┘   │
│                                         │
│  ┌──────────────────────────────────┐   │
│  │ Knowledge Base (auto-updated):   │   │
│  │  → poe_knowledge/ (~50MB)       │   │
│  │  → Updated every patch          │   │
│  │  → Downloaded from our GitHub   │   │
│  └──────────────────────────────────┘   │
│                                         │
│  Total disk: ~5GB                       │
│  RAM usage: 6-8GB during inference      │
│  First query: ~2-3 sec                  │
│  Subsequent: <1 sec (model stays loaded)│
└─────────────────────────────────────────┘
```

### Step 7: Model Update Pipeline
```
Every PoE patch / league:
  1. Extract new game data (items, gems, tree)
  2. Generate new training examples
  3. Fine-tune updated LoRA adapter
  4. Update knowledge base JSONs
  5. Push to GitHub Releases:
     → poe-seer-v2.gguf (LoRA adapter, ~200MB)
     → poe_knowledge_v2.zip (~50MB)
  6. App auto-downloads on next launch
  
User sees: "Seer update available — new league data"
```

### Step 8: Hybrid Mode — Best of Both Worlds
```
Settings → AI Provider:

  ◉ The Seer (Local)        ← default, free, offline
    Fast for: build analysis, item scoring, gem swaps,
    crafting advice, mod explanations

  ○ Claude (API)             ← for complex reasoning
    Better for: creative build ideas, explaining
    complex interactions, "why" questions

  ○ GPT-4 (API)              ← alternative cloud
  ○ Gemini (API)             ← alternative cloud  
  ○ Grok (API)               ← alternative cloud
  ○ OpenRouter (OAuth)        ← access to all models

  ☑ Auto-escalate to cloud for complex queries
    "If The Seer confidence < 70%, ask cloud model"

Flow:
  User asks question
    ↓
  The Seer (local) generates answer
    ↓
  If confidence HIGH → show answer
  If confidence LOW → 
    "The Seer is uncertain. Consult a greater power?"
    [Use Claude] [Use GPT] [Accept local answer]
```

### Development Timeline for Custom Model
```
Month 1: Data Collection
  → Extract all PoE game data via PyPoE/RePoE
  → Scrape and clean build guides (respect robots.txt)
  → Generate synthetic training data
  → Build knowledge base JSONs

Month 2: Training
  → Prepare training data in conversation format
  → Fine-tune Llama 3.1 8B with LoRA
  → Test and evaluate on held-out questions
  → Iterate on training data quality

Month 3: Integration
  → Bundle llama.cpp in Tauri app
  → Build RAG pipeline with SQLite vectors
  → Build context injection system
  → Build hybrid escalation logic

Month 4: Polish
  → Optimize inference speed
  → Build model update pipeline
  → Create evaluation benchmarks
  → A/B test local vs cloud accuracy
```

### What The Local Seer Can Do (Without Internet)
```
✅ Analyze your build (scores, issues, suggestions)
✅ Explain any mod, mechanic, or interaction
✅ Suggest upgrades with reasoning
✅ Calculate DPS impact of changes
✅ Recommend passive tree changes
✅ Crafting step-by-step guides
✅ Map mod danger analysis
✅ Boss readiness assessment
✅ Gem swap recommendations
✅ Build evolution paths
✅ Answer "why" questions about PoE mechanics

❌ Cannot: check live market prices (needs internet)
❌ Cannot: compare with poe.ninja builds (needs internet)
❌ Cannot: know about new patches until updated
```

---

## 4. TRAINING RESOURCES & OPTIMIZATION

For the complete guide on free/open community resources for training, optimizing
training time, data formats, evaluation benchmarks, and community contribution
workflows, see:

**[AI-TRAINING-GUIDE.md](AI-TRAINING-GUIDE.md)**

Key highlights:
- **$0-5 total compute cost** using free GPU tiers (Colab, Kaggle)
- **2-3 weeks** optimized timeline (down from 4+ months)
- Only QueryNet needs LLM fine-tuning; other networks use custom small NNs
- QLoRA + Unsloth = 2-4x faster than standard LoRA training
- Smaller base model (Phi-3 3.8B) can match 8B for domain-specific tasks
- Community contribution workflow for continuous improvement
