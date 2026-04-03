/**
 * Item Image Resolver — maps items to real game art
 * Fetches from PoE CDN (web.poecdn.com), caches locally
 * For Tauri desktop runtime
 */

const CDN_BASE = "https://web.poecdn.com/image";
const WIKI_BASE = "https://www.poewiki.net/wiki/Special:FilePath";

class ItemImageResolver {
  constructor(options = {}) {
    this.cacheDir = options.cacheDir || "";
    this.cache = new Map();
    this.pendingFetches = new Map();
  }

  // =============================================
  // PUBLIC API
  // =============================================

  /** Get image URL for any item (cached or CDN) */
  getItemImage(item) {
    const key = this._itemKey(item);
    if (this.cache.has(key)) return this.cache.get(key);

    const url = item.rarity === "UNIQUE"
      ? this._resolveUnique(item.name)
      : this._resolveBase(item.base, item.tags);

    this.cache.set(key, url);
    return url;
  }

  /** Get image URL for a gem */
  getGemImage(gemId) {
    const path = GEM_IMAGES[gemId];
    if (path) return `${CDN_BASE}/${path}`;
    // Fallback: construct from gemId
    return `${CDN_BASE}/Art/2DItems/Gems/${gemId}.png`;
  }

  /** Get skill bar icon for active skill */
  getSkillIcon(skillId) {
    const path = SKILL_ICONS[skillId];
    if (path) return `${CDN_BASE}/${path}`;
    return `${CDN_BASE}/Art/2DArt/SkillIcons/${skillId}.png`;
  }

  /** Get currency icon */
  getCurrencyImage(currencyName) {
    const path = CURRENCY_IMAGES[currencyName];
    if (path) return `${CDN_BASE}/${path}`;
    return null;
  }

  /** Get flask icon */
  getFlaskImage(flaskName, isUnique = false) {
    if (isUnique && UNIQUE_FLASK_IMAGES[flaskName]) {
      return `${CDN_BASE}/${UNIQUE_FLASK_IMAGES[flaskName]}`;
    }
    const path = FLASK_BASE_IMAGES[flaskName];
    if (path) return `${CDN_BASE}/${path}`;
    return null;
  }

  // =============================================
  // RESOLVERS
  // =============================================

  _resolveUnique(name) {
    const path = UNIQUE_ITEM_IMAGES[name];
    if (path) return `${CDN_BASE}/${path}`;
    // Fallback: try wiki
    const safeName = encodeURIComponent(name.replace(/ /g, "_"));
    return `${WIKI_BASE}/${safeName}_inventory_icon.png`;
  }

  _resolveBase(baseName, tags) {
    const path = BASE_TYPE_IMAGES[baseName];
    if (path) return `${CDN_BASE}/${path}`;
    // Fallback: guess from tags
    return this._guessBaseImage(baseName, tags);
  }

  _guessBaseImage(baseName, tags) {
    const tag = (tags || [])[0] || "";
    const categoryMap = {
      helmet: "Art/2DItems/Armours/Helmets/HelmetStr1.png",
      body_armour: "Art/2DItems/Armours/BodyArmours/BodyStr1.png",
      gloves: "Art/2DItems/Armours/Gloves/GlovesStr1.png",
      boots: "Art/2DItems/Armours/Boots/BootsStr1.png",
      shield: "Art/2DItems/Armours/Shields/ShieldStr1.png",
      ring: "Art/2DItems/Rings/Ring1.png",
      amulet: "Art/2DItems/Amulets/Amulet1.png",
      belt: "Art/2DItems/Belts/Belt1.png",
    };
    const path = categoryMap[tag] || categoryMap.ring;
    return `${CDN_BASE}/${path}`;
  }

  _itemKey(item) {
    return `${item.rarity}_${item.name || item.base}`;
  }
}

// =============================================
// IMAGE DATABASES
// =============================================

