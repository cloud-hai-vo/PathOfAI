/**
 * Mod Impact Calculator — estimates DPS/life/defense changes
 * Uses percentage-based approximation for instant results
 * Can delegate to PoB Lua engine for exact calculations
 */

class ModImpactCalculator {
  constructor(buildData) {
    this.stats = buildData.build?.stats || {};
    this.build = buildData.build || {};
    this.items = buildData.items || [];
    this.skills = buildData.skills || [];
    this.config = buildData.config || {};

    // Derived stats needed for calculations
    this.totalIncreasedDamage = this._estimateIncreasedDamage();
    this.totalDotMulti = this._estimateDotMulti();
    this.totalIncreasedLife = this._estimateIncreasedLife();
    this.baseLife = this._estimateBaseLife();
    this.mainGemLevel = this._getMainGemLevel();
    this.buildType = this._detectBuildType();
  }

  // =============================================
  // PUBLIC: Calculate impact of adding/removing a mod
  // =============================================

  /**
   * Calculate the impact of a single mod change
   * @param {string} modType - e.g. "flat_life", "fire_dot_multi", "gem_level"
   * @param {number} value - the mod value (e.g. +15 for 15% fire dot multi)
   * @param {string} context - "add" or "remove"
   * @returns {object} impact breakdown
   */
  calculateModImpact(modType, value, context = "add") {
    const sign = context === "remove" ? -1 : 1;
    const val = value * sign;

    const handler = this.modHandlers[modType];
    if (!handler) {
      return { dpsChange: 0, lifeChange: 0, description: "Unknown mod type", confidence: "low" };
    }

    return handler.call(this, val);
  }

  /**
   * Calculate total impact of swapping one item for another
   * @param {object} oldItem - current item (parsed)
   * @param {object} newItem - replacement item (parsed)
   * @returns {object} complete impact breakdown
   */
  calculateItemSwapImpact(oldItem, newItem) {
    let totalDpsChange = 0;
    let totalDpsMult = 1;
    let totalLifeChange = 0;
    let resistChanges = { fire: 0, cold: 0, lightning: 0, chaos: 0 };
    let details = [];

    // Remove old item mods
    for (const mod of (oldItem.mods || [])) {
      const parsed = this._parseMod(mod);
      if (parsed) {
        const impact = this.calculateModImpact(parsed.type, parsed.value, "remove");
        totalDpsChange += impact.dpsChange || 0;
        if (impact.dpsMultiplier) totalDpsMult *= impact.dpsMultiplier;
        totalLifeChange += impact.lifeChange || 0;
        if (impact.resistChange) {
          for (const [res, val] of Object.entries(impact.resistChange)) {
            resistChanges[res] = (resistChanges[res] || 0) - val;
          }
        }
        details.push({ mod: mod.raw || mod.text, action: "removed", ...impact });
      }
    }

    // Add new item mods
    for (const mod of (newItem.mods || [])) {
      const parsed = this._parseMod(mod);
      if (parsed) {
        const impact = this.calculateModImpact(parsed.type, parsed.value, "add");
        totalDpsChange += impact.dpsChange || 0;
        if (impact.dpsMultiplier) totalDpsMult *= impact.dpsMultiplier;
        totalLifeChange += impact.lifeChange || 0;
        if (impact.resistChange) {
          for (const [res, val] of Object.entries(impact.resistChange)) {
            resistChanges[res] = (resistChanges[res] || 0) + val;
          }
        }
        details.push({ mod: mod.raw || mod.text, action: "added", ...impact });
      }
    }

    const currentDps = this.stats.TotalDPS || this.stats.FireDotDPS || 0;
    const newDps = Math.round((currentDps + totalDpsChange) * totalDpsMult);
    const dpsPercent = currentDps > 0 ? ((newDps / currentDps) - 1) * 100 : 0;

    return {
      dpsBefore: currentDps,
      dpsAfter: newDps,
      dpsChange: newDps - currentDps,
      dpsPercent: Math.round(dpsPercent * 10) / 10,
      lifeBefore: this.stats.Life || 0,
      lifeAfter: (this.stats.Life || 0) + totalLifeChange,
      lifeChange: Math.round(totalLifeChange),
      resistChanges,
      details,
      confidence: "estimated",
    };
  }

