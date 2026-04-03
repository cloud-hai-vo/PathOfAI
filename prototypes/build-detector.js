/**
 * Build Detector — identifies main skill, archetype, playstyle,
 * missing components, and suggests skill/build evolution paths
 */

class BuildDetector {
  constructor(buildData) {
    this.build = buildData.build || {};
    this.stats = buildData.build?.stats || {};
    this.items = buildData.items || [];
    this.skills = buildData.skills || [];
    this.tree = buildData.tree || {};
    this.config = buildData.config || {};

    // Run detection on construction
    this.mainSkill = this.detectMainSkill();
    this.secondarySkills = this.detectSecondarySkills();
    this.dpsType = this.detectDPSType();
    this.archetype = this.detectArchetype();
    this.playstyle = this.detectPlaystyle();
    this.allGemIds = this.collectAllGemIds();
  }

  // =============================================
  // MAIN SKILL DETECTION
  // =============================================

  detectMainSkill() {
    const mainGroupIndex = (this.build.mainSocketGroup || 1) - 1;

    for (const skillSet of this.skills) {
      const skills = skillSet.skills || [];
      const mainSkill = skills[mainGroupIndex];

      if (mainSkill) {
        const activeGem = mainSkill.gems?.find(g => g.skillId || g.gemId === mainSkill.mainActiveSkill) || mainSkill.gems?.[0];
        if (activeGem) {
          const gemInfo = GEM_DATABASE[activeGem.gemId] || {};
          return {
            gemId: activeGem.gemId,
            skillId: activeGem.skillId || activeGem.gemId,
            name: gemInfo.name || activeGem.gemId,
            level: activeGem.level,
            quality: activeGem.quality,
            slot: mainSkill.slot,
            label: mainSkill.label,
            damageType: gemInfo.damageType || "unknown",
            element: gemInfo.element || "physical",
            mechanic: gemInfo.mechanic || "self-cast",
            tags: gemInfo.tags || [],
            supportGems: mainSkill.gems?.filter(g => g.gemId !== activeGem.gemId) || [],
          };
        }
      }
    }

    // Fallback: find skill matching highest DPS stat
    return this._fallbackMainSkill();
  }

  _fallbackMainSkill() {
    const dpsType = this.detectDPSType();
    for (const skillSet of this.skills) {
      for (const skill of (skillSet.skills || [])) {
        for (const gem of (skill.gems || [])) {
          if (gem.skillId) {
            const info = GEM_DATABASE[gem.gemId] || {};
            if (info.damageType === dpsType) {
              return {
                gemId: gem.gemId, skillId: gem.skillId, name: info.name || gem.gemId,
                level: gem.level, quality: gem.quality, slot: skill.slot,
                label: skill.label, damageType: info.damageType, element: info.element,
                mechanic: info.mechanic, tags: info.tags || [],
                supportGems: skill.gems?.filter(g => g.gemId !== gem.gemId) || [],
              };
            }
          }
        }
      }
    }
    return null;
  }

  // =============================================
  // DPS TYPE DETECTION
  // =============================================

  detectDPSType() {
    const candidates = [
      { stat: "FireDotDPS", type: "fire_dot" },
      { stat: "ColdDotDPS", type: "cold_dot" },
      { stat: "ChaosDotDPS", type: "chaos_dot" },
      { stat: "BleedDPS", type: "bleed" },
      { stat: "PoisonDPS", type: "poison" },
      { stat: "IgniteDPS", type: "ignite" },
      { stat: "PhysicalDPS", type: "phys_attack" },
      { stat: "FireDPS", type: "fire_hit" },
      { stat: "ColdDPS", type: "cold_hit" },
      { stat: "LightningDPS", type: "lightning_hit" },
      { stat: "TotalDPS", type: "mixed" },
    ];

    let best = { type: "unknown", value: 0 };
    for (const c of candidates) {
      const val = this.stats[c.stat] || 0;
      if (val > best.value) best = { type: c.type, value: val };
    }
    return best.type;
  }

  // =============================================
  // ARCHETYPE DETECTION
  // =============================================

