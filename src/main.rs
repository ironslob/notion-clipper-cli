use std::collections::HashMap;
use std::io;
use std::io::Write;
use reqwest::blocking::Client;
use reqwest::header;
use std::env;
use std::process;

use notion_clipper_cli::NotionPage;
use notion_clipper_cli::NotionParent;
use notion_clipper_cli::NotionConfig;
use notion_clipper_cli::NotionDatabase;
use notion_clipper_cli::NotionPageProperty;
use notion_clipper_cli::NotionDatabaseProperty;
use notion_clipper_cli::NotionText;
use notion_clipper_cli::NotionTitle;
use notion_clipper_cli::NotionDatabaseList;

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

fn configure_and_save(existing: &NotionConfig) -> NotionConfig {
    let cfg: NotionConfig = configure(&existing);

    confy::store(CONFY_NAME, &cfg).expect("fsck");

    return cfg;
}

fn get_databases(secret_token: &String) -> Vec<NotionDatabase> {
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

    return json_body.results;
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

    let mut rl = rustyline::Editor::<()>::new();
    let secret = rl.readline_with_initial("> ", (&existing.access_secret, "")).unwrap();

    println!("
Great, now I'm going to pull a list of databases so we can decide which one we want to add to.
");

    let databases = get_databases(&secret);
    
    // TODO check that there are some databases, handle errors

    let database = choose_database(&databases);

    let database_id = database.id.clone();
    let title_property = database.title_property();

    NotionConfig {
        access_secret: secret,
        database_id: database_id,
        title_property: title_property,
    }
}

fn choose_database(databases: &Vec<NotionDatabase>) -> &NotionDatabase {
    println!("Found {} databases, please choose one:\n", databases.len());

    for index in 0..databases.len() {
        println!("[{}] - {} ({})", index, databases[index].title_text(), databases[index].id);
    }

    println!("");

    let mut rl = rustyline::Editor::<()>::new();

    loop {
        let entry = rl.readline("Enter number > ").unwrap();
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

    let mut properties: HashMap<String, NotionPageProperty> = HashMap::new();
    properties.insert(cfg.title_property.clone(), NotionPageProperty {
        _type: String::from("title"),
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
        notion_clipper_cli::help(&args[0]);
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
        notion_clipper_cli::help(&args[0]);
    } else {
        for index in 1..args.len() {
            create_page(&cfg, &args[index]);
        }
    }
}