const UNIQUE_ITEM_IMAGES = {
  // Shields
  "Rise of the Phoenix": "Art/2DItems/Armours/Shields/ShieldStrDex6Unique.png",
  "Aegis Aurora": "Art/2DItems/Armours/Shields/ShieldStrInt5Unique.png",
  "Atziri's Reflection": "Art/2DItems/Armours/Shields/ShieldDex8Unique.png",
  "Prism Guardian": "Art/2DItems/Armours/Shields/ShieldStrDex4Unique.png",

  // Body Armours
  "Tabula Rasa": "Art/2DItems/Armours/BodyArmours/BodyStrDex1Unique.png",
  "Kaom's Heart": "Art/2DItems/Armours/BodyArmours/BodyStr3BUnique.png",
  "Brass Dome": "Art/2DItems/Armours/BodyArmours/BodyStr4Unique.png",
  "Belly of the Beast": "Art/2DItems/Armours/BodyArmours/BodyStr5Unique.png",
  "Lightning Coil": "Art/2DItems/Armours/BodyArmours/BodyStrDex3AUnique.png",

  // Helmets
  "Devoto's Devotion": "Art/2DItems/Armours/Helmets/HelmetDex9Unique.png",
  "Crown of the Inward Eye": "Art/2DItems/Armours/Helmets/HelmetStrDexInt3Unique.png",
  "Goldrim": "Art/2DItems/Armours/Helmets/HelmetDex2Unique.png",
  "Mask of the Tribunal": "Art/2DItems/Armours/Helmets/HelmetStrInt7Unique.png",

  // Boots
  "Replica Dreamfeather": "Art/2DItems/Armours/Boots/BootsStr5Unique.png",
  "Kaom's Roots": "Art/2DItems/Armours/Boots/BootsStr3BUnique.png",

  // Gloves
  "Hands of the High Templar": "Art/2DItems/Armours/Gloves/GlovesStrInt4Unique.png",

  // Belts
  "Mageblood": "Art/2DItems/Belts/MagebloadUnique.png",
  "Headhunter": "Art/2DItems/Belts/HeadhunterUnique.png",
  "Darkness Enthroned": "Art/2DItems/Belts/AbyssalBeltUnique.png",

  // Amulets
  "Ashes of the Stars": "Art/2DItems/Amulets/AshesOfTheStarsUnique.png",
  "Aul's Uprising": "Art/2DItems/Amulets/AulUnique.png",
  "Badge of the Brotherhood": "Art/2DItems/Amulets/BadgeBrotherhoodUnique.png",

  // Rings
  "Polaric Devastation": "Art/2DItems/Rings/PolaricDevastationUnique.png",
  "Pyre": "Art/2DItems/Rings/PyreUnique.png",

  // Flasks
  "Bottled Faith": "Art/2DItems/Flasks/SulphurFlaskUnique.png",
  "Dying Sun": "Art/2DItems/Flasks/RubyFlaskUnique.png",
  "Taste of Hate": "Art/2DItems/Flasks/SapphireFlaskUnique.png",

  // Jewels
  "Watcher's Eye": "Art/2DItems/Jewels/WatchersEyeUnique.png",
  "Forbidden Flame": "Art/2DItems/Jewels/ForbiddenFlameUnique.png",
  "Forbidden Flesh": "Art/2DItems/Jewels/ForbiddenFleshUnique.png",
  "Lethal Pride": "Art/2DItems/Jewels/LethalPrideUnique.png",
  "Elegant Hubris": "Art/2DItems/Jewels/ElegantHubrisUnique.png",
  "Militant Faith": "Art/2DItems/Jewels/MilitantFaithUnique.png",
  "Brutal Restraint": "Art/2DItems/Jewels/BrutalRestraintUnique.png",
  "Glorious Vanity": "Art/2DItems/Jewels/GloriousVanityUnique.png",
};

