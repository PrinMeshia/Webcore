![logo_webcore](https://github.com/PrinMeshia/Webcore/blob/main/Webcore.png)


Langage WebCore (.webc) – un langage natif pour le web, fusionnant structure, style et logique dans un seul langage déclaratif et réactif.


---

## 🎯 Objectif

WebCore vise à :

- Simplifier le développement web en combinant HTML, CSS et JS dans un seul langage.

- Offrir une réactivité native (state + data binding).

- Fournir un scoping automatique du CSS et des permissions natives.

- Compiler vers HTML/CSS/JS/WASM standard pour compatibilité maximale.

- Fournir un CLI simple pour créer, développer, tester et déployer des projets WebCore.



---

## 🛠 Fonctionnalités principales

- Composants déclaratifs (component, state, view, style, logic).

- Routing natif et layout global.

- Thèmes globaux et variables réutilisables.

- CSS scoped automatiquement.

- Support WebAssembly pour logique lourde.

- CLI complet (create, dev, build, test, deploy).


## 📈 Roadmap

### Phase 1 – MVP

[x] Syntaxe minimale définie (grammaire EBNF).

[x] Prototype de parseur → AST.

[x] Compilation d’un composant simple .webc → HTML/CSS/JS.


### Phase 2 – CLI et projet

[ ] CLI webc (create, dev, build).

[ ] Routing et layout global.

[ ] Gestion du state réactif.


### Phase 3 – Avancé

[ ] Thèmes et variables globales.

[ ] Scoped CSS et transitions.

[ ] Permissions natives (camera, storage, net).

[ ] Support WebAssembly pour logique lourde.


### Phase 4 – Tests & déploiement

[ ] Tests unitaires et d’intégration.

[ ] Déploiement simplifié (webc deploy).

[ ] Documentation complète pour développeurs.


---

## 🔧 Outils envisagés

- Parseur : ANTLR, PEG.js ou lark (Python).

- Compilateur : Node.js ou Python pour MVP.

- CLI : Node.js (npm/yarn).

- Versionning : Git + GitHub.



---

📝 Contribution

1. Fork → créer une branche → push → pull request.


2. Respecter le standard de code et ajouter tests.


3. Proposer des améliorations et nouvelles fonctionnalités via issues.




---

## 💡 Idées futures

- Génération automatique de composants via IA.

- Dark mode et accessibilité natifs.

- Optimisations pour PWA, AR/VR et WebAssembly.