  detectArchetype() {
    const skill = this.mainSkill?.gemId || "";
    const asc = this.build.ascendClassName || "";
    const cls = this.build.className || "";

    // Direct lookup
    const key = `${skill}+${asc}`;
    if (ARCHETYPE_DATABASE[key]) return ARCHETYPE_DATABASE[key];

    // Infer from stats
    return this._inferArchetype(skill, asc, cls);
  }

  _inferArchetype(skill, asc, cls) {
    const dps = this.dpsType;
    const life = this.stats.Life || 0;
    const es = this.stats.EnergyShield || 0;

    let archetype = {
      id: `${skill}_${asc}`.toLowerCase(),
      name: `${this.mainSkill?.name || skill} ${asc}`,
      category: "unknown",
      strengths: [],
      weaknesses: [],
    };

    // Categorize by defense style
    if (es > life * 2) archetype.category = "es_based";
    else if (es > life * 0.5) archetype.category = "hybrid";
    else archetype.category = "life_based";

    // Categorize by offense style
    if (dps.includes("dot")) archetype.offenseType = "damage_over_time";
    else if (dps.includes("attack")) archetype.offenseType = "attack";
    else archetype.offenseType = "spell";

    return archetype;
  }

  // =============================================
  // PLAYSTYLE DETECTION
  // =============================================

  detectPlaystyle() {
    const traits = [];

    // Defense style
    const block = this.stats.BlockChance || 0;
    const dodge = this.stats.DodgeChance || 0;
    const armour = this.stats.Armour || 0;
    const evasion = this.stats.Evasion || 0;
    const regen = this.stats.LifeRegen || 0;

    if (block >= 50) traits.push("max_block");
    else if (block >= 30) traits.push("moderate_block");
    if (armour >= 30000) traits.push("high_armour");
    if (evasion >= 30000) traits.push("high_evasion");
    if (regen >= 2000) traits.push("high_regen");

    // Offense style
    const dps = this.stats.TotalDPS || this.stats.FireDotDPS || 0;
    if (dps >= 5000000) traits.push("high_dps");
    else if (dps >= 2000000) traits.push("moderate_dps");
    else traits.push("low_dps");

    // Movement
    const hasShieldCharge = this.allGemIds.includes("ShieldCharge");
    const hasWhirl = this.allGemIds.includes("WhirlingBlades");
    const hasFlameDash = this.allGemIds.includes("FlameDash");
    if (hasShieldCharge || hasWhirl) traits.push("melee_movement");
    if (hasFlameDash) traits.push("blink_movement");

    // Classify overall
    let style = "balanced";
    if (traits.includes("max_block") && traits.includes("high_regen")) style = "immortal_tank";
    else if (traits.includes("high_armour") && traits.includes("moderate_block")) style = "tanky_facetank";
    else if (traits.includes("high_evasion")) style = "dodge_kite";
    else if (traits.includes("high_dps") && !traits.includes("high_armour")) style = "glass_cannon";

    return { style, traits, description: PLAYSTYLE_DESCRIPTIONS[style] || style };
  }

  // =============================================
  // SECONDARY SKILL DETECTION
  // =============================================

  detectSecondarySkills() {
    const secondary = [];
    const mainId = this.mainSkill?.gemId;

    for (const skillSet of this.skills) {
      for (const skill of (skillSet.skills || [])) {
        for (const gem of (skill.gems || [])) {
          if (gem.skillId && gem.gemId !== mainId) {
            secondary.push({
              gemId: gem.gemId,
              name: GEM_DATABASE[gem.gemId]?.name || gem.gemId,
              level: gem.level,
              quality: gem.quality,
              slot: skill.slot,
              role: this.classifyGemRole(gem.gemId),
              enabled: gem.enabled !== false,
            });
          }
        }
      }
    }
    return secondary;
  }

  classifyGemRole(gemId) {
    for (const [role, gems] of Object.entries(GEM_ROLES)) {
      if (gems.includes(gemId)) return role;
    }
    return "unknown";
  }

  collectAllGemIds() {
    const ids = [];
    for (const set of this.skills) {
      for (const skill of (set.skills || [])) {
        for (const gem of (skill.gems || [])) {
          ids.push(gem.gemId);
        }
      }
    }
    return ids;
  }

  // =============================================
  // MISSING COMPONENT DETECTION
  // =============================================

