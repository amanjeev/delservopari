/// shamelessly running code copied from
/// https://github.com/XAMPPRocky/octocrab/blob/master/examples/poll_events.rs
use octocrab::{
    Octocrab,
    etag::Etagged,
    models::events::{
        payload::{EventPayload, PushEventPayload},
        Event, EventType,
    },
    Page,
};
use std::collections::VecDeque;
use serde::Deserialize;
use serde::Serialize;

const DELAY_MS: u64 = 500;
const TRACKING_CAPACITY: usize = 200;


#[derive(Deserialize, Debug)]
struct Commit {
    message: String,
    url: String,  // yes it will be a string, you got a problem?
}

#[derive(Deserialize, Debug)]
struct CommitStats {
    additions: usize,
    deletions: usize,
    total: usize,
}

#[derive(Deserialize, Debug)]
struct CommitDetails {
    sha: String,
    commit: Commit,
    stats: CommitStats,
}

#[tokio::main]
async fn main() -> octocrab::Result<()> {
    let token = std::env::var("GITHUB_TOKEN").expect("GITHUB_TOKEN env variable is required");
    
    let mut etag = None;
    let mut seen = VecDeque::with_capacity(TRACKING_CAPACITY);
    
    let octo = Octocrab::builder().personal_token(token).build()?;
    
    loop {
        let response: Etagged<Page<Event>> = octo.events().etag(etag).per_page(100).send().await?;
        if let Some(page) = response.value {
            let page = page
                .into_iter()
                .filter(|x| x.r#type == EventType::PushEvent);
            for event in page {
                if !seen.contains(&event.id) {
                    let commits = match event.payload {
                        Some(e) => match e {
                            EventPayload::PushEvent(push_event_payload) => {
                                Some(push_event_payload.commits)
                            }
                            _ => None,
                        },
                        None => None,
                    };

                    // see each commit and judge it for deletion
                    for commit in commits.unwrap() {
                        // fix this ?
                        let commit_response = octo._get(commit.url.to_string(), None::<&()>).await?;
                        let commit_details: CommitDetails = commit_response.json().await.expect("Commit details are busted");

                        if commit_details.stats.additions != 0 && commit_details.stats.deletions != 0 {
                            let ratio = (commit_details.stats.deletions / commit_details.stats.additions) as f64;
                            if ratio > 1.0 {
                                // this is where we tweet
                                println!("DELETE ALL THE THINGS!  {:?}", commit_details);
                            }
                        } else if commit_details.stats.additions == 0 && commit_details.stats.deletions > 0 {
                            // this is where we tweet
                            println!("NO ADDITION IS THE BEST!  {:?}", commit_details);
                        }
                    }
                    if seen.len() == TRACKING_CAPACITY {
                        seen.pop_back();
                    }
                    seen.push_front(event.id);
                }
            }
        }
        etag = response.etag;
        tokio::time::sleep(tokio::time::Duration::from_millis(DELAY_MS)).await;
    }
}


