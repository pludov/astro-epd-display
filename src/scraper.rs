use tokio::{
    io::{AsyncBufReadExt, BufReader},
    process::ChildStdout,
    task,
};

use crate::{cli::Args, device_driver, state};

async fn parse(stdout: ChildStdout) {
    let reader = BufReader::new(stdout);
    let mut lines = reader.lines();
    while let Some(line) = lines.next_line().await.unwrap() {
        // JSON parse
        let parsed: Result<serde_json::Value, _> = serde_json::from_str(&line);
        if parsed.is_err() {
            eprintln!("Error parsing JSON: {:?}", parsed);
            continue;
        }
        let parsed = parsed.unwrap();

        // Send to the conntext script
        match state::merge_state(parsed, device_driver::RefreshSignal::Normal) {
            Ok(_) => {}
            Err((status, message)) => {
                eprintln!("Error merging state: {:?} {}", status, message);
            }
        }
    }
}

async fn scrape_once(scrape_program: &str) -> Result<(), std::io::Error> {
    let mut child = tokio::process::Command::new(scrape_program)
        .stdout(std::process::Stdio::piped())
        .kill_on_drop(true)
        .spawn()?;

    let stdout = child.stdout.take().unwrap();
    let read = task::spawn(async move { parse(stdout).await });

    let status = child.wait().await.expect("Failed to wait on child");

    if !status.success() {
        eprintln!("Scrape program failed: {:?}", status);
    }
    // Wait the read task to finish
    read.await.expect("Failed to wait on read task");
    Ok(())
}

async fn scrape(scrape_program: &str) {
    loop {
        match scrape_once(scrape_program).await {
            Ok(_) => {
                tokio::time::sleep(std::time::Duration::from_millis(100)).await;
            }
            Err(err) => {
                eprintln!("Error scraping: {:?}", err);
                // Introduce a delay before retrying
                tokio::time::sleep(std::time::Duration::from_secs(2)).await;
            }
        }
    }
}

pub fn start_scraper(args: &Args) {
    if let Some(scrape_program) = args.scrape_command.clone() {
        task::spawn(async move {
            scrape(&scrape_program).await;
        });
    }
}