  detectMissingComponents() {
    const issues = [];

    // Movement skill
    const movementGems = ["ShieldCharge", "FlameDash", "LeapSlam", "Dash", "WhirlingBlades", "FrostBlink", "LightningWarp", "Flicker"];
    if (!this.allGemIds.some(id => movementGems.includes(id))) {
      issues.push({
        type: "missing_movement", severity: "high", category: "skill",
        message: "No movement skill detected",
        suggestion: this._bestMovementSkill(),
        explanation: "Movement skills are essential for dodging boss mechanics and efficient mapping. Without one, you rely entirely on walk speed, which is too slow to avoid telegraphed attacks like Shaper Slam or Sirus Die Beam.",
        fixSteps: ["Add Shield Charge (1R socket) linked with Faster Attacks", "Alternative: Flame Dash (1B socket) for instant blink", "Best: Shield Charge + Faster Attacks + Fortify for damage reduction"],
      });
    }

    // Guard skill
    const guardGems = ["MoltenShell", "Steelskin", "ImmortalCall", "BoneArmour", "VaalMoltenShell"];
    if (!this.allGemIds.some(id => guardGems.includes(id))) {
      issues.push({
        type: "missing_guard", severity: "high", category: "skill",
        message: "No guard skill detected",
        suggestion: this._bestGuardSkill(),
        explanation: `Guard skills create a temporary damage shield. With your ${(this.stats.Armour||0).toLocaleString()} armour, Molten Shell would absorb approximately ${Math.round((this.stats.Armour||0) * 0.2).toLocaleString()} damage. Link with Cast When Damage Taken (CWDT) for automatic activation whenever you take a hit.`,
        fixSteps: ["Add CWDT (level 1) + Molten Shell (level 10) in any 2-link", "CWDT triggers automatically — no button press needed", "Keep CWDT at level 1 for maximum trigger frequency", "Molten Shell level must match CWDT level requirement"],
      });
    }

    // Curse
    const curseGems = ["Flammability", "ElementalWeakness", "Enfeeble", "TemporalChains", "Despair", "Vulnerability", "Punishment", "Conductivity", "Frostbite", "Poachers", "Warlords", "Assassins"];
    const hasCurseOnHitRing = this.items.some(i => (i.mods || []).some(m => {
      const raw = (m.raw || m.text || m.t || "").toLowerCase();
      return raw.includes("curse") && raw.includes("on hit");
    }));
    if (!this.allGemIds.some(id => curseGems.includes(id)) && !hasCurseOnHitRing) {
      issues.push({
        type: "missing_curse", severity: "medium", category: "skill",
        message: "No curse applied to enemies",
        suggestion: this._bestCurse(),
        explanation: `Curses significantly boost your damage or defense. For your fire DoT build, Flammability reduces enemy fire resistance by up to 44%, directly increasing all your fire damage. The easiest method is crafting "Curse Enemies with Flammability on Hit" on a ring — this is passive, no socket needed, no button press.`,
        fixSteps: ["Cheapest: Craft curse-on-hit on ring (bench craft, 1 suffix)", "Alternative: Arcanist Brand + Flammability (auto-applies to nearby enemies)", "Advanced: Blasphemy Support + Flammability (aura, reserves mana)"],
      });
    }

    // Defensive aura
    const defAuras = ["Determination", "Grace", "Discipline", "PurityOfElements"];
    if (!this.allGemIds.some(id => defAuras.includes(id))) {
      issues.push({
        type: "missing_defensive_aura", severity: "high", category: "aura",
        message: "No defensive aura active",
        suggestion: "Add Determination — roughly doubles your armour",
        explanation: "Determination is the strongest single defensive aura for armour-based builds. It adds a massive flat armour bonus plus a percentage increase, roughly doubling your armour. This also doubles your Molten Shell absorption, making it an incredibly efficient defense layer.",
        fixSteps: ["Add Determination (1R socket)", "Needs ~50% mana reservation", "Link with Enlighten Support to reduce reservation", "May need to drop another aura to fit"],
      });
    }

    // Offensive aura check for DoT builds
    if (this.dpsType.includes("dot")) {
      const dotAuras = ["Malevolence"];
      const hasDotAura = this.allGemIds.some(id => dotAuras.includes(id));
      if (!hasDotAura) {
        issues.push({
          type: "missing_dot_aura", severity: "medium", category: "aura",
          message: "No damage-over-time aura (Malevolence)",
          suggestion: "Malevolence gives 20% more DoT DPS + skill effect duration",
          explanation: "Malevolence is the best offensive aura for any DoT build. It gives approximately 20% MORE damage over time (not increased — this is multiplicative with everything else). It also increases skill effect duration, which boosts your damage uptime. The trade-off: 50% mana reservation, which may require dropping Vitality or getting more reservation efficiency.",
          fixSteps: ["Add Malevolence (1G socket, 50% reservation)", "May need to drop Vitality to fit", "Alternative: get -mana reservation efficiency on gear/tree", "Consider: losing Vitality regen vs gaining 20% more DPS"],
        });
      }
    }

    // Life flask
    // Check if build has RF (needs special flask considerations)
    const hasRF = this.allGemIds.includes("RighteousFire");
    if (hasRF) {
      issues.push({
        type: "rf_flask_warning", severity: "info", category: "flask",
        message: "RF build — life flasks have reduced effectiveness",
        suggestion: "Ensure you have enough regen to sustain without flask spam",
        explanation: "Righteous Fire's degen means life flasks feel less impactful since you're constantly losing life. Focus on regeneration over flask recovery. Your net regen (regen minus RF degen) should be at least +200/s for comfortable mapping. Use utility flasks (Ruby, Granite, Quicksilver) over multiple life flasks.",
      });
    }

    return issues;
  }

