# Path of AI — AI Model Training Guide

## Overview

This document covers the free/open community resources available for training The Seer Engine,
and strategies to optimize training time and cost.

The Seer Engine is a hybrid system of 5 specialized neural networks (~50-80MB total)
that runs locally on the user's PC with <100ms inference, no GPU required.

---

## 1. WHAT THE AI MODEL NEEDS

### Training Data Requirements Per Network

| Network | Size | Training Data Needed | Data Volume |
|---------|------|---------------------|-------------|
| ItemNet (scoring & comparison) | ~5MB | Item examples from poe.ninja/trade | 500K+ items |
| BuildNet (classification & issues) | ~8MB | Synthetic build analysis pairs | 20K+ examples |
| TreeNet (passive tree optimization) | ~6MB | Node data + pathing examples | ~1.5K nodes |
| QueryNet (user query understanding) | ~15MB | Q&A pairs from forums/Reddit/guides | 10K-50K pairs |
| EmbedNet (semantic search / RAG) | ~10MB | All PoE game data embeddings | Full game DB |
| ResponseGen (NL generation) | rule-based | Template + rule definitions | No ML training |

**Total training data target:** ~50K-100K high-quality examples

---

## 2. FREE / OPEN COMMUNITY RESOURCES

### 2.1 Game Data Extraction (Structured)