const BASE_TYPE_IMAGES = {
  // Helmets
  "Royal Burgonet": "Art/2DItems/Armours/Helmets/HelmetStr7.png",
  "Eternal Burgonet": "Art/2DItems/Armours/Helmets/HelmetStr8.png",
  "Lion Pelt": "Art/2DItems/Armours/Helmets/HelmetDex8.png",
  "Hubris Circlet": "Art/2DItems/Armours/Helmets/HelmetInt8.png",
  "Bone Helmet": "Art/2DItems/Armours/Helmets/BoneHelmet.png",

  // Body
  "Astral Plate": "Art/2DItems/Armours/BodyArmours/BodyStr4.png",
  "Glorious Plate": "Art/2DItems/Armours/BodyArmours/BodyStr5.png",
  "Vaal Regalia": "Art/2DItems/Armours/BodyArmours/BodyInt4.png",
  "Zodiac Leather": "Art/2DItems/Armours/BodyArmours/BodyDex4.png",
  "Sadist Garb": "Art/2DItems/Armours/BodyArmours/BodyStrDexInt1.png",

  // Gloves
  "Titan Gauntlets": "Art/2DItems/Armours/Gloves/GlovesStr3.png",
  "Slink Gloves": "Art/2DItems/Armours/Gloves/GlovesDex3.png",
  "Sorcerer Gloves": "Art/2DItems/Armours/Gloves/GlovesInt3.png",
  "Fingerless Silk Gloves": "Art/2DItems/Armours/Gloves/FingerlessSilkGloves.png",
  "Gripped Gloves": "Art/2DItems/Armours/Gloves/GrippedGloves.png",

  // Boots
  "Titan Greaves": "Art/2DItems/Armours/Boots/BootsStr3.png",
  "Slink Boots": "Art/2DItems/Armours/Boots/BootsDex3.png",
  "Sorcerer Boots": "Art/2DItems/Armours/Boots/BootsInt3.png",
  "Two-Toned Boots": "Art/2DItems/Armours/Boots/TwoTonedBoots.png",

  // Shields
  "Mosaic Kite Shield": "Art/2DItems/Armours/Shields/ShieldStrDex6.png",
  "Titanium Spirit Shield": "Art/2DItems/Armours/Shields/ShieldInt8.png",
  "Pinnacle Tower Shield": "Art/2DItems/Armours/Shields/ShieldStr8.png",

  // Rings
  "Ruby Ring": "Art/2DItems/Rings/RubyRing.png",
  "Opal Ring": "Art/2DItems/Rings/OpalRing.png",
  "Vermillion Ring": "Art/2DItems/Rings/VermillionRing.png",
  "Diamond Ring": "Art/2DItems/Rings/DiamondRing.png",
  "Amethyst Ring": "Art/2DItems/Rings/AmethystRing.png",
  "Sapphire Ring": "Art/2DItems/Rings/SapphireRing.png",
  "Topaz Ring": "Art/2DItems/Rings/TopazRing.png",
  "Prismatic Ring": "Art/2DItems/Rings/PrismaticRing.png",

  // Amulets
  "Turquoise Amulet": "Art/2DItems/Amulets/TurquoiseAmulet.png",
  "Onyx Amulet": "Art/2DItems/Amulets/OnyxAmulet.png",
  "Marble Amulet": "Art/2DItems/Amulets/MarbleAmulet.png",
  "Citrine Amulet": "Art/2DItems/Amulets/CitrineAmulet.png",
  "Agate Amulet": "Art/2DItems/Amulets/AgateAmulet.png",

  // Belts
  "Crystal Belt": "Art/2DItems/Belts/CrystalBelt.png",
  "Stygian Vise": "Art/2DItems/Belts/StygianVise.png",
  "Leather Belt": "Art/2DItems/Belts/LeatherBelt.png",
  "Heavy Belt": "Art/2DItems/Belts/HeavyBelt.png",
  "Rustic Sash": "Art/2DItems/Belts/RusticSash.png",
};

const GEM_IMAGES = {
  RighteousFire: "Art/2DItems/Gems/RighteousFire.png",
  FireTrap: "Art/2DItems/Gems/FireTrap.png",
  Determination: "Art/2DItems/Gems/Determination.png",
  Grace: "Art/2DItems/Gems/Grace.png",
  Vitality: "Art/2DItems/Gems/Vitality.png",
  Malevolence: "Art/2DItems/Gems/Malevolence.png",
  PurityOfFire: "Art/2DItems/Gems/PurityOfFire.png",
  PurityOfElements: "Art/2DItems/Gems/PurityOfElements.png",
  ShieldCharge: "Art/2DItems/Gems/ShieldCharge.png",
  FlameDash: "Art/2DItems/Gems/FlameDash.png",
  MoltenShell: "Art/2DItems/Gems/MoltenShell.png",
  Flammability: "Art/2DItems/Gems/Flammability.png",
  SupportBurningDamage: "Art/2DItems/Gems/Support/BurningDamage.png",
  SupportElementalFocus: "Art/2DItems/Gems/Support/ElementalFocus.png",
  SupportLifetap: "Art/2DItems/Gems/Support/Lifetap.png",
  SupportSwiftAffliction: "Art/2DItems/Gems/Support/SwiftAffliction.png",
  SupportEfficacy: "Art/2DItems/Gems/Support/Efficacy.png",
  SupportConcentratedEffect: "Art/2DItems/Gems/Support/ConcentratedEffect.png",
  SupportEnlighten: "Art/2DItems/Gems/Support/Enlighten.png",
  SupportFasterAttacks: "Art/2DItems/Gems/Support/FasterAttacks.png",
  SupportCastWhenDamage: "Art/2DItems/Gems/Support/CastWhenDamageTaken.png",
};

