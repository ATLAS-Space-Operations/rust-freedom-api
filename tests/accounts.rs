mod common;

use common::{TestResult, TestingEnv};
use freedom_api::prelude::*;
use futures::StreamExt;
use time::{Date, OffsetDateTime, Time};

#[tokio::test]
async fn fetch_account() -> TestResult {
    let env = TestingEnv::new();

    env.get_json_from_file("/accounts", vec![], "resources/accounts.json");
    let client = Client::from(env);

    let accounts = client
        .get_accounts()
        .map(|result| result.unwrap().into_inner())
        .collect::<Vec<Account>>()
        .await;
    let first = &accounts[0];

    assert_eq!(first.name, "ABC Space");
    assert_eq!(first.storage_key, "ABCSpace");
    assert_eq!(first.access_api_cidr, &[]);
    assert_eq!(first.access_api_cidr, &[]);
    assert!(!first.post_process_done_by_account);
    let date = Date::from_calendar_date(2022, time::Month::March, 24)?;
    let t = Time::from_hms(14, 35, 40)?;
    assert_eq!(first.created, OffsetDateTime::new_utc(date, t));

    Ok(())
}
