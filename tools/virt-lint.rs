/* SPDX-License-Identifier: LGPL-3.0-or-later */

use clap::Parser;
use std::fs;
use std::io;
use std::io::Read;

use virt::connect::Connect;
use virt_lint::*;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// connection uri
    #[arg(short = 'c', long = "connect", value_name = "URI")]
    uri: Option<String>,

    /// The path to the domain XML, otherwise read the XML from stdin
    #[arg(short, long, value_name = "FILE")]
    path: Option<std::path::PathBuf>,

    /// Turn debugging information on
    #[arg(short, long)]
    debug: bool,

    /// Comma separated list of validator tags, empty means all
    #[arg(short, long, value_delimiter = ',')]
    validators: Option<Vec<String>>,

    /// List known validator tags
    #[arg(short = 'l', long = "list-validator-tags")]
    list: bool,
}

fn main() {
    let mut domxml = String::new();

    let cli = Args::parse();

    if cli.list {
        if cli.debug {
            println!("Validator tags:");
        }

        VirtLint::list_validator_tags()
            .expect("Unable to list validator tags")
            .iter()
            .for_each(|tag| println!("{tag}"));

        return;
    }

    if let Some(file) = cli.path {
        domxml = fs::read_to_string(file).expect("Unable to read the file");
    } else {
        io::stdin()
            .read_to_string(&mut domxml)
            .expect("Unable to read stdin");
    }

    if cli.debug {
        dbg!(
            "Attempting to connect to hypervisor: '{:?}'",
            cli.uri.as_deref()
        );
    }

    let mut conn = match Connect::open(cli.uri.as_deref()) {
        Ok(c) => c,
        Err(e) => panic!("No connection to hypervisor: {}", e),
    };

    let mut l = VirtLint::new(Some(&conn));

    if let Err(e) = conn.close() {
        panic!("Failed to disconnect from hypervisor: {}", e);
    }

    if let Err(e) = l.validate(&domxml, &cli.validators.unwrap_or_default(), false) {
        println!("Validation failed: {}", e);
    }

    for w in l.warnings().iter() {
        let (tags, domain, level, msg) = w.get();
        println!(
            "Warning: tags={:?}\tdomain={domain}\tlevel={level}\tmsg={msg}",
            tags
        );
    }
}
