use hashy::*;
use indicatif::{ProgressBar, ProgressStyle};
use reqwest::blocking::Client;
use rusqlite::Connection;
use std::{fmt, fs, path::Path};
use touch::{dir, file};
struct Entry {
    plaintext: String,
    md5: String,
    sha1: String,
    sha256: String,
    sha512: String,
}

impl fmt::Display for Entry {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Plaintext: {}\nMD5: {}\nSHA1: {}\nSHA256: {}\nSHA512: {}",
            self.plaintext, self.md5, self.sha1, self.sha256, self.sha512
        )
    }
}

struct Database {
    conn: Connection,
}

trait Queryable {
    fn add_entry(&self, entry: Entry) -> bool;
    fn query_plaintext(&self, plaintext: String) -> Option<Entry>;
    fn query_hash(&self, hash: String) -> Option<Entry>;
    fn get_count(&self) -> isize;
}

impl Queryable for Database {
    fn add_entry(&self, entry: Entry) -> bool {
        let exists: i64 = self
            .conn
            .query_row(
                "SELECT COUNT(*) FROM rainbow WHERE plaintext = ?1",
                [&entry.plaintext],
                |row| row.get(0),
            )
            .unwrap();

        if exists > 0 {
            return false;
        }

        self.conn.execute(
			"INSERT INTO rainbow (plaintext, md5, sha1, sha256, sha512) VALUES (?1, ?2, ?3, ?4, ?5)",
			[
				&entry.plaintext,
				&entry.md5,
				&entry.sha1,
				&entry.sha256,
				&entry.sha512,
			],
		).unwrap();

        true
    }
    fn query_plaintext(&self, plaintext: String) -> Option<Entry> {
        let query = format!("SELECT * FROM rainbow WHERE {} LIKE ?", "plaintext");

        let entry = self
            .conn
            .prepare(&query)
            .unwrap()
            .query_row([plaintext], |row| {
                Ok(Entry {
                    plaintext: row.get(0).unwrap(),
                    md5: row.get(1).unwrap(),
                    sha1: row.get(2).unwrap(),
                    sha256: row.get(3).unwrap(),
                    sha512: row.get(4).unwrap(),
                })
            });

        entry.ok()
    }
    fn query_hash(&self, hash: String) -> Option<Entry> {
        let query = format!(
			"SELECT * FROM rainbow WHERE {} LIKE ? OR {} LIKE ? OR {} LIKE ? OR {} LIKE ? OR {} LIKE ?",
			"md5",
			"sha1",
			"sha256",
			"sha512",
			"sha512"
		);

        let entry = self.conn.prepare(&query).unwrap().query_row(
            [hash.clone(), hash.clone(), hash.clone(), hash.clone(), hash],
            |row| {
                Ok(Entry {
                    plaintext: row.get(0).unwrap(),
                    md5: row.get(1).unwrap(),
                    sha1: row.get(2).unwrap(),
                    sha256: row.get(3).unwrap(),
                    sha512: row.get(4).unwrap(),
                })
            },
        );

        entry.ok()
    }
    fn get_count(&self) -> isize {
        self.conn
            .prepare("SELECT COUNT(*) FROM rainbow")
            .unwrap()
            .query_row([], |row| row.get(0))
            .unwrap()
    }
}

fn easyselect(prompt: &str, choices: Vec<String>) -> String {
    inquire::Select::new(prompt, choices).prompt().unwrap()
}

fn easyinq(prompt: &str) -> String {
    inquire::Text::new(prompt).prompt().unwrap()
}

fn getdbfile() -> String {
    let basedir: String = format!(
        "{}{}{}",
        dirs::home_dir().unwrap().display(),
        std::path::MAIN_SEPARATOR,
        ".rainbow"
    );
    let dbfile: String = format!("{}{}{}", basedir, std::path::MAIN_SEPARATOR, "rainbow.db");

    dbfile
}

fn construct_entry(plaintext: String) -> Entry {
    Entry {
        plaintext: plaintext.clone(),
        md5: md5(plaintext.clone()),
        sha1: sha1(plaintext.clone()),
        sha256: sha256(plaintext.clone()),
        sha512: sha512(plaintext),
    }
}