  _bestMovementSkill() {
    const hasShield = this.items.some(i => (i.slot || "").toLowerCase().includes("shield") || (i.tags || []).includes("shield"));
    if (hasShield) return "Shield Charge — best for shield builds, also triggers Fortify";
    
    const mainElement = this.mainSkill?.element;
    if (mainElement === "fire") return "Flame Dash — instant blink, fire themed";
    if (mainElement === "cold") return "Frostblink — cold themed, good for kiting";
    return "Flame Dash — versatile, works everywhere";
  }

  _bestGuardSkill() {
    const armour = this.stats.Armour || 0;
    if (armour >= 10000) return `Molten Shell — absorbs ${Math.round(armour * 0.2).toLocaleString()} damage with your armour`;
    
    const life = this.stats.Life || 0;
    if (life >= 5000) return "Steelskin — flat absorption, doesn't scale with armour";
    
    return "Immortal Call — brief physical immunity";
  }

  _bestCurse() {
    if (this.dpsType.includes("fire")) return "Flammability — reduces enemy fire resistance by up to 44%";
    if (this.dpsType.includes("cold")) return "Frostbite — reduces enemy cold resistance";
    if (this.dpsType.includes("lightning")) return "Conductivity — reduces enemy lightning resistance";
    if (this.dpsType.includes("chaos") || this.dpsType.includes("poison")) return "Despair — reduces enemy chaos resistance + increased DoT taken";
    if (this.dpsType.includes("phys") || this.dpsType.includes("bleed")) return "Vulnerability — increased phys damage taken + bleed faster";
    return "Elemental Weakness — reduces all elemental resistances";
  }

  // =============================================
  // CONTENT-SPECIFIC SUGGESTIONS
  // =============================================

  suggestForContent(contentType) {
    const suggestions = {
      gemSwaps: this._suggestGemSwaps(contentType),
      auraSwaps: this._suggestAuraSwaps(contentType),
      flaskSwaps: this._suggestFlaskSwaps(contentType),
      gearSwaps: this._suggestGearSwaps(contentType),
      warnings: this._contentWarnings(contentType),
    };
    return suggestions;
  }

