# 🌐 WebCore Language

## 🚀 Vision
WebCore (`.webc`) est un langage **ultra-déclaratif** conçu pour **unifier HTML, CSS et JS** dans une seule syntaxe simple.  
L’objectif est de simplifier radicalement le développement web : plus besoin de jongler entre trois langages, tout passe par WebCore.

---

## 🎯 Objectifs

- **Un seul langage** pour le web → `.webc`
- **Compilateur Rust** robuste et rapide
- **Interopérabilité** avec CSS/JS externes mais toujours via des déclarations déclaratives
- **Build moderne** → HTML/CSS/JS minifiés, optimisés, respectant les dernières normes (ECMAScript, CSS WG)
- **Écosystème complet** (CLI, dev server, docs, IDE support)

---

## 🛠️ Roadmap

### Phase 1 - MVP (2-3 mois)
- Parser Rust avec grammaire EBNF
- Compilateur basique → HTML/CSS/JS
- CLI minimal : `webc build`
- Un composant qui compile et fonctionne

### Phase 2 - Fonctionnalités (3-4 mois)
- State management réactif
- Routing et layout
- Validation déclarative
- Hot reload en développement

### Phase 3 - Écosystème (6+ mois)
- Dev tools et IDE support
- Migration tools pour React/Vue
- Documentation complète
- Community active

---

## 📂 Structure du projet

### Côté compilateur
```
webcore-compiler/
│── Cargo.toml
│── src/
│    ├── main.rs
│    ├── parser.rs
│    ├── ast.rs
│    ├── codegen_html.rs
│    ├── codegen_css.rs
│    ├── codegen_js.rs
```

### Côté projet utilisateur
```
my-app/
│── webc.toml
│── src/
│    ├── main.webc
│    ├── components/
│    ├── layouts/
│    └── pages/
│── dist/
│── public/
```

---

## 🔑 Points clés
- **Ultra-déclaratif** → écrire un site complet avec seulement `.webc`
- **Interopérabilité** → possibilité d’inclure JS/CSS externes mais sans casser le modèle déclaratif
- **Modernité** → code généré toujours conforme aux dernières normes ECMAScript et CSS
- **Performances** → Rust pour la vitesse et la fiabilité
- **Accessibilité** → un langage simple, lisible et intuitif

---

## ✅ Exemple simple

### Code WebCore
```webc
layout default {
  page "index" {
    h1 "Hello, WebCore!"
    p "This is a demo page."
  }
}
```

### Généré (HTML simplifié)
```html
<!DOCTYPE html>
<html>
  <body>
    <h1>Hello, WebCore!</h1>
    <p>This is a demo page.</p>
  </body>
</html>
```

---

## 📌 Pourquoi WebCore ?
Parce que le web a besoin d’un **nouveau langage unifié**, plus simple, plus rapide, et qui libère les développeurs de la fragmentation HTML/CSS/JS.

---

## 🚀 Impact attendu

### Pour les développeurs
- Productivité : 3x plus rapide
- Simplicité : un seul langage à apprendre
- Performance : apps 10x plus rapides
- Maintenance : code lisible et maintenable

### Pour l’industrie
- Standardisation : un langage pour tout le web
- Innovation : focus sur la logique, pas sur la technique
- Accessibilité : apprentissage facilité
- Évolution : base solide pour le futur

---
