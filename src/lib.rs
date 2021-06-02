use std::collections::HashMap;
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct NotionParent {
    #[serde(rename="type")]
    pub _type: String,
    pub database_id: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct NotionPage {
    pub parent: NotionParent,
    pub properties: HashMap<String, NotionPageProperty>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct NotionText {
    pub content: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct NotionTitle {
    #[serde(rename="type")]
    pub _type: String,
    pub text: Option<NotionText>,
}

// FIXME notion api returns "{}" as the empty title property, but expects an array if there's
// value. i'm not sure how to handle that.
#[derive(Serialize, Deserialize, Debug)]
pub struct NotionPageProperty {
    #[serde(rename="type")]
    pub _type: String,
    pub title: Option<Vec<NotionTitle>>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct NotionDatabaseProperty {
    #[serde(rename="type")]
    pub _type: String,
    //pub title: Option<Vec<NotionTitle>>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct NotionRichText {
    pub plain_text: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct NotionDatabase {
    pub id: String,
    pub title: Vec<NotionRichText>,
    pub properties: HashMap<String, NotionDatabaseProperty>,
}

impl NotionDatabase {
    pub fn title_text(&self) -> String {
        return self.title[0].plain_text.clone().unwrap();
    }

    pub fn title_property(&self) -> String {
        let mut title_prop: String = String::new();

        for (key, prop) in self.properties.iter() {
            if prop._type == "title" {
                title_prop = key.clone();
                break;
            }
        }

        return title_prop;
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct NotionDatabaseList {
    pub results: Vec<NotionDatabase>,
    pub next_cursor: Option<String>,
    pub has_more: bool,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct NotionConfig {
    pub access_secret: String,
    pub database_id: String,
    pub title_property: String,
}

impl Default for NotionConfig {
    fn default() -> Self {
        NotionConfig {
            access_secret: String::from(""),
            database_id: String::from(""),
            title_property: String::from(""),
        }
    }
}

pub fn help(cmd: &String) {
    println!("\
{} [option]

Command line tool to quickly add items to a Notion database. Will only set the title of the new
page, no properties.

Options include:

    help      - this page
    configure - updates values for Notion database access
    *         - add content to Notion

All other parameters will be added to the configured Notion database as individual pages.

e.g. $ {} \"Chocolate hobnobs\" \"Beer\"

Will add two new Notion pages to the configured Notion database.
", cmd, cmd);
}

