WebCore — Détails techniques du compilateur
1. Principes directeurs

Performance & robustesse : implémentation en Rust (compilation native, bonnes libs pour JS/CSS).

UX développeur : diagnostics clairs, erreurs avec position source, build rapide (cache/incrémental).

Always modern : générer JS/CSS conformes à es_latest / CSS Next ; post-traitement via libs Rust (SWC, LightningCSS).

Séparation des responsabilités : Parser → AST → IR (optimisations) → Codegen → Transformers → Bundler.

2. Choix de parsing (parser → AST)
Options (tradeoffs)

pest (PEG)

Syntaxe claire, rapide à itérer pour grammaires mixtes (HTML-like + blocs).

Facile à écrire pour blocs imbriqués (view/style).

− PEG backtracking : attention aux ambiguïtés et aux performances sur certains patterns.

lalrpop (LALR)

Très performant et déterministe.

− Grammaire plus verbeuse ; moins naturelle pour fragments HTML imbriqués.

nom / combine (parser combinators)

Extrêmement flexible, bon pour parseurs incrémentaux.

− Code plus “programmation” que déclaration de grammaire.

Recommandation

MVP : commencer avec pest (rapidité d’itération, lisibilité).

Si besoin d’échelle : viser migration vers lalrpop / générateur LALR si on rencontre des problèmes de performance ou ambiguïtés.

Fonctions essentielles du parser

Retourner des erreurs lisibles (ligne/col, snippet avec caret).

Produire un AST annoté avec positions (start/end).

Supporter les blocs : app, component, state, view, style, logic, Global, layout, page, import.

3. AST vs IR

AST : représentation fidèle du code source (nœuds avec positions).

IR (Intermediate Representation) : structure simplifiée/normalisée pour optimiser/générer du code (ex : dépendances resolved, scoping des styles, tokens thèmes remplacés par variables).

Exemple minimal AST (Rust)
pub enum Node {
    Component { name: String, states: Vec<State>, view: Node, style: Option<Node>, pos: Span },
    Element { tag: String, attrs: Vec<Attr>, children: Vec<Node>, pos: Span },
    Text(String, Span),
    Interpolation(String, Span),
    // ...
}

Exemple IR (concept)

ComponentIR { id, name, state_slots: Vec<StateSlot>, template: HtmlTemplate, scoped_css: CssRules }

IR facilite : hoisting, tree-shaking, code-splitting, SSR/hydration.

4. Résolution des imports, modules et graphes

Construire un module graph : fichiers .webc, imports JS/CSS externes, assets.

Résolution : chemins relatifs, alias (ex: @components/), node_modules-like resolution si nécessaire.

Calculer dépendances pour bundling et HMR.

5. Codegen — stratégies et principes
Cibles

HTML : SSR-ready templates ou static HTML selon mode (static vs SPA).

JS : ES Modules (ESM) modernes par défaut (top-level await, classes privés, optional chaining).

CSS : variables :root, scoped CSS modules / CSS-in-compiled / CSS with generated unique selectors.

Approche

HTML codegen : transformer IR → DOM templates. Pour SSR : output index.html; pour SPA : minimal HTML + hydration script.

JS codegen : générer modules :

state.js (classe/objets réactifs),

componentX.js (render + lifecycle hooks),

main.js (bootstrap/hydrate/router).

CSS codegen : générer out.tmp.css (variables, scope markers), puis transformer via LightningCSS.

Output format recommandé

Modules ESM: import { ... } from "./componentA.js".

Minifier + bundle séparé en dev/prod.

Sourcemaps pour debug.

6. Post-traitement JS/CSS (transformers)
JS

SWC (Rust) : parsing, optimisation, minification, target selection.

Utiliser SWC pour :

transformer le code généré si on veut garantir compatibilité vers cible (optionnel, par défaut es_latest),

minifier (terser-like),

tree-shake si on bundle.

CSS

LightningCSS ou parcel_css : support des features modernes (nesting, container queries), autoprefixing, minification, source maps.

Intégration

Faire ces dépendances optionnelles (features Cargo) pour garder flexibilité.

Exposer config webc.toml : build.target = "es_latest", css.targets = ["last 2 versions"].

7. Bundler et assets pipeline

Minimal bundler interne pour V1 :

Copier public/ → dist/assets/ avec fingerprinting (hash) pour cache-busting.

Émettre dist/index.html, dist/state.js, dist/out.css.

Avancé : support code-splitting (route-based), lazy loading, dynamic imports.

