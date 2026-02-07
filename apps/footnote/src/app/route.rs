#[derive(Debug, Clone, Routable, PartialEq)]
enum Route {
    #[layout(Main)]
    #[route("/")]
    Home {},
    #[route("/note/:..file_path_segments")]
    NoteView { file_path_segments: Vec<String> },
    #[route("/contact/:name")]
    ContactView { name: String },
    #[route("/me")]
    ProfileView {},
}