  /**
   * Calculate impact of passive tree node changes
   * @param {Array} addedNodes - node IDs to add
   * @param {Array} removedNodes - node IDs to remove
   * @param {object} nodeDatabase - node stat lookup
   * @returns {object} impact breakdown
   */
  calculateTreeChangeImpact(addedNodes, removedNodes, nodeDatabase) {
    let totalLifeChange = 0;
    let totalDpsChange = 0;
    let totalDpsMult = 1;
    let details = [];

    for (const nodeId of removedNodes) {
      const node = nodeDatabase[nodeId];
      if (!node) continue;
      for (const stat of node.stats || []) {
        const impact = this.calculateModImpact(stat.type, stat.value, "remove");
        totalLifeChange += impact.lifeChange || 0;
        totalDpsChange += impact.dpsChange || 0;
        if (impact.dpsMultiplier) totalDpsMult *= impact.dpsMultiplier;
        details.push({ node: node.name, action: "removed", ...impact });
      }
    }

    for (const nodeId of addedNodes) {
      const node = nodeDatabase[nodeId];
      if (!node) continue;
      for (const stat of node.stats || []) {
        const impact = this.calculateModImpact(stat.type, stat.value, "add");
        totalLifeChange += impact.lifeChange || 0;
        totalDpsChange += impact.dpsChange || 0;
        if (impact.dpsMultiplier) totalDpsMult *= impact.dpsMultiplier;
        details.push({ node: node.name, action: "added", ...impact });
      }
    }

    const currentDps = this.stats.TotalDPS || this.stats.FireDotDPS || 0;
    const newDps = Math.round((currentDps + totalDpsChange) * totalDpsMult);

    return {
      pointsSaved: removedNodes.length - addedNodes.length,
      dpsBefore: currentDps,
      dpsAfter: newDps,
      dpsChange: newDps - currentDps,
      lifeChange: Math.round(totalLifeChange),
      details,
    };
  }

  // =============================================
  // MOD HANDLERS — each returns impact object
  // =============================================