const SKILL_ICONS = {
  RighteousFire: "Art/2DArt/SkillIcons/RighteousFire.png",
  FireTrap: "Art/2DArt/SkillIcons/FireTrap.png",
  Determination: "Art/2DArt/SkillIcons/Determination.png",
  MoltenShell: "Art/2DArt/SkillIcons/MoltenShell.png",
  ShieldCharge: "Art/2DArt/SkillIcons/ShieldCharge.png",
  FlameDash: "Art/2DArt/SkillIcons/FlameDash.png",
  Flammability: "Art/2DArt/SkillIcons/Flammability.png",
};

const CURRENCY_IMAGES = {
  "Divine Orb": "Art/2DItems/Currency/CurrencyModValues.png",
  "Chaos Orb": "Art/2DItems/Currency/CurrencyRerollRare.png",
  "Exalted Orb": "Art/2DItems/Currency/CurrencyAddModToRare.png",
  "Orb of Alchemy": "Art/2DItems/Currency/CurrencyUpgradeToRare.png",
  "Vaal Orb": "Art/2DItems/Currency/CurrencyVaal.png",
  "Orb of Alteration": "Art/2DItems/Currency/CurrencyRerollMagic.png",
  "Jeweller's Orb": "Art/2DItems/Currency/CurrencyRerollSocketNumbers.png",
  "Orb of Fusing": "Art/2DItems/Currency/CurrencyRerollSocketLinks.png",
  "Chromatic Orb": "Art/2DItems/Currency/CurrencyRerollSocketColours.png",
  "Orb of Scouring": "Art/2DItems/Currency/CurrencyConvertToNormal.png",
  "Blessed Orb": "Art/2DItems/Currency/CurrencyImplicitMod.png",
  "Regal Orb": "Art/2DItems/Currency/CurrencyUpgradeMagicToRare.png",
  "Orb of Regret": "Art/2DItems/Currency/CurrencyPassiveRefund.png",
  "Mirror of Kalandra": "Art/2DItems/Currency/CurrencyDuplicate.png",
  "Awakener's Orb": "Art/2DItems/Currency/TransferOrb.png",
  "Maven's Orb": "Art/2DItems/Currency/MavensOrb.png",
};

const FLASK_BASE_IMAGES = {
  "Divine Life Flask": "Art/2DItems/Flasks/LifeFlask5.png",
  "Eternal Life Flask": "Art/2DItems/Flasks/LifeFlask6.png",
  "Ruby Flask": "Art/2DItems/Flasks/FlaskRuby.png",
  "Sapphire Flask": "Art/2DItems/Flasks/FlaskSapphire.png",
  "Topaz Flask": "Art/2DItems/Flasks/FlaskTopaz.png",
  "Granite Flask": "Art/2DItems/Flasks/FlaskGranite.png",
  "Basalt Flask": "Art/2DItems/Flasks/FlaskBasalt.png",
  "Jade Flask": "Art/2DItems/Flasks/FlaskJade.png",
  "Quicksilver Flask": "Art/2DItems/Flasks/FlaskQuicksilver.png",
  "Sulphur Flask": "Art/2DItems/Flasks/FlaskSulphur.png",
  "Silver Flask": "Art/2DItems/Flasks/FlaskSilver.png",
  "Amethyst Flask": "Art/2DItems/Flasks/FlaskAmethyst.png",
  "Diamond Flask": "Art/2DItems/Flasks/FlaskDiamond.png",
  "Quartz Flask": "Art/2DItems/Flasks/FlaskQuartz.png",
};

const UNIQUE_FLASK_IMAGES = {
  "Bottled Faith": "Art/2DItems/Flasks/SulphurFlaskUnique.png",
  "Dying Sun": "Art/2DItems/Flasks/RubyFlaskUnique.png",
  "Taste of Hate": "Art/2DItems/Flasks/SapphireFlaskUnique.png",
  "Sin's Rebirth": "Art/2DItems/Flasks/FlaskSinRebirth.png",
  "The Wise Oak": "Art/2DItems/Flasks/FlaskWiseOak.png",
};

module.exports = ItemImageResolver;
