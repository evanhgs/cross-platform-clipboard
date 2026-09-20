//! Raccourci global Wayland via le portail XDG (`ashpd`).
//!
//! TODO : activer la feature `global_shortcuts` de `ashpd`, créer une session,
//! demander à l'utilisateur l'association du raccourci, puis écouter
//! `receive_activated`. Le portail peut être indisponible selon le compositeur :
//! l'application doit alors rester utilisable depuis sa fenêtre ou son tray.
//!
//! Ne pas écouter le clavier brut sous Wayland : le protocole l'interdit pour
//! préserver la sécurité et la confidentialité des autres applications.