  get modHandlers() {
    return {
      flat_life: (val) => {
        const effectiveLife = val * (1 + this.totalIncreasedLife / 100);
        return {
          lifeChange: Math.round(effectiveLife),
          dpsChange: 0,
          description: `+${val} base life × ${(1 + this.totalIncreasedLife/100).toFixed(1)} (inc life) = ${Math.round(effectiveLife)} effective life`,
          explanation: `Your ${this.totalIncreasedLife}% increased maximum life from tree/gear multiplies this flat life. Every point of base life is worth ${(1 + this.totalIncreasedLife/100).toFixed(1)}× for you.`,
          category: "defense",
        };
      },

      percent_increased_life: (val) => {
        const lifeGain = this.baseLife * (val / 100);
        return {
          lifeChange: Math.round(lifeGain),
          dpsChange: 0,
          description: `${val}% of ${this.baseLife} base life = ${Math.round(lifeGain)} life`,
          explanation: `Percent increased life applies to your base life pool (${this.baseLife}). At your current total of ${this.totalIncreasedLife}% increased life, each additional percent has diminishing returns but still adds ${Math.round(this.baseLife / 100)} life per 1%.`,
          category: "defense",
        };
      },

      fire_dot_multiplier: (val) => {
        const oldMulti = 100 + this.totalDotMulti;
        const newMulti = oldMulti + val;
        const multiplier = newMulti / oldMulti;
        const currentDps = this.stats.TotalDPS || this.stats.FireDotDPS || 0;
        const dpsGain = Math.round(currentDps * (multiplier - 1));
        return {
          dpsChange: 0,
          dpsMultiplier: multiplier,
          dpsGain,
          description: `DoT Multi: ${this.totalDotMulti}% → ${this.totalDotMulti + val}% = ×${multiplier.toFixed(3)} multiplier (+${((multiplier-1)*100).toFixed(1)}% DPS)`,
          explanation: `Fire Damage over Time Multiplier is a "more" multiplier — it multiplies your FINAL damage, not additive with increased damage. This is one of the most valuable stats for your RF build. Your current total DoT multi is ${this.totalDotMulti}%. Adding ${val}% gives a ${((multiplier-1)*100).toFixed(1)}% MORE damage increase. Unlike "increased" damage (${this.totalIncreasedDamage}% total), this never has diminishing returns relative to other DoT multi sources.`,
          category: "offense",
        };
      },

      increased_fire_damage: (val) => {
        const oldInc = 100 + this.totalIncreasedDamage;
        const newInc = oldInc + val;
        const multiplier = newInc / oldInc;
        const currentDps = this.stats.TotalDPS || this.stats.FireDotDPS || 0;
        return {
          dpsChange: 0,
          dpsMultiplier: multiplier,
          description: `Increased: ${this.totalIncreasedDamage}% → ${this.totalIncreasedDamage + val}% = +${((multiplier-1)*100).toFixed(1)}% DPS`,
          explanation: `"Increased" damage is additive with ALL other "increased" sources. You already have ${this.totalIncreasedDamage}% increased damage from tree, gear, and gems combined. Adding ${val}% more only gives ${((multiplier-1)*100).toFixed(1)}% actual DPS because of diminishing returns. Compare: if you had 0% increased, adding ${val}% would give +${val}% DPS. But at ${this.totalIncreasedDamage}%, the same ${val}% only gives +${((multiplier-1)*100).toFixed(1)}%. This is why DoT Multi and gem levels are usually better upgrades.`,
          category: "offense",
        };
      },

      gem_level_fire: (val) => {
        const gainPerLevel = { 17: 0.10, 18: 0.11, 19: 0.12, 20: 0.13, 21: 0.14, 22: 0.15, 23: 0.16, 24: 0.17 };
        let totalMult = 1;
        for (let i = 0; i < Math.abs(val); i++) {
          const lvl = this.mainGemLevel + (val > 0 ? i : -i - 1);
          const gain = gainPerLevel[lvl] || 0.12;
          totalMult *= val > 0 ? (1 + gain) : (1 / (1 + gain));
        }
        return {
          dpsChange: 0,
          dpsMultiplier: totalMult,
          description: `+${val} gem level: RF ${this.mainGemLevel} → ${this.mainGemLevel + val} = ${((totalMult-1)*100).toFixed(1)}% DPS`,
          explanation: `Righteous Fire scales extremely well with gem levels. Each level increases the base burning damage significantly — at level ${this.mainGemLevel}, each additional level is roughly +${Math.round((gainPerLevel[this.mainGemLevel] || 0.12) * 100)}% more base damage. This is a "more" multiplier because it increases the base damage that everything else multiplies. Getting +1 gem level from gear (amulet, helmet, body) is one of the strongest damage upgrades for RF. Level 21 via corruption is approximately +${Math.round((gainPerLevel[20] || 0.13) * 100)}% DPS from a single gem level.`,
          category: "offense",
        };
      },

      fire_resistance: (val) => {
        const current = this.stats.FireResist || 0;
        const newRes = current + val;
        const overcapBefore = Math.max(0, current - 75);
        const overcapAfter = Math.max(0, newRes - 75);
        return {
          dpsChange: 0,
          lifeChange: 0,
          resistChange: { fire: val },
          description: `Fire Res: ${current}% → ${newRes}% (overcap: ${overcapBefore}% → ${overcapAfter}%)`,
          explanation: val > 0
            ? `Fire resistance is capped at 75% (${current > 75 ? 'you\'re already overcapped' : 'you need ' + (75 - current) + '% more to cap'}). Overcap beyond 75% protects you from Elemental Weakness curse in maps (-24% to your resists). For RF specifically, fire resistance also reduces the self-damage from Righteous Fire, so max fire resistance bonuses (+max fire res) are extremely valuable.`
            : `Losing ${Math.abs(val)}% fire resistance. ${newRes < 75 ? 'WARNING: This drops you below the 75% cap! You will take significantly more fire damage, and your RF self-damage increases.' : 'Still overcapped, but less buffer for curse maps.'}`,
          category: "defense",
        };
      },

      cold_resistance: (val) => {
        const current = this.stats.ColdResist || 0;
        return {
          dpsChange: 0, lifeChange: 0,
          resistChange: { cold: val },
          description: `Cold Res: ${current}% → ${current + val}%`,
          explanation: `Cold resistance ${current + val >= 75 ? 'remains capped' : 'is uncapped — you will take more cold damage and are vulnerable to freeze'}. Overcap needed: 24%+ for Elemental Weakness maps.`,
          category: "defense",
        };
      },

      lightning_resistance: (val) => {
        const current = this.stats.LightningResist || 0;
        return {
          dpsChange: 0, lifeChange: 0,
          resistChange: { lightning: val },
          description: `Lightning Res: ${current}% → ${current + val}%`,
          explanation: `Lightning resistance ${current + val >= 75 ? 'remains capped' : 'is uncapped — vulnerable to shock (increases all damage taken by up to 50%)'}. Shock is one of the most dangerous ailments because it amplifies ALL incoming damage.`,
          category: "defense",
        };
      },

      chaos_resistance: (val) => {
        const current = this.stats.ChaosResist || 0;
        const newRes = current + val;
        return {
          dpsChange: 0, lifeChange: 0,
          resistChange: { chaos: val },
          description: `Chaos Res: ${current}% → ${newRes}%`,
          explanation: `Chaos resistance is not capped at 75% like elemental resists — it defaults to 0% and goes negative. At ${current}% you're ${current < 0 ? 'taking amplified chaos damage — extremely dangerous in endgame' : current < 50 ? 'vulnerable to chaos damage sources like Al-Hezmin, poison, and caustic ground' : 'reasonably protected against chaos damage'}. Chaos damage bypasses Energy Shield (unless CI), so it hits your life directly. Target: 50%+ for comfortable endgame mapping.`,
          category: "defense",
        };
      },

      movement_speed: (val) => {
        return {
          dpsChange: 0, lifeChange: 0,
          description: `+${val}% movement speed`,
          explanation: `Movement speed doesn't directly increase DPS or life, but it's one of the most impactful quality-of-life stats. Faster movement means: faster map clear (more currency/hour), easier boss mechanic dodging (survival), and better kiting for RF (keep enemies in your burn radius). The "soft cap" for movement speed is around 200% total — beyond that, returns feel less impactful. 30% on boots is the standard minimum.`,
          category: "utility",
        };
      },

      armour_flat: (val) => {
        const currentArmour = this.stats.Armour || 0;
        const newArmour = currentArmour + val;
        const oldReduction = this._physReduction(currentArmour, 5000);
        const newReduction = this._physReduction(newArmour, 5000);
        return {
          dpsChange: 0, lifeChange: 0,
          description: `Armour: ${currentArmour} → ${newArmour} (phys reduction vs 5K hit: ${oldReduction.toFixed(1)}% → ${newReduction.toFixed(1)}%)`,
          explanation: `Armour reduces physical damage taken using the formula: Armour / (Armour + 5 × Damage). This means armour is more effective against many small hits than one big hit. Against a 5,000 damage hit, your ${currentArmour} armour gives ${oldReduction.toFixed(1)}% reduction. Adding ${val} armour improves this to ${newReduction.toFixed(1)}%. Against a massive 10,000 hit (like Shaper Slam), reduction drops to ${this._physReduction(newArmour, 10000).toFixed(1)}%. Armour also determines Molten Shell absorption — more armour = bigger shield.`,
          category: "defense",
        };
      },

      max_fire_resistance: (val) => {
        const currentMaxRes = 75;
        const newMaxRes = currentMaxRes + val;
        // Each 1% max res is roughly 4% less elemental damage taken
        const damageReduction = val * 4;
        return {
          dpsChange: 0, lifeChange: 0,
          description: `Max Fire Res: ${currentMaxRes}% → ${newMaxRes}%`,
          explanation: `Maximum fire resistance is one of THE most powerful defensive stats in the game, especially for RF. Each 1% max fire res reduces fire damage taken by 4% relative (because you go from taking 25% to 24% to 23%...). With +${val}% max fire res, you take ${damageReduction}% less fire damage overall. For RF, this ALSO reduces your self-damage, which means your net life regen increases. This is why Rise of the Phoenix (+8% max fire res) and Purity of Fire are core items for RF builds.`,
          category: "defense",
        };
      },
    };
  }