  _suggestGemSwaps(content) {
    const swaps = [];

    if (this.mainSkill?.gemId === "RighteousFire") {
      if (content === "bossing") {
        swaps.push({
          out: "Efficacy", into: "Concentrated Effect",
          impact: { dps: "+22%", area: "-38%" },
          explanation: "Concentrated Effect provides a massive MORE damage multiplier while reducing AoE radius. Against a single boss target, you don't need the AoE — the boss is standing in your RF already. This is the single biggest DPS swap available for bossing.",
          autoApply: true,
          revert: "Swap back to Efficacy for mapping",
        });
        swaps.push({
          out: "Swift Affliction", into: "Burning Damage (Awakened)",
          impact: { dps: "+5%", duration: "+26%" },
          explanation: "If you have Awakened Burning Damage, it provides slightly more damage than Swift Affliction for single target while not reducing duration. The duration penalty from Swift Affliction matters less for bossing but this is still an upgrade.",
          autoApply: true,
          condition: "Only if you own Awakened Burning Damage",
        });
      }

      if (content === "mapping") {
        swaps.push({
          out: "Concentrated Effect", into: "Increased Area of Effect",
          impact: { dps: "-35%", area: "+50%" },
          explanation: "For mapping, AoE is king. A larger RF radius means you clear entire packs by walking past them. Map monsters die in 1-2 ticks regardless, so the DPS loss is irrelevant. Your Fire Trap handles any tanky rares.",
          autoApply: true,
        });
      }

      if (content === "simulacrum") {
        swaps.push({
          out: "Efficacy", into: "Concentrated Effect",
          impact: { dps: "+22%", area: "-38%" },
          explanation: "Simulacrum wave 25+ requires maximum DPS. The arena is small, so reduced AoE barely matters. Monsters come to you — keep your RF tight and lethal.",
          autoApply: true,
        });
      }

      if (content === "delve") {
        swaps.push({
          out: "Concentrated Effect", into: "Increased AoE",
          impact: { dps: "-35%", area: "+50%" },
          explanation: "In Delve, larger RF radius = more light radius. You need to cover as much ground as possible in the darkness. DPS matters less than survival and coverage.",
          autoApply: true,
        });
      }
    }

    return swaps;
  }

  _suggestAuraSwaps(content) {
    const swaps = [];

    if (content === "bossing") {
      if (this.allGemIds.includes("Vitality") && !this.allGemIds.includes("Malevolence")) {
        swaps.push({
          out: "Vitality", into: "Malevolence",
          impact: { dps: "+20% more", regen: "-500/s" },
          explanation: "For boss fights, Malevolence's 20% MORE damage over time is transformative. You lose Vitality's life regen, but boss fights are about DPS checks — if you kill the boss faster, you take less total damage. Ensure your net regen stays positive without Vitality before swapping.",
          risk: "Medium — test your regen without Vitality first",
        });
      }
    }

    if (content === "delve") {
      swaps.push({
        out: null, into: "Purity of Elements",
        impact: { dps: "0%", defense: "ailment immunity" },
        explanation: "Purity of Elements grants full ailment immunity (freeze, shock, ignite, bleed). In deep Delve, this is extremely valuable. May need to drop another aura to fit.",
        condition: "If you have reservation space",
      });
    }

    return swaps;
  }

  _suggestFlaskSwaps(content) {
    const swaps = [];

    if (content === "bossing") {
      swaps.push({
        suggestion: "Swap Quicksilver → Sulphur Flask",
        explanation: "Bosses are fought in small arenas — movement speed is less important. Sulphur Flask creates Consecrated Ground, giving 40% increased damage and 6% life regen. The regen also helps sustain during long fights.",
      });
    }

    return swaps;
  }

  _suggestGearSwaps(content) {
    const swaps = [];

    if (content === "bossing") {
      swaps.push({
        slot: "Ring",
        suggestion: "Equip a ring with 'Curse Enemies with Flammability on Hit' for boss fights",
        explanation: "Against bosses with high fire resistance, Flammability is a massive DPS increase. A curse-on-hit ring applies it automatically via your Shield Charge or Fire Trap hits.",
      });
    }

    return swaps;
  }

