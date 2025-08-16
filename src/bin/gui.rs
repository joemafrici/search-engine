use iced::widget::{button, column, text, Column};
use search_engine::index::Index;
#[derive(Default)]
struct Documents {
    index: Index,
    value: String,
}

#[derive(Clone, Debug)]
pub enum Message {
    Init,
    Search,
}

impl Documents {
    pub fn view(&self) -> Column<Message> {
        column![
            button("Initialize").on_press(Message::Init),
            button("Search").on_press(Message::Search),
            text(&self.value).size(50),
        ]
    }
    pub fn update(&mut self, message: Message) {
        match message {
            Message::Search => {
                let results = self.index.search("plato");
                self.value = results[0].snippets[0].clone();
            }
            Message::Init => {
                self.index = Index::new("/home/deepwater/Documents/books")
                    .expect("Should have been able to build index")
            }
        }
    }
}

fn main() -> iced::Result {
    iced::run("Search Engine", Documents::update, Documents::view)
}
