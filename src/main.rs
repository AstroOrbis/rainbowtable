use hashy::*;
use indicatif::{ProgressBar, ProgressStyle};
use reqwest::blocking::Client;
use rusqlite::Connection;
use std::fmt;
use std::path::Path;
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

fn filesep() -> String {
	match std::env::consts::OS {
		"windows" => String::from("\\"),
		_ => String::from("/"),
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
		filesep(),
		".rainbow"
	);
	let dbfile: String = format!("{}{}{}", basedir, filesep(), "rainbow.db");

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

fn add_to_table(conn: Connection, entry: Entry, verbose: bool) -> bool {
	if verbose {
		println!("Adding \"{}\" to the rainbow table...", entry.plaintext);
	}

	let exists: i64 = conn
		.query_row(
			"SELECT COUNT(*) FROM rainbow WHERE plaintext = ?1",
			[&entry.plaintext],
			|row| row.get(0),
		)
		.unwrap();

	if exists > 0 {
		if verbose {
			println!(
				"Entry with plaintext \"{}\" already exists in the rainbow table.",
				entry.plaintext
			);
		}
		return false;
	}

	conn.execute(
		"INSERT INTO rainbow (plaintext, md5, sha1, sha256, sha512) VALUES (?1, ?2, ?3, ?4, ?5)",
		[
			&entry.plaintext,
			&entry.md5,
			&entry.sha1,
			&entry.sha256,
			&entry.sha512,
		],
	)
	.unwrap();

	if verbose {
		println!("Added an entry to the rainbow table:");
		println!("{}", entry);
	}

	true
}

fn createdb() -> bool {
	println!("Creating database files...");
	let basedir: String = format!(
		"{}{}{}",
		dirs::home_dir().unwrap().display(),
		filesep(),
		".rainbow"
	);
	let dbfile: String = format!("{}{}{}", basedir, filesep(), "rainbow.db");

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

	Connection::open(getdbfile())
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

	let conn = Connection::open(getdbfile()).unwrap();

	let opts = vec![
		String::from("Add string to rainbow table"),
		String::from("Lookup string in rainbow table"),
		String::from("Add remote file list to rainbow table"),
		String::from("Get count of entries in rainbow table")
	];

	match easyselect("What would you like to do?", opts.clone()) {
		choice if choice == opts[0] => {
			add_to_table(
				conn,
				construct_entry(easyinq("Enter a string to add to the rainbow table.")),
				true,
			);
		}

		choice if choice == opts[1] => {
			let types: Vec<String> = vec![
				String::from("plaintext"),
				String::from("md5"),
				String::from("sha1"),
				String::from("sha256"),
				String::from("sha512"),
			];

			let lv = easyinq("Enter the value to lookup:");

			let query = format!(
				"SELECT * FROM rainbow WHERE {} LIKE ? OR {} LIKE ? OR {} LIKE ? OR {} LIKE ? OR {} LIKE ?",
				types[0],
				types[1],
				types[2],
				types[3],
				types[4]
			);

			let entry = conn
				.prepare(&query)
				.unwrap()
				.query_row(
					[lv.clone(), lv.clone(), lv.clone(), lv.clone(), lv],
					|row| {
						Ok(Entry {
							plaintext: row.get(0).unwrap(),
							md5: row.get(1).unwrap(),
							sha1: row.get(2).unwrap(),
							sha256: row.get(3).unwrap(),
							sha512: row.get(4).unwrap(),
						})
					},
				)
				.unwrap();

			println!("\n{}", entry);
		}

		choice if choice == opts[2] => {
			let url = easyinq("Enter the URL of the file (Will parse on each newline):");
			let client = Client::new();
			let response = client.get(url).send().unwrap();

			// Read the lines from the downloaded file
			let body: String = response.text().unwrap();
			let lines: Vec<&str> = body.lines().collect();
			let total_lines = lines.len();
			let mut skipped = 0;

			// Create a progress bar
			let progress_bar = ProgressBar::new(total_lines as u64);
			progress_bar.set_style(
				ProgressStyle::default_bar()
					.template("[{elapsed_precise}] {bar:40.cyan/blue} {pos}/{len} ({percent}%)")
					.unwrap(),
			);

			let mut count = 0;
			for line in lines {
				if !line.is_empty() {
					if add_to_table(
						Connection::open(getdbfile()).unwrap(),
						construct_entry(line.to_string()),
						false,
					) {
						count += 1;
					} else {
						skipped += 1;
					}
				}

				// Update the progress bar and fraction
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

		choice if choice == opts[3] => {
			let count: isize = conn
				.prepare("SELECT COUNT(*) FROM rainbow")
				.unwrap()
				.query_row([], |row| row.get(0))
				.unwrap();

			println!("There are {} entries in the rainbow table.", count);
		}

		_ => panic!("Invalid choice"),
	}
}
