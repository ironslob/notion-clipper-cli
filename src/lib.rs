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
    pub properties: HashMap<String, NotionDatabaseProperty>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct NotionAnnotations {
    pub bold: Option<bool>,
    pub italic: Option<bool>,
    pub strikethrough: Option<bool>,
    pub underline: Option<bool>,
    pub code: Option<bool>,
    //color: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct NotionUrl {
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

#[derive(Serialize, Deserialize, Debug)]
pub struct NotionDatabaseDate {
}

#[derive(Serialize, Deserialize, Debug)]
pub struct NotionOption {
    pub color: Option<String>,
    pub id: Option<String>,
    pub name: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct NotionSelect {
    pub options: Option<Vec<NotionOption>>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct NotionMultiSelect {
    pub options: Option<Vec<NotionOption>>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct NotionDatabaseProperty {
    pub id: Option<String>,
    #[serde(rename="type")]
    pub _type: String,
    pub date: Option<NotionDatabaseDate>,
    pub multi_select: Option<NotionMultiSelect>,
    pub title: Option<Vec<NotionTitle>>,
    pub text: Option<NotionText>,
    pub rich_text: Option<NotionRichText>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct NotionRichText {
    pub plain_text: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct NotionDatabase {
    pub created_time: String,
    pub last_edited_time: String,
    pub object: String,
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
    pub object: String,
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