  _contentWarnings(content) {
    const warnings = [];

    if (content === "simulacrum") {
      const dps = this.stats.TotalDPS || this.stats.FireDotDPS || 0;
      if (dps < 3000000) {
        warnings.push({
          severity: "high",
          message: `DPS too low for wave 25+ (${fmtDps(dps)}, need 3M+)`,
          explanation: "Simulacrum wave 25-30 spawns extremely tanky monsters with high elemental resistance. At your current DPS, you'll time out before clearing the wave. Focus on DPS upgrades before attempting deep Simulacrum.",
        });
      }
    }

    if (content === "delve") {
      if ((this.stats.ChaosResist || 0) < 30) {
        warnings.push({
          severity: "high",
          message: "Chaos resistance too low for deep Delve",
          explanation: "Deep Delve spawns chaos damage monsters that bypass Energy Shield. With only " + (this.stats.ChaosResist || 0) + "% chaos res, you'll get one-shot by Vaal Constructs and Chaos-based encounters.",
        });
      }
    }

    return warnings;
  }

  // =============================================
  // BUILD EVOLUTION PATHS
  // =============================================

  suggestBuildEvolution() {
    const archId = this.archetype?.id || "";

    // Check known evolution paths
    for (const [pattern, paths] of Object.entries(EVOLUTION_DATABASE)) {
      if (archId.includes(pattern)) {
        return paths.map(p => ({
          ...p,
          feasibility: this._assessFeasibility(p),
          currentProgress: this._assessProgress(p),
        }));
      }
    }

    // Generic evolution paths
    return this._genericEvolution();
  }

  _assessFeasibility(path) {
    const currency = 50; // Would come from stash scanner
    const cost = parseInt(path.cost) || 0;
    if (currency >= cost) return "ready";
    if (currency >= cost * 0.5) return "saving";
    return "long_term";
  }

  _assessProgress(path) {
    let completed = 0;
    const total = (path.changes || []).length;
    // Would check each change against current build state
    return { completed, total, percent: total > 0 ? Math.round(completed / total * 100) : 0 };
  }

  _genericEvolution() {
    return [
      {
        name: "Tankier",
        icon: "🛡",
        description: "Invest in defensive layers",
        changes: ["Upgrade armour on gear", "Add block nodes", "Get ailment immunity"],
        cost: "10-20 div",
        dpsChange: "0%",
        survivalChange: "+50%",
      },
      {
        name: "More DPS",
        icon: "⚔",
        description: "Push damage higher",
        changes: ["Upgrade gem levels", "Add cluster jewels", "Get awakened supports"],
        cost: "20-40 div",
        dpsChange: "+50-80%",
        survivalChange: "0%",
      },
      {
        name: "Speed",
        icon: "⚡",
        description: "Faster clear for mapping",
        changes: ["Get movement speed", "Increase AoE", "Add herald/aura"],
        cost: "5-15 div",
        dpsChange: "-10%",
        survivalChange: "-10%",
      },
    ];
  }

  // =============================================
  // FULL DETECTION REPORT
  // =============================================

  generateReport() {
    return {
      mainSkill: this.mainSkill,
      secondarySkills: this.secondarySkills,
      dpsType: this.dpsType,
      archetype: this.archetype,
      playstyle: this.playstyle,
      missingComponents: this.detectMissingComponents(),
      contentReadiness: {
        mapping: this.suggestForContent("mapping"),
        bossing: this.suggestForContent("bossing"),
        simulacrum: this.suggestForContent("simulacrum"),
        delve: this.suggestForContent("delve"),
      },
      evolutionPaths: this.suggestBuildEvolution(),
    };
  }
}

// =============================================
// DATABASES
// =============================================

