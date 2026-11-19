#![allow(dead_code)]
//! Estados de la aplicación (`Menu`, `Flight(SystemId)`, `Gallery`) y sus
//! transiciones por teclado.

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SystemId {
    Realista,
    Favoritos,
}

impl SystemId {
    pub fn display_name(self) -> &'static str {
        match self {
            SystemId::Realista => "Sistema solar realista",
            SystemId::Favoritos => "Sistema de favoritos",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AppState {
    Menu,
    Flight(SystemId),
    Gallery,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MenuEntry {
    RealisticSystem,
    FavoritesSystem,
    Gallery,
    Controls,
    Exit,
}

pub const MENU_ENTRIES: [MenuEntry; 5] = [
    MenuEntry::RealisticSystem,
    MenuEntry::FavoritesSystem,
    MenuEntry::Gallery,
    MenuEntry::Controls,
    MenuEntry::Exit,
];

impl MenuEntry {
    pub fn label(self) -> &'static str {
        match self {
            MenuEntry::RealisticSystem => "Sistema solar realista",
            MenuEntry::FavoritesSystem => "Sistema de favoritos",
            MenuEntry::Gallery => "Galería de trofeos",
            MenuEntry::Controls => "Controles",
            MenuEntry::Exit => "Salir",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TransitionOutcome {
    Stay,
    ChangeState(AppState),
    ToggleControls,
    ExitApp,
}

#[derive(Clone, Debug)]
pub struct AppController {
    pub state: AppState,
    pub selected_menu: usize,
    pub show_menu_controls: bool,
}

impl Default for AppController {
    fn default() -> Self {
        Self::new()
    }
}

impl AppController {
    pub fn new() -> Self {
        Self {
            state: AppState::Menu,
            selected_menu: 0,
            show_menu_controls: false,
        }
    }

    pub fn move_menu_up(&mut self) {
        if self.state != AppState::Menu || self.show_menu_controls {
            return;
        }
        self.selected_menu = if self.selected_menu == 0 {
            MENU_ENTRIES.len() - 1
        } else {
            self.selected_menu - 1
        };
    }

    pub fn move_menu_down(&mut self) {
        if self.state != AppState::Menu || self.show_menu_controls {
            return;
        }
        self.selected_menu = (self.selected_menu + 1) % MENU_ENTRIES.len();
    }

    pub fn on_escape(&mut self) -> TransitionOutcome {
        match self.state {
            AppState::Menu => TransitionOutcome::ExitApp,
            AppState::Flight(_) | AppState::Gallery => {
                self.state = AppState::Menu;
                self.show_menu_controls = false;
                TransitionOutcome::ChangeState(AppState::Menu)
            }
        }
    }

    pub fn activate_menu(&mut self, favorites_ready: bool) -> TransitionOutcome {
        if self.state != AppState::Menu {
            return TransitionOutcome::Stay;
        }
        if self.show_menu_controls {
            self.show_menu_controls = false;
            return TransitionOutcome::ToggleControls;
        }
        match MENU_ENTRIES[self.selected_menu] {
            MenuEntry::RealisticSystem => {
                self.state = AppState::Flight(SystemId::Realista);
                TransitionOutcome::ChangeState(self.state)
            }
            MenuEntry::FavoritesSystem => {
                if favorites_ready {
                    self.state = AppState::Flight(SystemId::Favoritos);
                    TransitionOutcome::ChangeState(self.state)
                } else {
                    TransitionOutcome::Stay
                }
            }
            MenuEntry::Gallery => {
                self.state = AppState::Gallery;
                TransitionOutcome::ChangeState(self.state)
            }
            MenuEntry::Controls => {
                self.show_menu_controls = true;
                TransitionOutcome::ToggleControls
            }
            MenuEntry::Exit => TransitionOutcome::ExitApp,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn escape_returns_to_menu_from_flight_and_gallery() {
        let mut app = AppController::new();
        assert_eq!(app.state, AppState::Menu);

        app.state = AppState::Flight(SystemId::Realista);
        assert_eq!(
            app.on_escape(),
            TransitionOutcome::ChangeState(AppState::Menu)
        );
        assert_eq!(app.state, AppState::Menu);

        app.state = AppState::Flight(SystemId::Favoritos);
        assert_eq!(
            app.on_escape(),
            TransitionOutcome::ChangeState(AppState::Menu)
        );
        assert_eq!(app.state, AppState::Menu);

        app.state = AppState::Gallery;
        assert_eq!(
            app.on_escape(),
            TransitionOutcome::ChangeState(AppState::Menu)
        );
        assert_eq!(app.state, AppState::Menu);
    }

    #[test]
    fn escape_on_menu_exits_app() {
        let mut app = AppController::new();
        assert_eq!(app.on_escape(), TransitionOutcome::ExitApp);
    }

    #[test]
    fn menu_navigation_wraps_and_activates_states() {
        let mut app = AppController::new();
        app.move_menu_up();
        assert_eq!(app.selected_menu, MENU_ENTRIES.len() - 1);
        app.move_menu_down();
        assert_eq!(app.selected_menu, 0);

        // Entrada 0: Sistema solar realista
        assert_eq!(
            app.activate_menu(false),
            TransitionOutcome::ChangeState(AppState::Flight(SystemId::Realista))
        );
        assert_eq!(app.state, AppState::Flight(SystemId::Realista));

        // Volver y abrir galería (entrada 2)
        app.on_escape();
        app.move_menu_down();
        app.move_menu_down();
        assert_eq!(
            app.activate_menu(false),
            TransitionOutcome::ChangeState(AppState::Gallery)
        );
        assert_eq!(app.state, AppState::Gallery);

        // Volver, abrir controles (entrada 3) y cerrar con Enter
        app.on_escape();
        app.move_menu_down();
        assert_eq!(app.activate_menu(false), TransitionOutcome::ToggleControls);
        assert!(app.show_menu_controls);
        assert_eq!(app.activate_menu(false), TransitionOutcome::ToggleControls);
        assert!(!app.show_menu_controls);

        // Salir (entrada 4)
        app.move_menu_down();
        assert_eq!(app.activate_menu(false), TransitionOutcome::ExitApp);
    }

    #[test]
    fn disabled_favorites_cannot_be_activated_from_menu() {
        let mut app = AppController::new();
        app.move_menu_down(); // Sistema de favoritos
        assert_eq!(app.activate_menu(false), TransitionOutcome::Stay);
        assert_eq!(app.state, AppState::Menu);

        assert_eq!(
            app.activate_menu(true),
            TransitionOutcome::ChangeState(AppState::Flight(SystemId::Favoritos))
        );
    }
}