| Resource | What It Provides | License / Cost |
|----------|-----------------|----------------|
| [RePoE](https://github.com/brather1ng/RePoE) | All PoE game data exported as JSON — mods, gems, items, passive tree, base types, tags | Free, open source |
| [PyPoE](https://github.com/OmegaK2/PyPoE) | Python library to extract data directly from PoE .dat game files | Free, open source |
| [poedb.tw](https://poedb.tw) | Comprehensive mod database with weights, tiers, crafting info | Free (respect robots.txt) |
| [PoE Wiki (poewiki.net)](https://www.poewiki.net) | Full game mechanics, items, skills, boss damage values | CC-BY-SA, free |
| [PoB Community Fork data](https://github.com/PathOfBuildingCommunity/PathOfBuilding) | Lua calc engine + internal game data tables | Free, open source |

**Usage:** These provide the structured knowledge base (items, mods, gems, tree) that feeds
into RAG and trains ItemNet/BuildNet/TreeNet on verified game data.

### 2.2 Build Data (Training Examples)

| Resource | What It Provides | Access |
|----------|-----------------|--------|
| [poe.ninja API](https://poe.ninja) | Top builds by class/skill, item prices, economy data, build popularity | Free API, no auth needed |
| [poe.ninja build pages](https://poe.ninja/builds) | Thousands of endgame builds with full gear/tree/gem data | Free, scrapeable |
| Reddit r/PathOfExile | Build guides, Q&A discussions, crafting advice | Free (Reddit API, rate-limited) |
| PoE Official Forums | Thousands of detailed build guides with gear/tree/gems | Scrapeable (respect ToS) |
| YouTube build guides | Video descriptions often contain PoB codes + gear breakdowns | Transcripts via API |
| Pushshift / Arctic Shift | Historical Reddit data for large-scale Q&A extraction | Free academic dataset |

**Usage:** Convert build guides + Q&A threads into conversation-format training pairs for QueryNet.
Use poe.ninja builds for ItemNet/BuildNet training data (real builds with known scores).

### 2.3 AI / ML Frameworks (Free)

| Tool | Purpose | Why Use It |
|------|---------|-----------|
| [Unsloth](https://github.com/unslothai/unsloth) | LoRA/QLoRA fine-tuning | 2x faster, 60% less VRAM than standard training |
| [Axolotl](https://github.com/OpenAccess-AI-Collective/axolotl) | Fine-tuning framework | Supports LoRA, QLoRA, full fine-tune, many base models |
| [llama.cpp](https://github.com/ggerganov/llama.cpp) | Run GGUF models locally | CPU/GPU inference, small binary (~2MB), bundles in Tauri |
| [ONNX Runtime](https://onnxruntime.ai) | Run custom neural nets | Cross-platform, CPU optimized, ideal for ItemNet/BuildNet/TreeNet |
| [PyTorch](https://pytorch.org) | Train custom NNs | Industry standard, free, GPU accelerated |
| [Sentence-Transformers](https://sbert.net) | Embedding model for RAG | all-MiniLM-L6-v2 (22MB), pre-trained, no training needed |
| [sqlite-vss](https://github.com/asg017/sqlite-vss) | Vector search for RAG | No external DB needed, embeds in SQLite |
| [Hugging Face Hub](https://huggingface.co) | Model + dataset hosting | Free tier, community contributions, model sharing |

### 2.4 Free Compute for Training

| Platform | GPU Available | Limits | Best For |
|----------|-------------|--------|----------|
| [Google Colab](https://colab.research.google.com) | T4 (16GB VRAM) | ~12hr sessions, usage limits | QLoRA fine-tuning of 3-8B models |
| [Kaggle Notebooks](https://www.kaggle.com) | P100 (16GB VRAM) | 30 hrs/week | Longer training runs |
| [Lightning.ai](https://lightning.ai) | Free GPU credits | Limited monthly credits | Quick experiments |
| [Hugging Face Spaces](https://huggingface.co/spaces) | CPU / free GPU tier | Inference hosting | Testing + demo hosting |
| Local gaming PC (RTX 3060+) | 8-12GB VRAM | Unlimited | QLoRA with Unsloth |

**Total compute cost: $0-5 for the entire training pipeline.**

### 2.5 Community Collaboration Channels

| Channel | How to Leverage |
|---------|----------------|
| **Hugging Face community** | Host the fine-tuned model + training dataset publicly; invite community contributions and improvements |
| **PoE modding/tools community** | Many devs maintain game data extractors (RePoE, PyPoE contributors) — collaborate on data pipelines |
| **r/LocalLLaMA** | Active community of local model fine-tuners who can help with training approach, hyperparameters, evaluation |
| **Open source training data** | Publish the Q&A training pairs as a HF dataset — community will review, correct, and expand it for free |
| **PoE Discord servers** | Source domain experts for Q&A quality review and edge case discovery |
| **GitHub Discussions** | Let users submit build analysis corrections that feed back into training data |

---

## 3. OPTIMIZING TRAINING TIME

### 3.1 Current Baseline (from ADVANCED-SYSTEMS.md)

```
Base model:    Llama 3.1 8B
Method:        LoRA fine-tune
Hardware:      RTX 4090 (24GB VRAM)
Data:          50K-100K examples
Time:          4-8 hours
Cost:          $1-2/hr cloud GPU or own hardware
```

### 3.2 Optimization A — Use QLoRA Instead of LoRA (2-3x faster)

```
LoRA (original plan):   24GB VRAM required, ~4-8 hrs
QLoRA:                  8-12GB VRAM required, ~2-4 hrs
  - 4-bit quantized base model during training
  - Same output quality as full LoRA
  - Half the VRAM = fits on RTX 3060 (12GB) or free Colab T4
  - Unsloth makes QLoRA trivially easy to set up
```

### 3.3 Optimization B — Use Unsloth (2x Speedup)

```
Standard HF Trainer:    4-8 hours
With Unsloth:           2-4 hours (2x faster, 60% less memory)
  - Custom CUDA kernels for attention layers
  - Free, open source, drop-in replacement for HF Trainer
  - pip install unsloth → change 2 lines of code
```

### 3.4 Optimization C — Smaller, Smarter Base Model

```
Original plan:   Llama 3.1 8B     → 4-8 hrs training, 4.5GB model
Alternative 1:   Phi-3 Mini 3.8B  → 1-2 hrs training, 2.3GB model
Alternative 2:   Llama 3.2 3B     → 1-2 hrs training, 2.0GB model
Alternative 3:   Qwen 2.5 3B      → 1-2 hrs training, 2.0GB model

Why this works:
  - For PoE-specific tasks, domain-specific training data matters more
    than raw model size
  - A well-trained 3B model beats a generic 8B model on PoE questions
  - Smaller model = faster inference = better UX (<50ms vs <100ms)
  - Smaller download for users (2GB vs 4.5GB)
```

### 3.5 Optimization D — Progressive Training Strategy

```
Phase 1: Train on 10K high-quality examples → evaluate    (~30 min)
Phase 2: Add 20K more examples if quality gaps found      (~1 hr)
Phase 3: Full 50K-100K only if still not meeting benchmarks (~2-4 hrs)

Why:
  - Most quality gains come from the first 10-20K examples
  - Diminishing returns after that — data quality >> data quantity
  - Saves hours of wasted compute if 10K is already sufficient
  - Evaluate at each stage with held-out PoE questions
```

### 3.6 Optimization E — Distillation from Cloud Models (Free Data Generation)

```
Strategy: Use free tiers of cloud AI to generate training data

1. Send 5K build scenarios to Claude/GPT via free tiers
2. Collect high-quality analysis responses
3. Use those responses as training targets for the local model
   ("Teacher model" Claude/GPT trains "student model" local Seer)

Free tier estimates:
  - Claude free tier: ~20 messages/day x 90 days = ~1,800 examples
  - ChatGPT free tier: similar volume
  - Gemini free tier: higher limits
  - Combined with synthetic generation: 10K+ pairs at zero cost

Quality boost:
  - Cloud model responses are higher quality than scraped forum posts
  - Can be targeted at specific weak areas (crafting, boss mechanics)
  - Consistent format and tone (PoE-flavored "Seer" voice)
```

### 3.7 Optimization F — Skip Fine-Tuning Where Not Needed

Not every network in the Seer Engine needs LLM-style fine-tuning:

| Network | Best Approach | Training Time | Notes |
|---------|--------------|---------------|-------|
| ItemNet | Custom feed-forward NN (PyTorch → ONNX) | ~1-2 hrs on CPU | Structured input/output, no language needed |
| BuildNet | Custom NN + rule engine hybrid | ~1-2 hrs on CPU | Classification task, not generation |
| TreeNet | Graph algorithms + small NN | ~30 min on CPU | Pathfinding is algorithmic, not linguistic |
| QueryNet | Fine-tune small transformer | ~1-2 hrs on GPU | **Only network that needs LLM fine-tuning** |
| EmbedNet | Use pre-trained all-MiniLM-L6-v2 as-is | **0 training** | Pre-trained embeddings work out of the box |
| ResponseGen | Template + rule-based system | **0 training** | Structured data → templates → natural language |

**Key insight:** Only QueryNet actually needs LLM-style fine-tuning. The rest are better
served by small custom neural networks trained on structured data (minutes-to-hours on CPU)
or pre-trained models used as-is.

### 3.8 Combined Optimized Timeline

```
Original spec:   4 months data collection + training
Optimized:       2-3 weeks active work

Week 1: Data Collection (automated scripts)
  - Run RePoE export → structured game data JSONs
  - Fetch poe.ninja API → item prices + top builds
  - Generate synthetic Q&A pairs from build guides
  - Start cloud model distillation (runs over background days)
  Total: mostly automated, ~2 days of script writing

Week 2: Train Specialized NNs
  - ItemNet: train on 500K items from poe.ninja (PyTorch, CPU, ~2 hrs)
  - BuildNet: train on 20K synthetic builds (PyTorch, CPU, ~2 hrs)
  - TreeNet: train on pathing examples (PyTorch, CPU, ~30 min)
  - Export all to ONNX format for runtime
  Total: ~5 hours of training on local machine

Week 3: Fine-Tune QueryNet + Integration
  - QLoRA + Unsloth on Phi-3 Mini 3.8B or Llama 3.2 3B
  - 10K-20K Q&A examples (progressive approach)
  - Free Google Colab T4 GPU, ~1-2 hours
  - RAG setup: index knowledge base with all-MiniLM-L6-v2
  - Integration test in Tauri with ONNX Runtime + llama.cpp
  Total: ~3 hours training + 2 days integration

Final bundle:
  - ONNX models (ItemNet + BuildNet + TreeNet): ~20MB
  - QueryNet GGUF (quantized Phi-3 + LoRA): ~2.5GB
  - EmbedNet (all-MiniLM-L6-v2): ~22MB
  - Knowledge base JSONs: ~50MB
  - Total: ~2.6GB (user downloads once, auto-updates LoRA + knowledge base)
```

---

## 4. TRAINING DATA FORMAT

### Q&A Conversation Format (for QueryNet fine-tuning)

```json
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
      "content": "Your Ring 2 is the weakest link in your build, Exile. A +45 life roll is Tier 5 — barely worth the socket. For an RF Inquisitor at your level, you need:\n\n1. Replace Ring 2 with: Opal Ring base (+25% ele damage implicit), T1-T2 life (+80-99), fire DoT multiplier (+15-18%), and resistance to fill gaps.\n\n2. Fire DoT Multiplier is a MORE multiplier — at your current ~180% total DoT multi, adding +15% gives (195/180) = 8.3% MORE final DPS.\n\n3. Expected cost: 5-8 divine for a good ring, 15+ for a great one.\n\nThis single upgrade should push you to ~3.1M DPS with better life."
    }
  ]
}
```

### Item Scoring Format (for ItemNet training)

```json
{
  "slot": "Ring",
  "rarity": "RARE",
  "base": "Opal Ring",
  "level": 92,
  "build_archetype": "fire_dot",
  "mods": [
    {"stat": "maximum_life", "type": "flat", "value": 89, "tier": 1},
    {"stat": "fire_dot_multiplier", "type": "percent", "value": 16, "tier": 2},
    {"stat": "fire_resistance", "type": "percent", "value": 38, "tier": 2},
    {"stat": "cold_resistance", "type": "percent", "value": 41, "tier": 1}
  ],
  "score": 87,
  "price_divine": 8.5
}
```

### Build Analysis Format (for BuildNet training)

```json
{
  "input": {
    "class": "Templar",
    "ascendancy": "Inquisitor",
    "level": 92,
    "main_skill": "RighteousFire",
    "life": 6400,
    "dps": 2800000,
    "fire_res": 82,
    "cold_res": 76,
    "lightning_res": 75,
    "chaos_res": -12,
    "armour": 24000
  },
  "output": {
    "archetype": "fire_dot",
    "tier": "B",
    "issues": ["negative_chaos_res", "low_overcap_lightning"],
    "priority_upgrades": ["ring_2", "amulet", "helmet_enchant"],
    "phase": "mid_endgame"
  }
}
```

---

## 5. EVALUATION BENCHMARKS

### PoE Knowledge Accuracy Test
- 200 factual questions about PoE mechanics
- Target: 95%+ accuracy (wrong answers are worse than "I don't know")
- Examples:
  - "What is the max fire resistance with Rise of the Phoenix?" → 83%
  - "How much armour to get 50% phys reduction vs 5000 hit?" → 5000
  - "What support gem gives the most DPS for RF?" → Burning Damage / Elemental Focus

### Build Analysis Quality Test
- 50 builds with known issues (hand-labeled by PoE experts)
- Target: detect 90%+ of critical issues, <5% false positives
- Compare against PoB's own calculations for accuracy

### Query Understanding Test
- 100 user questions with labeled intent + entities
- Target: 90%+ intent classification, 85%+ entity extraction
- Examples:
  - "Why am I dying?" → intent: defense_analysis
  - "Best helmet for RF" → intent: item_recommendation, skill: RF

### Response Quality Test
- 50 build scenarios → evaluate response helpfulness (human eval)
- Target: responses rated "helpful" or "very helpful" 80%+ of the time
- Compare local model vs cloud model (Claude) quality

---

## 6. MODEL UPDATE PIPELINE

```
Every PoE patch / league start:

  1. Data team runs extraction scripts
     → RePoE export → new items, gems, tree changes
     → poe.ninja scrape → new meta builds
     → Generate updated Q&A pairs

  2. Retrain/update affected networks
     → ItemNet: retrain if new mod types added (~1 hr)
     → BuildNet: retrain if meta shifted significantly (~1 hr)
     → QueryNet: LoRA update with new league Q&A (~1-2 hrs)
     → Knowledge base JSONs: always updated

  3. Publish to GitHub Releases
     → poe-seer-v{N}.gguf (LoRA adapter, ~200MB)
     → poe-knowledge-v{N}.zip (knowledge base, ~50MB)
     → ONNX model updates if networks retrained (~20MB)

  4. App auto-downloads on next launch
     → "Seer update available — new league data"
     → Background download, no restart needed for knowledge base
     → Model swap requires restart
```

---

## 7. COMMUNITY CONTRIBUTION WORKFLOW

```
How community helps improve The Seer:

  1. Users report wrong advice via in-app feedback button
     → "The Seer said X but correct answer is Y"
     → Stored as correction pair

  2. Corrections reviewed by maintainers
     → Added to training dataset if valid
     → Knowledge base JSON updated immediately (no retraining needed)

  3. Community submits Q&A pairs via GitHub
     → PR to training-data repository
     → Reviewed + merged into next training batch

  4. PoE experts volunteer as data reviewers
     → Review generated Q&A pairs for accuracy
     → Flag hallucinations or outdated info
     → Quality gate before training data is used

  5. Monthly community model evaluation
     → Post 50 test questions, community rates answers
     → Track quality over time
     → Focus retraining on weak areas
```
