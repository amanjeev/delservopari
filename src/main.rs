use egg_mode::tweet;
/// shamelessly running code copied from
/// https://github.com/XAMPPRocky/octocrab/blob/master/examples/poll_events.rs
use octocrab::{
    etag::Etagged,
    models::events::{payload::EventPayload, Event, EventType},
    Octocrab, Page,
};
use serde::Deserialize;
use std::collections::VecDeque;

const DELAY_MS: u64 = 1000;
const TRACKING_CAPACITY: usize = 2;

#[derive(Deserialize, Debug)]
struct Commit {
    message: String,
    url: String, // yes it will be a string, you got a problem?
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
    let twitter_api_key =
        std::env::var("TWITTER_API_KEY").expect("TWITTER_API_KEY env variable is required");
    let twitter_api_secret =
        std::env::var("TWITTER_API_SECRET").expect("TWITTER_API_SECRET env variable is required");
    let twitter_access_key =
        std::env::var("TWITTER_ACCESS_KEY").expect("TWITTER_ACCESS_KEY env variable is required");
    let twitter_access_secret = std::env::var("TWITTER_ACCESS_SECRET")
        .expect("TWITTER_ACCESS_SECRET env variable is required");

    // github setup
    let mut etag = None;
    let mut seen = VecDeque::with_capacity(TRACKING_CAPACITY);
    let octo = Octocrab::builder().personal_token(token).build()?;

    // twitter setup
    let con_token = egg_mode::KeyPair::new(twitter_api_key, twitter_api_secret);
    let acc_token = egg_mode::KeyPair::new(twitter_access_key, twitter_access_secret);
    let twitter_token = egg_mode::Token::Access {
        consumer: con_token,
        access: acc_token,
    };

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
                        let commit_response =
                            octo._get(commit.url.to_string(), None::<&()>).await?;
                        let commit_details: CommitDetails = commit_response
                            .json()
                            .await
                            .expect("Commit details are busted");

                        if commit_details.stats.additions != 0
                            && commit_details.stats.deletions != 0
                        {
                            let ratio = (commit_details.stats.deletions
                                / commit_details.stats.additions)
                                as f64;
                            if ratio > 10.0 {
                                // this is where we tweet
                                let mut tweet_text = "DELETE ALL THE THINGS!  ".to_string();
                                tweet_text.push_str(
                                    format!("  DELETIONS: {}", &commit_details.stats.deletions)
                                        .as_str(),
                                );
                                tweet_text
                                    .push_str(format!("  SHA: {}", &commit_details.sha).as_str());
                                tweet_text.push_str(
                                    format!("  \"{}\"  ", &commit_details.commit.message).as_str(),
                                );
                                tweet_text.push_str(&commit_details.commit.url);

                                let tweet = egg_mode::tweet::DraftTweet::new(tweet_text);
                                tweet
                                    .send(&twitter_token)
                                    .await
                                    .expect("something went wrong while tweeting");
                            }
                        } else if commit_details.stats.additions == 0
                            && commit_details.stats.deletions > 0
                        {
                            // this is where we tweet
                            let mut tweet_text = "NO ADDITION IS THE BEST!  ".to_string();
                            tweet_text.push_str(
                                format!("  DELETIONS: {}", &commit_details.stats.deletions)
                                    .as_str(),
                            );
                            tweet_text.push_str(format!("  SHA: {}", &commit_details.sha).as_str());
                            tweet_text.push_str(
                                format!("  \"{}\"  ", &commit_details.commit.message).as_str(),
                            );
                            tweet_text.push_str(&commit_details.commit.url);

                            let tweet = egg_mode::tweet::DraftTweet::new(tweet_text);
                            tweet
                                .send(&twitter_token)
                                .await
                                .expect("something went wrong while tweeting");
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
