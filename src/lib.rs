//! Bibliothèque du presse-papiers : le binaire Dioxus ne fait que la démarrer.
//!
//! Cette séparation permet aux tests d'intégration d'exercer le cœur du produit
//! sans ouvrir de fenêtre, sans raccourci global et sans session Wayland.
pub mod application;
pub mod clipboard;
pub mod domain;
pub mod platform;
pub mod storage;
pub mod ui;