const GEM_DATABASE = {
  RighteousFire: { name: "Righteous Fire", damageType: "fire_dot", element: "fire", mechanic: "self-cast", tags: ["fire", "dot", "aoe", "spell"] },
  FireTrap: { name: "Fire Trap", damageType: "fire_dot", element: "fire", mechanic: "trap", tags: ["fire", "dot", "trap", "aoe"] },
  ArcticBreath: { name: "Arctic Breath", damageType: "cold_dot", element: "cold", mechanic: "self-cast", tags: ["cold", "dot", "spell", "projectile"] },
  Spark: { name: "Spark", damageType: "lightning_hit", element: "lightning", mechanic: "self-cast", tags: ["lightning", "spell", "projectile"] },
  LightningArrow: { name: "Lightning Arrow", damageType: "lightning_hit", element: "lightning", mechanic: "attack", tags: ["lightning", "attack", "bow", "aoe"] },
  CycloneAttack: { name: "Cyclone", damageType: "phys_attack", element: "physical", mechanic: "attack", tags: ["physical", "attack", "melee", "aoe", "channelling"] },
  SummonRagingSpirit: { name: "Summon Raging Spirit", damageType: "fire_hit", element: "fire", mechanic: "minion", tags: ["fire", "minion", "spell"] },
  Determination: { name: "Determination", damageType: null, element: null, mechanic: "aura", tags: ["aura", "defense"] },
  Grace: { name: "Grace", damageType: null, element: null, mechanic: "aura", tags: ["aura", "defense"] },
  Malevolence: { name: "Malevolence", damageType: null, element: null, mechanic: "aura", tags: ["aura", "offense", "dot"] },
  Vitality: { name: "Vitality", damageType: null, element: null, mechanic: "aura", tags: ["aura", "defense", "regen"] },
  PurityOfFire: { name: "Purity of Fire", damageType: null, element: "fire", mechanic: "aura", tags: ["aura", "defense", "fire"] },
  ShieldCharge: { name: "Shield Charge", damageType: null, element: "physical", mechanic: "attack", tags: ["movement", "attack", "melee"] },
  FlameDash: { name: "Flame Dash", damageType: null, element: "fire", mechanic: "self-cast", tags: ["movement", "spell", "fire"] },
  MoltenShell: { name: "Molten Shell", damageType: null, element: "fire", mechanic: "self-cast", tags: ["guard", "defense", "fire"] },
  Steelskin: { name: "Steelskin", damageType: null, element: null, mechanic: "self-cast", tags: ["guard", "defense"] },
  Flammability: { name: "Flammability", damageType: null, element: "fire", mechanic: "self-cast", tags: ["curse", "fire"] },
};

const GEM_ROLES = {
  supplement_dps: ["FireTrap", "FlameWall", "OrbOfStorms", "ScorchingRay", "Armageddon Brand", "StormBrand", "BladeVortex"],
  aura_defense: ["Determination", "Grace", "Discipline", "PurityOfFire", "PurityOfIce", "PurityOfLightning", "PurityOfElements", "Vitality"],
  aura_offense: ["Malevolence", "Hatred", "Wrath", "Anger", "Zealotry", "Pride"],
  movement: ["ShieldCharge", "FlameDash", "LeapSlam", "Dash", "WhirlingBlades", "FrostBlink", "LightningWarp", "Flicker"],
  guard: ["MoltenShell", "VaalMoltenShell", "Steelskin", "ImmortalCall", "BoneArmour"],
  curse: ["Flammability", "ElementalWeakness", "Enfeeble", "TemporalChains", "Despair", "Vulnerability", "Punishment", "Conductivity", "Frostbite"],
  utility: ["BloodRage", "PhaseRun", "TempestShield", "ArcticArmour", "ConvocationMinion"],
  trigger: ["CastWhenDamage", "CastOnCrit", "CastWhileChannel"],
};

const ARCHETYPE_DATABASE = {
  "RighteousFire+Inquisitor": { id: "rf_inquisitor", name: "RF Inquisitor", category: "life_based", offenseType: "fire_dot", tier: "S", strengths: ["Excellent regen", "Tanky", "Simple playstyle", "Good clear"], weaknesses: ["Low single target", "Can't run no-regen maps", "Slow boss kills"] },
  "RighteousFire+Juggernaut": { id: "rf_juggernaut", name: "RF Juggernaut", category: "life_based", offenseType: "fire_dot", tier: "A", strengths: ["Maximum tankiness", "Unstoppable", "Endurance charges"], weaknesses: ["Lower DPS than Inquisitor", "Slow"] },
  "RighteousFire+Chieftain": { id: "rf_chieftain", name: "RF Chieftain", category: "life_based", offenseType: "fire_dot", tier: "A", strengths: ["Fire damage", "Totem synergy", "Leech"], weaknesses: ["Less regen than Inquisitor"] },
};

