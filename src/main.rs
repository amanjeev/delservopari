/// shamelessly running code copied from 
/// https://github.com/XAMPPRocky/octocrab/blob/master/examples/poll_events.rs 


use octocrab::{Page, etag::Etagged, models::events::{Event, EventType, payload::{EventPayload, PushEventPayload}}};
use std::{collections::VecDeque};

const DELAY_MS: u64 = 500;
const TRACKING_CAPACITY: usize = 200;

#[tokio::main]
async fn main() -> octocrab::Result<()> {
    let mut etag = None;
    let mut seen = VecDeque::with_capacity(TRACKING_CAPACITY);
    let octo = octocrab::instance();
    loop {
        let response: Etagged<Page<Event>> = octo.events().etag(etag).per_page(100).send().await?;
        if let Some(page) = response.value {
            let page = page.into_iter().filter(|x| x.r#type == EventType::PushEvent);
            for event in page {
                if !seen.contains(&event.id) {
                    let commits = match event.payload {
                        Some(e) => {
                            match e {
                                EventPayload::PushEvent(push_event_payload) => { Some(push_event_payload.commits) },
                                _ => None
                            }
                        },
                        None => { None },
                    };
                    println!("{:?}", commits);
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

