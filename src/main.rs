use std::ffi::OsStr;
use url::Url;
use ego_tree::iter::Edge::*;
use scraper::{Html, Node, Selector};
use walkdir::{WalkDir};

#[derive(Default)]
struct Current {
    path: String,
    id: String,
    text: String,
}

fn main() {
    let dir = std::env::args()
        .nth(1)
        .expect("gimme the path of the documentation");
    let walkdir = WalkDir::new(&dir)
        .into_iter()
        .map(|res| res.expect("no idea why this can fail"))
        .filter(|file| file.path().extension() == Some(OsStr::new("html")));

    println!("[");

    for file in walkdir {
        eprintln!("inspecting: {}", file.path().display());

        let html = std::fs::read_to_string(file.path()).expect("could not read one of the file");
        let html = Html::parse_document(&html);
        let main_section = Selector::parse("#main").unwrap();
        let main_section = html.select(&main_section).next();
        if main_section.is_none() {
            continue;
        }
        let main_section = main_section.unwrap();

        let mut current = Current::new(format!("{}", file.path().display()), "main".to_string());

        for node in main_section.traverse() {
            match node {
                Open(node) => current.handle_open(node.value()),
                Close(node) => current.handle_close(node.value()),
            }
        }
    }

    println!("]");
}

impl Current {
    pub fn new(path: String, base_id: String) -> Self {
        Self {
            path,
            id: base_id,
            text: String::new(),
        }
    }

    pub fn handle_open(&mut self, node: &Node) {
        match node {
            Node::Text(t) => self.text.push_str(t),
            Node::Element(el) => {
                match el.name() {
                    "h1" | "h2" | "h3" | "h4" | "h5" | "h6" => {
                        let url = Url::parse(&format!("https://doc.rust-lang.org/stable/std/{}#{}", self.path, self.id)).unwrap();
                        println!(
                            r#"{{ "url": "{}", "content": "{}" }},"#,
                            url.as_str(), self.text.replace("\\", "\\\\").replace("\"", "\\\"").replace("\n", "\\n"), // it's too late to import serde
                        );
                        if let Some(id) = el.id() {
                            self.id = id.to_string();
                            self.text.clear();
                        }
                    }
                    _ => (),
                }
            }
            _ => (),
        }
    }

    pub fn handle_close(&mut self, node: &Node) {
        match node {
            Node::Element(el) => (), //println!("{:?}", el),
            Node::Text(t) => (), //println!("{:?}", t),
            _ => (),
        }
    }
}