const EVOLUTION_DATABASE = {
  "rf_inquisitor": [
    {
      name: "Aegis Immortal",
      icon: "🛡",
      description: "Near-unkillable tank — the most popular RF endgame path",
      changes: [
        { change: "Shield: Rise of Phoenix → Aegis Aurora", done: false, cost: "15-25 div", impact: "+3000 eHP on block" },
        { change: "Tree: Respec into Glancing Blows + block nodes", done: false, cost: "5 regret orbs", impact: "+30% block chance" },
        { change: "Add Tempest Shield for +25% spell block", done: false, cost: "free", impact: "+shock immunity + block" },
        { change: "Gear: Get +max fire res elsewhere (Purity 23, tree)", done: false, cost: "2-5 div", impact: "Compensate losing Rise of Phoenix" },
      ],
      cost: "25-40 div",
      dpsChange: "-5%",
      survivalChange: "+300% effective HP",
      priority: "Recommended first — transforms survivability",
      explanation: "Aegis Aurora replenishes ES equal to 2% of armour on block. With your armour and block chance, this creates a constantly regenerating shield that makes you nearly immortal against anything except one-shots. This is the #1 recommended upgrade path for RF Inquisitor and the reason it's one of the top builds in the game.",
    },
    {
      name: "DPS Ascension",
      icon: "🔥",
      description: "Push RF damage to 5M+ for pinnacle bosses",
      changes: [
        { change: "Aura: Vitality → Malevolence", done: false, cost: "free", impact: "+20% more DoT DPS" },
        { change: "Amulet: Get +2 to fire gem levels", done: false, cost: "15-30 div", impact: "+25% DPS from gem levels" },
        { change: "Tree: Add 2x Large + 4x Medium cluster jewels", done: false, cost: "10-20 div", impact: "+30-40% DPS from notables" },
        { change: "Gems: All Awakened supports", done: false, cost: "15-25 div", impact: "+15% DPS total" },
        { change: "Helmet: Enchant + Elevated fire mods", done: false, cost: "10-20 div", impact: "+15% DPS" },
      ],
      cost: "50-100 div",
      dpsChange: "+80-120%",
      survivalChange: "-10% (less regen without Vitality)",
      priority: "After Aegis — push for Uber bosses",
      explanation: "This path focuses on scaling RF damage through gem levels (+1/+2 on gear), cluster jewel notables (Blowback, Burning Bright, Prismatic Heart), and Awakened support gems. The key trade-off is dropping Vitality for Malevolence — you lose regen but gain 20% MORE damage. Only do this after you have Aegis Aurora providing alternative sustain through ES on block.",
    },
    {
      name: "Speed Demon",
      icon: "⚡",
      description: "Optimized for fast map farming currency generation",
      changes: [
        { change: "Helmet: → Devoto's Devotion", done: false, cost: "1-3 div", impact: "+20% move speed, -life/res" },
        { change: "Anoint: Arsonist (AoE)", done: false, cost: "2 oils", impact: "+RF radius for pack clear" },
        { change: "Boot enchant: 16% attack speed if killed recently", done: false, cost: "free (lab)", impact: "Faster Shield Charge" },
        { change: "Add Flame Dash + Second Wind for gap closing", done: false, cost: "free", impact: "Blink over gaps and walls" },
      ],
      cost: "5-15 div",
      dpsChange: "-15%",
      survivalChange: "-20%",
      priority: "For dedicated mapping alt or when build is comfortable",
      explanation: "This variant trades some tankiness and DPS for raw speed. Devoto's Devotion gives massive movement speed and attack speed (for faster Shield Charge), at the cost of life and armour. Combined with AoE investment, your RF radius covers entire screen widths, letting you sprint through maps. Best as a second configuration — keep your tanky setup for bosses.",
    },
  ],
};

const PLAYSTYLE_DESCRIPTIONS = {
  immortal_tank: "Nearly unkillable — facetanks most content, high block + regen sustain",
  tanky_facetank: "Very tanky — walks into packs and survives, high armour + moderate block",
  dodge_kite: "Evasion-based — avoids hits, kites enemies, dies when hit but rarely gets hit",
  glass_cannon: "High damage, low defense — kills before getting killed, risky playstyle",
  balanced: "Moderate offense and defense — versatile but not specialized",
};

function fmtDps(n) {
  if (n >= 1e6) return (n / 1e6).toFixed(1) + "M";
  if (n >= 1e3) return (n / 1e3).toFixed(0) + "K";
  return n.toString();
}

module.exports = BuildDetector;