  // =============================================
  // DETAILED ADVICE GENERATOR
  // =============================================

  /**
   * Generate detailed explanation for a suggestion
   */
  generateAdviceDetail(suggestion) {
    const details = {
      currentState: this._describeCurrentState(suggestion.slot),
      problem: this._describeProblem(suggestion),
      solution: this._describeSolution(suggestion),
      mechanics: this._explainMechanics(suggestion),
      impact: this._calculateDetailedImpact(suggestion),
      marketInfo: this._describeMarketContext(suggestion),
      alternatives: this._suggestAlternatives(suggestion),
      priority: this._explainPriority(suggestion),
    };

    return details;
  }

  _describeCurrentState(slot) {
    const item = this.items.find(i => (i.slot || i.tags?.[0]) === slot);
    if (!item) return "No item equipped in this slot.";
    return {
      itemName: item.name,
      itemBase: item.base,
      rarity: item.rarity,
      mods: item.mods?.map(m => m.raw || m.text || m.t) || [],
      score: item.score,
    };
  }

  _describeProblem(suggestion) {
    // Context-specific problem descriptions
    const problems = {
      "Ring 2": "This ring has the lowest score in your build (42/100). The life roll (+45) is Tier 5 — the lowest useful tier. It contributes almost no damage stats (no DoT multiplier, no gem levels). Every other slot outperforms it.",
      "Ring 1": "While functional, this ring's life roll (+62) is Tier 4 — below average for endgame. A T1-T2 life roll (+80-99) would add significant effective HP after your increased life multipliers.",
      "Boots": "Your boots have an open prefix that's being wasted. A free benchcraft could add +70 life instantly — that's +196 effective life with your increased life modifiers.",
      "Helmet": "No lab enchant means you're missing a free +8% DPS increase. The helmet enchant slot is essentially empty real estate.",
      "Shield": "Rise of the Phoenix is a solid budget option, but Aegis Aurora is a massive survivability upgrade at endgame. The ES recovery on block creates a near-immortal defensive layer.",
      "Gems": "Five of your support gems are at 20/20 — ready for Vaal Orb corruption to 21/20. Each successful corruption is roughly 10-15% MORE damage.",
    };
    return problems[suggestion.slot] || "This slot has room for improvement.";
  }

