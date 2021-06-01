#  Notion Clipper CLI

An exercise in learning Rust by creating a CLI tool for adding to my GTD Notion
database.

# Building

$ cargo build --release

Will create a binary in target/release/notion-clipper-cli

# Running

The tool will self-configure if it doesn't have what it needs, but it's not yet
ready to work till Notion resolves some confusion around the use of
/v1/databases with private integrations.
