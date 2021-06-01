use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use std::io;
use std::io::Write;
use reqwest::blocking::Client;
use reqwest::header;
use std::env;
use std::process;

/*
- config loading (or defaulting)
    - add confy crate
    - figure out how to use confy - start with most basic complete example
    - structure for config
    - integration key
    - database id
    - "title" property
- config option to configure
    - accept integration key
    - load databases from notion
    - select database id
    - retrieve database, extract title property
    - store config
- cli option to track in database
    - check if the config is empty and trigger config option as above
    - extract text from command line
    - push into notion via api
    - handle unauthenticated error - re-auth
    - handle other errors
- help page
    - load by default
*/

static CONFY_NAME: &str = "notion_clipper";

#[derive(Serialize, Deserialize, Debug)]
struct NotionConfig {
    access_secret: String,
    database_id: String,
    title_property: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct NotionParent {
    #[serde(rename="type")]
    _type: String,
    database_id: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct NotionPage {
    parent: NotionParent,
    properties: HashMap<String, NotionDatabaseProperty>,
}

#[derive(Serialize, Deserialize, Debug)]
struct NotionAnnotations {
    bold: Option<bool>,
    italic: Option<bool>,
    strikethrough: Option<bool>,
    underline: Option<bool>,
    code: Option<bool>,
    //color: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct NotionUrl {
}

#[derive(Serialize, Deserialize, Debug)]
struct NotionText {
    content: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
struct NotionTitle {
    #[serde(rename="type")]
    _type: String,
    text: Option<NotionText>,
}

#[derive(Serialize, Deserialize, Debug)]
struct NotionDatabaseDate {
}

#[derive(Serialize, Deserialize, Debug)]
struct NotionOption {
    color: Option<String>,
    id: Option<String>,
    name: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
struct NotionSelect {
    options: Option<Vec<NotionOption>>,
}

#[derive(Serialize, Deserialize, Debug)]
struct NotionMultiSelect {
    options: Option<Vec<NotionOption>>,
}

#[derive(Serialize, Deserialize, Debug)]
struct NotionDatabaseProperty {
    id: Option<String>,
    #[serde(rename="type")]
    _type: String,
    date: Option<NotionDatabaseDate>,
    multi_select: Option<NotionMultiSelect>,
    title: Option<Vec<NotionTitle>>,
    text: Option<NotionText>,
    rich_text: Option<NotionRichText>,
}

#[derive(Serialize, Deserialize, Debug)]
struct NotionRichText {
    plain_text: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
struct NotionDatabase {
    created_time: String,
    last_edited_time: String,
    object: String,
    id: String,
    title: Vec<NotionRichText>,
    properties: HashMap<String, NotionDatabaseProperty>,
}

#[derive(Serialize, Deserialize, Debug)]
struct NotionDatabaseList {
    object: String,
    results: Vec<NotionDatabase>,
    next_cursor: Option<String>,
    has_more: bool,
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

fn help(cmd: &String) {
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

fn configure_and_save(existing: &NotionConfig) -> NotionConfig {
    let cfg: NotionConfig = configure(&existing);

    confy::store(CONFY_NAME, &cfg).expect("fsck");

    return cfg;
}

#[derive(Debug)]
struct Database {
    id: String,
    title: String,
    title_property: String,
}


fn get_databases(secret_token: &String) -> Vec<Database> {
    let mut databases: Vec<Database> = Vec::new();

    let url = "https://api.notion.com/v1/databases";

    // TODO must add error response checking via Result

    let client = Client::new();
    let mut headers = header::HeaderMap::new();
    let auth = format!("Bearer {}", secret_token);
    headers.insert(header::AUTHORIZATION, auth.parse().unwrap());

    let response = client.get(url)
        .headers(headers)
        .send()
        .unwrap();

    let body = response.text().unwrap();

    let json_body: NotionDatabaseList = serde_json::from_str(body.as_str()).unwrap();

    let results = json_body.results;

    for result in results.iter() {
        let database_id = result.id.clone();
        let title = result.title[0].plain_text.clone();
        let mut title_prop: String = String::new();

        for (key, prop) in result.properties.iter() {
            if prop._type == "title" {
                title_prop = key.clone();
                break;
            }
        }

        /*
        let database_id = result.get("id").unwrap().to_string();
        let title =  result
            .get("title")
            .unwrap()
            .get(0)
            .unwrap()
            .get("plain_text")
            .unwrap()
            .to_string();
        */

        databases.push(Database {
            id: database_id,
            title: title.unwrap(),
            title_property: title_prop,
        });
    }

    return databases;
}

fn configure(existing: &NotionConfig) -> NotionConfig {
    println!("\
To access your Notion account we need an integration access token, follow these steps to create one:

1. Visit https://www.notion.so/my-integrations
2. Click \"+ New integration\"
3. For name choose something you'll associate with this account
4. Select a logo if you wish (it won't be used here)
5. Choose a workspace that you want to grant access to - we'll choose a database in the next step
6. Click \"Submit\"

You will now be presented with information about your new integration, including a \"Internal Integration Token\". Click \"Show\" next to this, then \"Copy\", and paste the full string below.
");

    print!("> ");

    // flushing, not gonna worry about errors here
    io::stdout().flush().unwrap();

    let mut secret = String::new();
    io::stdin()
        .read_line(&mut secret)
        .expect("Failed to read");

    secret = secret.trim().to_string();

    println!("
Great, now I'm going to pull a list of databases so we can decide which one we want to add to.
");

    let databases = get_databases(&secret);
    
    // TODO check that there are some databases, handle errors

    let database = choose_database(&databases);

    let database_id = database.id.clone();
    let title_property = database.title_property.clone();

    NotionConfig {
        access_secret: secret,
        database_id: database_id,
        title_property: title_property,
    }
}

fn choose_database(databases: &Vec<Database>) -> &Database {
    println!("Found {} databases, please choose one:", databases.len());

    for index in 0..databases.len() {
        println!("[{}] - {} ({})", index, databases[index].title, databases[index].id);
    }

    loop {
        print!("Enter number > ");
        io::stdout().flush().unwrap();

        let mut entry = String::new();
        io::stdin().read_line(&mut entry).unwrap();

        let entry_id: usize = match entry.trim().parse() {
            Ok(num) => num,
            Err(_) => {
                println!("Try again");
                continue;
            },
        };

        let database_count = databases.len();

        if entry_id < database_count {
            return &databases[entry_id];
        } else {
            println!("Try again");
        }
    }
}

fn create_page(cfg: &NotionConfig, title: &String) {
    print!("Creating page \"{}\"... ", title);
    io::stdout().flush().unwrap();

    let title_prop = NotionTitle {
        _type: String::from("text"),
        text: Some(NotionText {
            content: Some(title.clone()),
        }),
    };
    let mut properties: HashMap<String, NotionDatabaseProperty> = HashMap::new();
    properties.insert(cfg.title_property.clone(), NotionDatabaseProperty {
        _type: String::from("title"),
        date: None,
        id: None,
        multi_select: None,
        rich_text: None,
        text: None,
        title: Some(vec![
            title_prop,
        ]),
    });

    let payload = NotionPage {
        parent: NotionParent {
            _type: String::from("database_id"),
            database_id: cfg.database_id.clone(),
        },
        properties: properties,
    };

    let payload_json = serde_json::to_string(&payload);

    let url = "https://api.notion.com/v1/pages";

    let client = Client::new();
    let mut headers = header::HeaderMap::new();
    let auth = format!("Bearer {}", cfg.access_secret);
    headers.insert(header::AUTHORIZATION, auth.parse().unwrap());
    headers.insert(header::CONTENT_TYPE, "application/json".parse().unwrap());

    let response = client.post(url)
        .headers(headers)
        .body(payload_json.unwrap())
        .send()
        .unwrap();

    if !response.status().is_success() {
        panic!("Error from Notion: {}", response.text().unwrap());
    }

    println!("✅");
}

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        help(&args[0]);
        process::exit(1);
    }

    let mut cfg: NotionConfig = match confy::load(CONFY_NAME) {
        Ok(cfg) => cfg,
        _ => {
            println!("*** Problems loading previous config, starting over ***");
            NotionConfig::default()
        },
    };

    let mut configured = false;

    if cfg.access_secret == "" || cfg.database_id == "" || cfg.title_property == "" {
        cfg = configure_and_save(&cfg);
        configured = true;

        if args[1] == "configure" {
            process::exit(0);
        }
    }

    if args[1] == "configure" {
        if !configured {
            configure_and_save(&cfg);
        }
    } else if args[1] == "help" {
        help(&args[0]);
    } else {
        for index in 1..args.len() {
            create_page(&cfg, &args[index]);
        }
    }
}