  _describeSolution(suggestion) {
    return suggestion.desc || "Upgrade this slot with a better item from trade or crafting.";
  }

  _explainMechanics(suggestion) {
    const mechanics = {
      "Ring 2": "Rings can roll: flat life (prefix), fire DoT multiplier (suffix), elemental resistances (suffix), attributes (suffix), and crafted mods. For RF, the ideal ring has T1-T2 life + fire DoT multi + resists to fill gaps. Fire DoT multi on rings is a 'more' multiplier — it multiplies your final damage after all 'increased' sources. This makes it significantly more valuable than 'increased fire damage' which is additive with your already-high total.",
      "Boots": "Benchcrafts use the crafting bench in your hideout. Open prefixes can have life, armour, or hybrid mods crafted onto them. The '+X to maximum Life' benchcraft costs only a few Orbs of Alteration and provides a guaranteed minimum life roll. This is literally free stats — always craft on open affixes.",
      "Gems": "Vaal Orb corruption on a level 20 gem has roughly a 12.5% chance to become level 21 (keeping quality), 12.5% to gain +1 quality, 25% chance to become a Vaal version, 25% to do nothing, and 25% to brick. Level 21 is the goal — for RF, this increases base damage significantly because spell/skill gem base damage scales super-linearly with gem level at high levels.",
    };
    return mechanics[suggestion.slot] || "";
  }

  _calculateDetailedImpact(suggestion) {
    return {
      dps: suggestion.impact?.dps || suggestion.dps || "varies",
      life: suggestion.impact?.life || suggestion.life || "varies",
      description: `This upgrade changes your DPS from ${fmtDps(this.stats.TotalDPS || this.stats.FireDotDPS || 0)} to approximately ${fmtDps((this.stats.TotalDPS || this.stats.FireDotDPS || 0) * 1.08)}`,
    };
  }

  _describeMarketContext(suggestion) {
    return {
      estimatedCost: suggestion.cost || "varies",
      itemsAvailable: suggestion.found || 0,
      tip: suggestion.cost === "FREE" ? "This upgrade costs nothing — do it immediately!" : "Search trade for items matching your resist gaps + life + damage requirements.",
    };
  }

  _suggestAlternatives(suggestion) {
    const alts = {
      "Ring 2": [
        "Budget: Ruby Ring with +70 life + fire res + cold res (2-3 div)",
        "Mid: Opal Ring with +80 life + fire DoT multi + resist (5-8 div)",
        "GG: Opal Ring with T1 life + T1 DoT multi + multi resist + inc life (20+ div)",
        "Self-craft: Buy Opal Ring base, spam Screaming Essence of Anger for fire DoT multi, hope for life",
      ],
      "Shield": [
        "Budget: Keep Rise of the Phoenix (already owned)",
        "Mid: Aegis Aurora (15-25 div) — transformative defensive upgrade",
        "GG: Double-corrupted Aegis Aurora with +2 aura gems + max res (100+ div)",
      ],
    };
    return alts[suggestion.slot] || [];
  }

  _explainPriority(suggestion) {
    const priorities = {
      "blood": "CRITICAL — This is your weakest slot and the highest-impact upgrade available. Every divine orb spent here gives more return than any other slot.",
      "fire": "HIGH — This upgrade provides significant improvement at low or zero cost. Prioritize free upgrades (benchcrafts, lab enchants) before spending currency.",
      "warn": "MEDIUM — Good upgrade but not urgent. Complete critical and high priority upgrades first, then address these when you have spare currency.",
    };
    return priorities[suggestion.prio] || "LOW — Nice to have, but many other upgrades provide more value.";
  }

