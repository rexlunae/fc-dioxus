//! Accessibility command line interface.
#![forbid(unsafe_code)]

use fc_a11y::{Audit, Document};
use std::{
    env, fs,
    io::{self, Read},
    process::ExitCode,
};

fn main() -> ExitCode {
    match run() {
        Ok(false) => ExitCode::SUCCESS,
        Ok(true) => ExitCode::FAILURE,
        Err(error) => {
            eprintln!("fc-a11y: {error}");
            ExitCode::from(2)
        }
    }
}
fn run() -> Result<bool, String> {
    let mut args = env::args().skip(1);
    match args.next().as_deref() {
        Some("rules") => {
            for rule in fc_a11y::rules::catalogue() {
                println!("{}\tWCAG {}", rule.id(), rule.criteria().join(", "));
            }
            Ok(false)
        }
        Some("check") => {
            let paths: Vec<_> = args.collect();
            if paths.is_empty() {
                return Err("check requires an HTML file or - for stdin".to_owned());
            }
            let mut failed = false;
            for path in paths {
                let mut html = String::new();
                if path == "-" {
                    io::stdin()
                        .read_to_string(&mut html)
                        .map_err(|error| error.to_string())?;
                } else {
                    html = fs::read_to_string(&path).map_err(|error| format!("{path}: {error}"))?;
                }
                let document = Document::parse(&html);
                let report = Audit.run(&document);
                for finding in &report.findings {
                    let tag = document
                        .nodes
                        .get(finding.node)
                        .map_or("?", |node| node.tag.as_str());
                    println!(
                        "{path}:{} <{tag}> {:?} [{}] {} {}",
                        finding.node, finding.severity, finding.rule, finding.message, finding.help
                    );
                }
                println!("{path}: {} errors, {} findings ({} static rules; not a conformance certification)", report.errors(), report.findings.len(), fc_a11y::rules::catalogue().len());
                failed |= report.errors() > 0;
            }
            Ok(failed)
        }
        _ => Err("usage: fc-a11y check <HTML|->... | rules".to_owned()),
    }
}
