# Pyramide de tests

La base du projet doit avoir beaucoup de tests rapides et peu de tests qui
dépendent de l'environnement graphique. Exécuter l'ensemble avec :

```bash
cargo test
```

## 1. Tests unitaires

Ils sont au plus près du code dans `src/`. Ils ne créent aucune fenêtre et ne
touchent pas au presse-papiers réel. Ils couvrent les règles de données : hash,
parsing d'une entrée SQLite, contenu vide, etc.

## 2. Tests d'intégration

Ils sont dans `tests/sqlite_repository.rs` et utilisent une vraie base SQLite
temporaire. Ils valident migrations, écriture, lecture et politique de rétention
des 200 éléments. Ils ne lisent jamais la base personnelle de l'utilisateur.

## 3. Tests système locaux

`tests/system_text_history.rs` vérifie le chemin complet service → SQLite →
historique, sans fenêtre Dioxus et sans session graphique. C'est le test à
étendre lors de l'ajout de la déduplication et des images.

## 4. Tests d'acceptation

Oui, ils sont utiles, mais en nombre réduit. Le test automatisé actuel vérifie
le besoin utilisateur essentiel : une copie reste présente après redémarrage.

Pour les fonctionnalités dépendantes de Wayland, compléter avec cette recette
manuelle sous Ubuntu :

1. Démarrer `dx serve` sous Wayland.
2. Copier un texte dans une autre application.
3. Vérifier qu'une seule entrée apparaît dans l'historique.
4. Fermer puis redémarrer l'application : l'entrée est encore visible.
5. Autoriser le raccourci dans le portail XDG, l'utiliser, puis vérifier le
   repli fenêtre/tray si l'environnement refuse le portail.

Exécuter la même recette dans une VM X11 lorsque le backend `global-hotkey` est
implémenté. Les images viendront avec leur propre scénario : copie, vignette,
redémarrage et nettoyage du fichier associé.
