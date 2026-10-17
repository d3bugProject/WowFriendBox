# Guide des Ressources Graphiques (Assets)

Ce dossier contient toutes les images et polices de l'interface graphique.
Tu peux remplacer n'importe quel fichier par tes propres créations !

---

## 1. Cadre de Fond & Structure Principale

* **`assets/frame.png`** (ou `background.png`) :
  * **Dimensions totales** : **1000 × 100 px**
  * **Format** : **PNG 32-bit avec transparence (Canal Alpha)**
  * **Zone utile pour les boutons / textes** : **900 × 80 px** (centrée avec des marges de 50px à gauche/droite et 10px en haut/bas).
  * **Astuce de graphiste** : Tu peux créer des coins biseautés, des néons, des transparences fumées, des bordures style HUD sci-fi ou métal brossé WoW. Les zones transparentes laisseront voir ton jeu en arrière-plan !

---

## 2. Polices (`assets/fonts/`)

* **`custom_font.ttf`** : 
  * C'est la police utilisée pour tout le texte de l'interface (titres, logs, fps, boutons).
  * Tu peux télécharger n'importe quelle Google Font (ex: *JetBrains Mono*, *Fira Code*, *Share Tech Mono*, *VT323*, *Roboto Mono*) au format `.ttf` et la renommer en `custom_font.ttf` ici.

---

## 3. Boutons & Icônes (`assets/buttons/`)

Les formats recommandés sont **PNG avec transparence** (ou JPG).

| Fichier | Description | Taille suggérée (Largeur x Hauteur) |
|---|---|---|
| `btn_close.png` | Bouton pour fermer la fenêtre `[X]` | 16x16 px (ou 20x20) |
| `btn_minimize.png` | Bouton pour réduire la fenêtre `[-]` | 16x16 px (ou 20x20) |
| `btn_keyboard_on.png` | État actif du broadcast clavier | 64x24 px (ou 70x24) |
| `btn_keyboard_off.png` | État inactif du broadcast clavier | 64x24 px (ou 70x24) |
| `btn_mouse_on.png` | État actif du broadcast souris | 64x24 px (ou 70x24) |
| `btn_mouse_off.png` | État inactif du broadcast souris | 64x24 px (ou 70x24) |
| `btn_realign.png` | Bouton pour réaligner la grille des fenêtres | 64x24 px (ou 70x24) |
| `btn_f1.png` | Swap vers Slot 1 (Centre) | 28x20 px (ou 32x22) |
| `btn_f2.png` | Swap vers Slot 2 (Haut Gauche) | 28x20 px (ou 32x22) |
| `btn_f3.png` | Swap vers Slot 3 (Bas Gauche) | 28x20 px (ou 32x22) |
| `btn_f4.png` | Swap vers Slot 4 (Haut Droite) | 28x20 px (ou 32x22) |
| `btn_f5.png` | Swap vers Slot 5 (Bas Droite) | 28x20 px (ou 32x22) |

---

> **Note :** Si une image est absente ou si tu en supprimes une, l'application affichera un bouton de secours textuel propre sans planter.
