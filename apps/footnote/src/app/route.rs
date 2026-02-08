use dioxus::prelude::*;

use crate::Main;

use crate::body::contact::ContactBrowser;
use crate::body::home::Home;
use crate::body::note::NoteView;
use crate::body::profile::Profile;

#[derive(Debug, Clone, Routable, PartialEq)]
pub enum Route {
    #[layout(Main)]
    #[route("/")]
    Home {},

    #[route("/note/:..file_path_segments")]
    NoteView { file_path_segments: Vec<String> },

    #[route("/contact/:name")]
    ContactBrowser { name: String },

    #[route("/me")]
    Profile {},
}
