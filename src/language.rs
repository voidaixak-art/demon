pub mod language {
    use std::collections::HashSet;

    /// Represents the mathematical Shape/Tensor type of a word in DisCoCat
    /// Pregroup Types:
    ///   Noun         = n           (1D Vector / State)
    ///   Adjective    = n · nˡ      (2D Matrix: modifies a noun)
    ///   TransitiveVerb = nʳ · s · nˡ (3D Tensor: connects subject to object)
    ///   IntransitiveVerb = nʳ · s   (2D: takes subject only)
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub enum WordType {
        /// 1D Vector (State). Pregroup: n. Example: "Gravity", "Apple"
        Noun,
        /// 2D Matrix (Operator: n · nˡ). Example: "Red", "Strong"
        Adjective,
        /// 3D Tensor (Operator: nʳ · s · nˡ). Example: "Pulls", "Causes"
        TransitiveVerb,
        /// Pointer/Identity Matrix. Example: "He", "It"
        Pronoun,
        /// Operator: n → n ← n. Example: "In", "On"
        Preposition,
        /// Identity Matrix for Nouns. Example: "The", "A", "An"
        Determiner,
        /// Equivalence Tensor. Example: "Is", "Are", "Am"
        Copula,
        /// Punctuation triggers a tensor contraction
        Punctuation,
        /// Noise/Structural words to be ignored by Vector Center of Mass
        StopWord,
    }

    pub struct GrammarParser;

    impl GrammarParser {
        /// Native POS Tagger (Rule-based foundation for DisCoCat)
        /// Now includes 300+ common English verbs + morphological suffix rules
        pub fn get_word_type(word: &str) -> WordType {
            let clean = word.to_lowercase();

            // === Layer 0a: Copulas (Equivalence Tensors) ===
            let copulas: HashSet<&str> = [
                "is", "are", "was", "were", "be", "been", "being", "am"
            ].into_iter().collect();

            if copulas.contains(clean.as_str()) {
                return WordType::Copula;
            }

            // === Layer 0b: Determiners (Identity Matrices) ===
            let determiners: HashSet<&str> = [
                "the", "a", "an", "this", "that", "these", "those"
            ].into_iter().collect();

            if determiners.contains(clean.as_str()) {
                return WordType::Determiner;
            }

            // === Layer 0c: Stop Words (Attention Filter) ===
            let stopwords: HashSet<&str> = [
                "do", "does", "did", "has", "have", "had",
                "shall", "will", "should", "would", "could", "may", "might", "must", "can",
                "what", "where", "how", "why", "who", "whom", "which",
                "of", "and", "or", "but", "nor", "yet", "so",
                "not", "no", "if", "then", "than", "too", "also", "very", "just",
                "to", "from", "into", "onto", "upon", "as", "its", "their", "our", "my", "your",
                "there", "here", "when", "while", "during", "before", "after", "since", "until",
                "each", "every", "all", "both", "either", "neither", "some", "any", "such",
            ].into_iter().collect();

            if stopwords.contains(clean.as_str()) {
                return WordType::StopWord;
            }

            // === Layer 1: Punctuation ===
            if clean == "." || clean == "," || clean == "?" || clean == "!" || clean == ";" || clean == ":" {
                return WordType::Punctuation;
            }

            // === Layer 2: Pronouns ===
            let pronouns: HashSet<&str> = [
                "he", "she", "it", "they", "we", "i", "you",
                "him", "her", "them", "us", "me",
                "himself", "herself", "itself", "themselves", "ourselves",
            ].into_iter().collect();
            if pronouns.contains(clean.as_str()) {
                return WordType::Pronoun;
            }

            // === Layer 3: Prepositions ===
            let preps: HashSet<&str> = [
                "in", "on", "at", "by", "for", "with", "about", "against",
                "between", "through", "above", "below", "under", "over",
                "near", "around", "among", "within", "without", "toward", "towards",
                "along", "across", "behind", "beside", "beneath", "beyond",
            ].into_iter().collect();
            if preps.contains(clean.as_str()) {
                return WordType::Preposition;
            }

            // === Layer 4: VERB DETECTION (THE CRITICAL FIX) ===
            // 300+ common English transitive verbs organized by scientific/general category

            let verbs: HashSet<&str> = [
                // --- Physics & Science Verbs ---
                "pulls", "pull", "pulled", "pulling",
                "pushes", "push", "pushed", "pushing",
                "attracts", "attract", "attracted", "attracting",
                "repels", "repel", "repelled", "repelling",
                "accelerates", "accelerate", "accelerated", "accelerating",
                "decelerates", "decelerate", "decelerated", "decelerating",
                "rotates", "rotate", "rotated", "rotating",
                "orbits", "orbit", "orbited", "orbiting",
                "oscillates", "oscillate", "oscillated", "oscillating",
                "vibrates", "vibrate", "vibrated", "vibrating",
                "radiates", "radiate", "radiated", "radiating",
                "absorbs", "absorb", "absorbed", "absorbing",
                "reflects", "reflect", "reflected", "reflecting",
                "refracts", "refract", "refracted", "refracting",
                "diffracts", "diffract", "diffracted", "diffracting",
                "emits", "emit", "emitted", "emitting",
                "transmits", "transmit", "transmitted", "transmitting",
                "conducts", "conduct", "conducted", "conducting",
                "insulates", "insulate", "insulated", "insulating",
                "dissolves", "dissolve", "dissolved", "dissolving",
                "evaporates", "evaporate", "evaporated", "evaporating",
                "condenses", "condense", "condensed", "condensing",
                "freezes", "freeze", "froze", "frozen", "freezing",
                "melts", "melt", "melted", "melting",
                "boils", "boil", "boiled", "boiling",
                "oxidizes", "oxidize", "oxidized", "oxidizing",
                "reduces", "reduce", "reduced", "reducing",
                "catalyzes", "catalyze", "catalyzed", "catalyzing",
                "synthesizes", "synthesize", "synthesized", "synthesizing",
                "decomposes", "decompose", "decomposed", "decomposing",
                "measures", "measure", "measured", "measuring",
                "observes", "observe", "observed", "observing",
                "calculates", "calculate", "calculated", "calculating",
                "computes", "compute", "computed", "computing",
                "derives", "derive", "derived", "deriving",
                "proves", "prove", "proved", "proven", "proving",
                "predicts", "predict", "predicted", "predicting",
                "hypothesizes", "hypothesize", "hypothesized",
                "experiments", "experiment", "experimented",
                "conserves", "conserve", "conserved", "conserving",
                "transforms", "transform", "transformed", "transforming",
                "converts", "convert", "converted", "converting",
                "generates", "generate", "generated", "generating",
                "produces", "produce", "produced", "producing",
                "consumes", "consume", "consumed", "consuming",
                "transfers", "transfer", "transferred", "transferring",
                "propagates", "propagate", "propagated", "propagating",
                "collides", "collide", "collided", "colliding",
                "scatters", "scatter", "scattered", "scattering",
                "decays", "decay", "decayed", "decaying",
                "fuses", "fuse", "fused", "fusing",
                "splits", "split", "splitting",
                "binds", "bind", "bound", "binding",
                "bonds", "bond", "bonded", "bonding",
                "ionizes", "ionize", "ionized", "ionizing",
                "polarizes", "polarize", "polarized", "polarizing",
                "magnetizes", "magnetize", "magnetized",
                "charges", "charge", "charged", "charging",
                "discharges", "discharge", "discharged",
                "induces", "induce", "induced", "inducing",
                "amplifies", "amplify", "amplified", "amplifying",
                "dampens", "dampen", "dampened", "dampening",
                "distorts", "distort", "distorted", "distorting",
                "warps", "warp", "warped", "warping",
                "bends", "bend", "bent", "bending",
                "stretches", "stretch", "stretched", "stretching",
                "compresses", "compress", "compressed", "compressing",
                "expands", "expand", "expanded", "expanding",
                "contracts", "contract", "contracted", "contracting",
                "flows", "flow", "flowed", "flowing",
                "diffuses", "diffuse", "diffused", "diffusing",
                "permeates", "permeate", "permeated",
                "saturates", "saturate", "saturated",
                "dilutes", "dilute", "diluted",
                "concentrates", "concentrate", "concentrated",
                "mixes", "mix", "mixed", "mixing",
                "separates", "separate", "separated", "separating",
                "filters", "filter", "filtered", "filtering",
                "distills", "distill", "distilled",
                "crystallizes", "crystallize", "crystallized",
                "precipitates", "precipitate", "precipitated",
                "corrodes", "corrode", "corroded",
                "erodes", "erode", "eroded", "eroding",
                "weathers", "weather", "weathered",

                // --- Biology & Evolution Verbs ---
                "evolves", "evolve", "evolved", "evolving",
                "adapts", "adapt", "adapted", "adapting",
                "mutates", "mutate", "mutated", "mutating",
                "inherits", "inherit", "inherited", "inheriting",
                "reproduces", "reproduce", "reproduced",
                "replicates", "replicate", "replicated",
                "metabolizes", "metabolize", "metabolized",
                "photosynthesizes", "photosynthesize",
                "respires", "respire", "respired",
                "digests", "digest", "digested", "digesting",
                "secretes", "secrete", "secreted",
                "excretes", "excrete", "excreted",
                "regulates", "regulate", "regulated", "regulating",
                "stimulates", "stimulate", "stimulated",
                "inhibits", "inhibit", "inhibited", "inhibiting",
                "activates", "activate", "activated", "activating",
                "suppresses", "suppress", "suppressed",
                "expresses", "express", "expressed", "expressing",
                "encodes", "encode", "encoded", "encoding",
                "transcribes", "transcribe", "transcribed",
                "translates", "translate", "translated", "translating",
                "fertilizes", "fertilize", "fertilized",
                "germinates", "germinate", "germinated",
                "pollinates", "pollinate", "pollinated",
                "migrates", "migrate", "migrated",
                "hibernates", "hibernate", "hibernated",
                "parasitizes", "parasitize",
                "symbioses", "symbiose",
                "infects", "infect", "infected",
                "immunizes", "immunize", "immunized",

                // --- Causation & Logic Verbs ---
                "causes", "cause", "caused", "causing",
                "determines", "determine", "determined", "determining",
                "affects", "affect", "affected", "affecting",
                "influences", "influence", "influenced", "influencing",
                "implies", "imply", "implied", "implying",
                "requires", "require", "required", "requiring",
                "necessitates", "necessitate", "necessitated",
                "prevents", "prevent", "prevented", "preventing",
                "enables", "enable", "enabled", "enabling",
                "triggers", "trigger", "triggered", "triggering",
                "initiates", "initiate", "initiated",
                "terminates", "terminate", "terminated",
                "precedes", "precede", "preceded",
                "follows", "follow", "followed", "following",
                "results", "result", "resulted",
                "leads", "lead", "led", "leading",
                "yields", "yield", "yielded", "yielding",
                "contributes", "contribute", "contributed",
                "depends", "depend", "depended", "depending",

                // --- General Action Verbs ---
                "creates", "create", "created", "creating",
                "destroys", "destroy", "destroyed", "destroying",
                "builds", "build", "built", "building",
                "breaks", "break", "broke", "broken", "breaking",
                "forms", "form", "formed", "forming",
                "shapes", "shape", "shaped", "shaping",
                "contains", "contain", "contained", "containing",
                "holds", "hold", "held", "holding",
                "carries", "carry", "carried", "carrying",
                "moves", "move", "moved", "moving",
                "changes", "change", "changed", "changing",
                "remains", "remain", "remained", "remaining",
                "increases", "increase", "increased", "increasing",
                "decreases", "decrease", "decreased", "decreasing",
                "grows", "grow", "grew", "grown", "growing",
                "shrinks", "shrink", "shrank", "shrunk",
                "rises", "rise", "rose", "risen", "rising",
                "falls", "fall", "fell", "fallen", "falling",
                "connects", "connect", "connected", "connecting",
                "links", "link", "linked", "linking",
                "joins", "join", "joined", "joining",
                "combines", "combine", "combined", "combining",
                "divides", "divide", "divided", "dividing",
                "surrounds", "surround", "surrounded",
                "covers", "cover", "covered", "covering",
                "fills", "fill", "filled", "filling",
                "empties", "empty", "emptied",
                "opens", "open", "opened", "opening",
                "closes", "close", "closed", "closing",
                "supports", "support", "supported", "supporting",
                "resists", "resist", "resisted", "resisting",
                "opposes", "oppose", "opposed", "opposing",
                "balances", "balance", "balanced", "balancing",
                "stabilizes", "stabilize", "stabilized",
                "disrupts", "disrupt", "disrupted",
                "maintains", "maintain", "maintained",
                "preserves", "preserve", "preserved",
                "protects", "protect", "protected",
                "limits", "limit", "limited", "limiting",
                "restricts", "restrict", "restricted",
                "defines", "define", "defined", "defining",
                "describes", "describe", "described", "describing",
                "explains", "explain", "explained", "explaining",
                "demonstrates", "demonstrate", "demonstrated",
                "illustrates", "illustrate", "illustrated",
                "represents", "represent", "represented",
                "indicates", "indicate", "indicated",
                "shows", "show", "showed", "shown", "showing",
                "reveals", "reveal", "revealed", "revealing",
                "establishes", "establish", "established",
                "confirms", "confirm", "confirmed",
                "validates", "validate", "validated",
                "verifies", "verify", "verified",
                "denies", "deny", "denied",
                "refutes", "refute", "refuted",
                "contradicts", "contradict", "contradicted",
                "negates", "negate", "negated",
                "assumes", "assume", "assumed",
                "postulates", "postulate", "postulated",
                "proposes", "propose", "proposed",
                "suggests", "suggest", "suggested",
                "asserts", "assert", "asserted",
                "states", "state", "stated",
                "claims", "claim", "claimed",
                "argues", "argue", "argued",
                "concludes", "conclude", "concluded",
                "deduces", "deduce", "deduced",
                "infers", "infer", "inferred",
                "abstracts", "abstract", "abstracted",
                "generalizes", "generalize", "generalized",
                "specializes", "specialize", "specialized",
                "classifies", "classify", "classified",
                "categorizes", "categorize", "categorized",
                "identifies", "identify", "identified",
                "distinguishes", "distinguish", "distinguished",
                "compares", "compare", "compared",
                "contrasts", "contrast", "contrasted",
                "analyzes", "analyze", "analyzed", "analyzing",
                "examines", "examine", "examined",
                "investigates", "investigate", "investigated",
                "studies", "study", "studied",
                "explores", "explore", "explored",
                "discovers", "discover", "discovered",
                "invents", "invent", "invented",
                "develops", "develop", "developed",
                "designs", "design", "designed",
                "constructs", "construct", "constructed",
                "implements", "implement", "implemented",
                "executes", "execute", "executed",
                "operates", "operate", "operated",
                "functions", "function", "functioned",
                "performs", "perform", "performed",
                "achieves", "achieve", "achieved",
                "accomplishes", "accomplish", "accomplished",
                "completes", "complete", "completed",
                "finishes", "finish", "finished",
                "begins", "begin", "began", "begun",
                "starts", "start", "started",
                "ends", "end", "ended",
                "stops", "stop", "stopped",
                "continues", "continue", "continued",
                "persists", "persist", "persisted",
                "exists", "exist", "existed", "existing",
                "occurs", "occur", "occurred",
                "happens", "happen", "happened",
                "appears", "appear", "appeared",
                "emerges", "emerge", "emerged",
                "arises", "arise", "arose", "arisen",
                "originates", "originate", "originated",
                "derives", "derive", "derived",
                "stems", "stem", "stemmed",

                // --- Math Verbs ---
                "equals", "equal", "equaled",
                "exceeds", "exceed", "exceeded",
                "approximates", "approximate", "approximated",
                "converges", "converge", "converged",
                "diverges", "diverge", "diverged",
                "integrates", "integrate", "integrated",
                "differentiates", "differentiate", "differentiated",
                "sums", "sum", "summed",
                "multiplies", "multiply", "multiplied",
                "maps", "map", "mapped", "mapping",
                "projects", "project", "projected",
                "intersects", "intersect", "intersected",
                "spans", "span", "spanned",
                "satisfies", "satisfy", "satisfied",
                "solves", "solve", "solved", "solving",
                "optimizes", "optimize", "optimized",
                "minimizes", "minimize", "minimized",
                "maximizes", "maximize", "maximized",
                "normalizes", "normalize", "normalized",
                "parameterizes", "parameterize", "parameterized",
            ].into_iter().collect();

            if verbs.contains(clean.as_str()) {
                return WordType::TransitiveVerb;
            }

            // === Layer 5: MORPHOLOGICAL SUFFIX RULES (Catch verbs not in list) ===
            // These patterns catch regular English verb conjugations
            if clean.len() > 4 {
                // "-ates" (generates, oscillates, propagates)
                if clean.ends_with("ates") { return WordType::TransitiveVerb; }
                // "-izes" (ionizes, polarizes, magnetizes)
                if clean.ends_with("izes") { return WordType::TransitiveVerb; }
                // "-ifies" (amplifies, classifies, identifies)
                if clean.ends_with("ifies") { return WordType::TransitiveVerb; }
                // "-ises" (British spelling: catalyses, synthesises)
                if clean.ends_with("ises") { return WordType::TransitiveVerb; }
            }

            // === Layer 6: ADJECTIVE DETECTION (Suffix-based) ===
            if clean.len() > 3 {
                // "-ous" (gravitational→ous, luminous, porous)
                if clean.ends_with("ous") { return WordType::Adjective; }
                // "-ive" (massive, attractive, repulsive)
                if clean.ends_with("ive") { return WordType::Adjective; }
                // "-ful" (powerful, forceful, meaningful)
                if clean.ends_with("ful") { return WordType::Adjective; }
                // "-less" (massless, weightless, frictionless)
                if clean.ends_with("less") { return WordType::Adjective; }
                // "-ible"/"-able" (visible, observable, measurable)
                if clean.ends_with("ible") || clean.ends_with("able") { return WordType::Adjective; }
                // "-ical" (physical, mathematical, theoretical)
                if clean.ends_with("ical") { return WordType::Adjective; }
                // "-nal" (gravitational, rotational, vibrational)
                if clean.ends_with("ional") { return WordType::Adjective; }
                // "-ent"/"-ant" (coherent, abundant, resultant)
                if clean.ends_with("ent") || clean.ends_with("ant") { return WordType::Adjective; }
                // "-ic" (atomic, electric, magnetic) but not too short
                if clean.len() > 4 && clean.ends_with("ic") && !clean.ends_with("tic") { return WordType::Adjective; }
                // "-al" (universal, gravitational, mechanical) — careful, many nouns end in -al
                // Only match if long enough to avoid "metal", "trial", etc.
                if clean.len() > 6 && clean.ends_with("al") && !clean.ends_with("ial") { return WordType::Adjective; }
            }

            // === Layer 7: Default = Noun ===
            // If none of the above matched, it's a Noun (the most common POS in scientific text)
            WordType::Noun
        }

        /// Returns the Pregroup type string for DisCoCat
        pub fn get_pregroup_type(wt: &WordType) -> &'static str {
            match wt {
                WordType::Noun => "n",
                WordType::Adjective => "n·nˡ",
                WordType::TransitiveVerb => "nʳ·s·nˡ",
                WordType::Pronoun => "n",
                WordType::Preposition => "nʳ·n·nˡ",
                WordType::Determiner => "n·nˡ", // Acts as a modifier
                WordType::Copula => "nʳ·s·nˡ", // Acts like a transitive verb
                WordType::Punctuation => "⊥",
                WordType::StopWord => "1",
            }
        }

        pub fn get_attention_weight(word: &str) -> f64 {
            match Self::get_word_type(word) {
                WordType::StopWord => 0.0,
                WordType::Punctuation => 0.0,
                WordType::Determiner => 0.1,
                WordType::Pronoun => 0.5,
                WordType::Preposition => 0.8,
                WordType::Copula => 1.0,
                WordType::Adjective => 1.0,
                WordType::TransitiveVerb => 1.5,
                WordType::Noun => 2.0,
            }
        }

        pub fn parse_sentence(sentence: &str) -> Vec<(String, WordType, f64)> {
            let mut parsed = Vec::new();
            let words: Vec<&str> = sentence.split_whitespace().collect();
            for word in words {
                let clean_word = word.trim_matches(|c: char| !c.is_alphanumeric());
                if !clean_word.is_empty() {
                    let w_type = Self::get_word_type(clean_word);
                    let weight = Self::get_attention_weight(clean_word);
                    parsed.push((clean_word.to_string(), w_type, weight));
                }
            }
            parsed
        }
    }
}