fn createdb() -> bool {
    println!("Creating database files...");
    let basedir: String = format!(
        "{}{}{}",
        dirs::home_dir().unwrap().display(),
        std::path::MAIN_SEPARATOR,
        ".rainbow"
    );
    let dbfile: String = getdbfile();

    if !Path::new(basedir.as_str()).exists() {
        println!("Creating directory ~/.rainbow...");

        match dir::create(basedir.as_str()) {
            Ok(_) => println!("Created directory ~/.rainbow"),
            Err(_) => panic!("Failed to create database files. Do you have the right permissions?"),
        };
    }

    if !Path::new(dbfile.as_str()).exists() {
        println!("Creating database file...");

        match file::create(dbfile.as_str(), false) {
            Ok(_) => println!("Created file ~/.rainbow/rainbow.db"),
            Err(_) => {
                println!("Failed to create database files. Do you have the right permissions?")
            }
        }
    }

    Connection::open(dbfile)
        .unwrap()
        .execute(
            "CREATE TABLE IF NOT EXISTS rainbow (
			plaintext TEXT PRIMARY KEY,
			md5 TEXT NOT NULL,
			sha1 TEXT NOT NULL,
			sha256 TEXT NOT NULL,
			sha512 TEXT NOT NULL
		)",
            [],
        )
        .unwrap();

    true
}

fn main() {
    if !Path::new(getdbfile().as_str()).exists() {
        createdb();
    }

    println!("Welcome to RainbowTable v{}", env!("CARGO_PKG_VERSION"));

    let db: Database = Database {
        conn: Connection::open(getdbfile()).unwrap(),
    };

    let opts = vec![
        String::from("Add string"),
        String::from("Lookup string"),
        String::from("Lookup hash"),
        String::from("Add file"),
        String::from("Get count of entries"),
    ];

    match easyselect("What would you like to do?", opts.clone()) {
        choice if choice == opts[0] => {
            let plaintext = easyinq("Enter the plaintext value to add to the rainbow table:");
            let entry = construct_entry(plaintext.clone());
            db.add_entry(entry);
            println!("Added {} to the rainbow table.", plaintext)
        }

        choice if choice == opts[1] => {
            let entry = db
                .query_plaintext(easyinq("Enter the plaintext value to lookup:"))
                .unwrap();

            println!("\n{}", entry);
        }

        choice if choice == opts[2] => {
            let entry: Entry = db
                .query_hash(easyinq("Enter the hash value to lookup:"))
                .unwrap();

            println!("\n{}", entry);
        }

        choice if choice == opts[3] => {
            let fileopts: Vec<String> = vec![String::from("Local file"), String::from("URL")];

            let body: String = match easyselect("Where is the file located?", fileopts.clone()) {
                choice if choice == fileopts[0] => {
                    let strpath =
                        easyinq("Enter the full path to the file (Will parse on each newline)");
                    String::from_utf8_lossy(&fs::read(strpath).unwrap())
                        .parse()
                        .unwrap()
                }

                choice if choice == fileopts[1] => {
                    let url = easyinq("Enter the URL of the file (Will parse on each newline):");
                    let client = Client::new();
                    let response = client.get(url).send().unwrap();
                    response.text().unwrap()
                }

                _ => panic!("Invalid choice"),
            };

            let lines: Vec<&str> = body.lines().collect();
            let total_lines = lines.len();
            let mut skipped = 0;

            let progress_bar = ProgressBar::new(total_lines as u64);
            progress_bar.set_style(
                ProgressStyle::default_bar()
                    .template("[{elapsed_precise}] {bar:40.cyan/blue} {pos}/{len} ({percent}%)")
                    .unwrap(),
            );

            let mut count = 0;
            for line in lines {
                if !line.is_empty() {
                    if db.add_entry(construct_entry(line.to_string())) {
                        count += 1;
                    } else {
                        skipped += 1;
                    }
                }

                progress_bar.set_position(count as u64);
                let msg = format!("{}/{}", count, total_lines);
                progress_bar.set_message(msg);
            }

            progress_bar.finish_with_message("Processing complete.");
            println!(
                "Added {} entries to the rainbow table (skipped {}).",
                count, skipped
            );
        }

        choice if choice == opts[4] => {
            let count = db.get_count();
            println!("There are {} entries in the rainbow table.", count);
        }

        _ => panic!("Invalid choice"),
    }
}