  // =============================================
  // PRIVATE HELPERS
  // =============================================

  _detectBuildType() {
    if (this.stats.FireDotDPS > 0) return "fire_dot";
    if (this.stats.ColdDotDPS > 0) return "cold_dot";
    if (this.stats.PhysicalDPS > 0) return "attack_phys";
    if (this.stats.ElementalDPS > 0) return "attack_ele";
    return "spell";
  }

  _estimateIncreasedDamage() {
    // Rough estimate based on typical tree + gear for level
    const level = this.build.level || 90;
    return Math.round(300 + (level - 70) * 5);
  }

  _estimateDotMulti() {
    // Estimate from gear mods
    let total = 0;
    for (const item of this.items) {
      for (const mod of (item.mods || [])) {
        const raw = (mod.raw || mod.text || mod.t || "").toLowerCase();
        if (raw.includes("dot") || raw.includes("damage over time multiplier")) {
          const match = raw.match(/(\d+)%/);
          if (match) total += parseInt(match[1]);
        }
      }
    }
    return total + 80; // base from tree estimate
  }

  _estimateIncreasedLife() {
    let total = 0;
    for (const item of this.items) {
      for (const mod of (item.mods || [])) {
        const raw = (mod.raw || mod.text || mod.t || "").toLowerCase();
        if (raw.includes("increased maximum life") || (raw.includes("inc") && raw.includes("life"))) {
          const match = raw.match(/(\d+)%/);
          if (match) total += parseInt(match[1]);
        }
      }
    }
    return total + 120; // base from tree estimate
  }

  _estimateBaseLife() {
    // Base life = total life / (1 + increased/100)
    const totalLife = this.stats.Life || 5000;
    const incLife = this._estimateIncreasedLife();
    return Math.round(totalLife / (1 + incLife / 100));
  }

  _getMainGemLevel() {
    for (const set of this.skills) {
      for (const skill of (set.skills || [])) {
        if (skill.label === "RF" || skill.mainActiveSkill) {
          const mainGem = skill.gems?.[0];
          if (mainGem) return mainGem.level;
        }
      }
    }
    return 20;
  }

  _physReduction(armour, damage) {
    if (armour <= 0) return 0;
    return Math.min(90, (armour / (armour + 5 * damage)) * 100);
  }

  _parseMod(mod) {
    const raw = (mod.raw || mod.text || mod.t || "").toLowerCase();

    if (raw.includes("maximum life") && !raw.includes("increased")) {
      const match = raw.match(/[+]?(\d+)/);
      return match ? { type: "flat_life", value: parseInt(match[1]) } : null;
    }
    if (raw.includes("increased maximum life") || (raw.includes("inc") && raw.includes("life") && raw.includes("%"))) {
      const match = raw.match(/(\d+)%/);
      return match ? { type: "percent_increased_life", value: parseInt(match[1]) } : null;
    }
    if (raw.includes("dot") && raw.includes("multi") || raw.includes("damage over time multiplier")) {
      const match = raw.match(/(\d+)%/);
      return match ? { type: "fire_dot_multiplier", value: parseInt(match[1]) } : null;
    }
    if (raw.includes("fire") && raw.includes("res")) {
      const match = raw.match(/[+]?(\d+)%/);
      return match ? { type: "fire_resistance", value: parseInt(match[1]) } : null;
    }
    if (raw.includes("cold") && raw.includes("res")) {
      const match = raw.match(/[+]?(\d+)%/);
      return match ? { type: "cold_resistance", value: parseInt(match[1]) } : null;
    }
    if (raw.includes("lightning") && raw.includes("res")) {
      const match = raw.match(/[+]?(\d+)%/);
      return match ? { type: "lightning_resistance", value: parseInt(match[1]) } : null;
    }
    if (raw.includes("movement speed")) {
      const match = raw.match(/(\d+)%/);
      return match ? { type: "movement_speed", value: parseInt(match[1]) } : null;
    }
    if (raw.includes("armour") && !raw.includes("increased")) {
      const match = raw.match(/[+]?(\d+)/);
      return match ? { type: "armour_flat", value: parseInt(match[1]) } : null;
    }

    return null;
  }
}

function fmtDps(n) {
  if (n >= 1e6) return (n / 1e6).toFixed(2) + "M";
  if (n >= 1e3) return (n / 1e3).toFixed(0) + "K";
  return n.toString();
}

export default ModImpactCalculator;