Support des imports d’assets dans .webc (e.g. <img src="./logo.png">) -> inliner small assets, otherwise copy and rewrite paths.

8. Runtime (petit runtime JS généré / embarqué)

Fournir un runtime minimal (few KB) qui gère :

hooks lifecycle (mounted, onMount),

micro-reactivity (simple scheduler),

theme injection & API (theme, app),

facade pour SSO/tracking adaptative.

Two modes:

Zero-runtime SSR (static output).

Runtime-hydration (small runtime to hydrate interactive parts).

9. Hot reload / Dev server

File watcher : notify crate (Rust) pour détecter changements.

HMR strategy (v2) : via websocket to browser; send delta (recompiled component) and apply patch.

Dev server simple : tiny-http or warp/axum for static serving and WS for HMR.

10. Diagnostics & ergonomie des erreurs

Utiliser miette + codespan-reporting pour messages d’erreurs riches avec snippet et caret.

Chaque AST node contient Span (file, line, col) pour mapping.

Erreurs catégorisées : syntaxe, type (state/type mismatch), semantic (duplicate id), import missing.

11. Tests & conformité

Unit tests pour parser, AST → golden files (input .webc → expected AST / IR / generated HTML).

Integration tests : full builds from example projects.

Spec compliance : utiliser test262 pour JS transform output (dans CI) et WPT pour CSS (optionnel, avancé).

Fuzzing du parser (libfuzzer / cargo-fuzz) pour robustness.

12. Sécurité

Sandbox pour raw_js / raw_css : marquer et isoler, limiter accès (pas d’eval non CSFriendly), generate CSP directives automatiquement.

SRI pour scripts externes si configured.

Permissions model dans compiler (déclarer permissions: { camera: false } etc) — générer warnings/manifest.

13. Extensibilité & plugins

Exporter IR plugin hooks : pre-transform, post-transform, codegen hooks.

Plugin API en Rust (proc-macros? dynamic loading?) — ou via a CLI plugin system (npm-like) si on supporte Node tooling.

Plugins utiles : linter, formatter, stylelint integration, design token generator.

14. Architecture du repo (workspace Rust)
/workspace
  /compiler_core    # lib: parser, ast, IR, codegen
  /cli              # binary: webc (uses compiler_core)
  /dev_server       # optional binary for dev mode
  /examples/my-app  # sample app
  /docs
  /tests/integration


Utiliser features Cargo pour optional deps (swc, lightningcss).

15. Priorités de mise en œuvre (Do Now / Soon / Later)

Do Now (V1 MVP)

Parser (pest) + AST minimal.

IR simple (component, view, state, style).

HTML/JS/CSS codegen stubs that create dist/.

CLI webc build.

Good error messages (basic).

Do Soon

Integrate LightningCSS for CSS postprocess.

Integrate SWC for JS minify/transform.

Sourcemaps generation.

Module graph and asset copying.

Later (V2)

HMR / Dev server with websocket.

IR optimisations (tree-shaking, code-splitting).

Full reactivity runtime and hydration.

Plugin architecture, LSP support, IDE integration.

test262 / WPT checks in CI.

16. Exemple de flux concret (implémentation)

webc build CLI → read webc.toml, find entry src/main.webc.

Parser (pest) → AST with spans.

AST → IR normalization (resolve imports, name-mangle, scope CSS).

IR → generate out.tmp.css, state.tmp.js, componentX.js, index.html.

Run lightningcss::transform(out.tmp.css) → out.css (minified).

Run swc::transform(state.tmp.js) → state.js (minified).

Bundle & copy assets → dist/.

Emit sourcemaps & final reports.

17. Observabilité / métriques build

Temps de parse / codegen / transform.

Cache hit rate (for incremental builds).

Size of outputs (gzip + brotli).

Warnings count.

18. Points d’attention & risques

Ambiguïtés de grammaire si on autorise too much free-text inside view → testez early.

Backtracking PEG heavy patterns → watch perf on big projects.

Keeping up with JS/CSS spec : dépendre de libs (SWC/LightningCSS) pour suivre la spec.

Interoperability edge-cases (importing existing libs, React/SSR integration).

19. Prochaines actions concrètes que je peux produire maintenant

générer parser.pest complet pour la grammaire MVP (component/state/view/style) ;

écrire un parser.rs en Rust qui produit AST avec Span et tests unitaires ;

implémenter codegen_html.rs qui transforme AST → dist/index.html pour my-app d’exemple ;

intégrer LightningCSS (stub → real) dans transformers.rs ;

ajouter diagnostics riches (miette) au parser.