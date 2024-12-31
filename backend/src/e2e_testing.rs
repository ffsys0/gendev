#[cfg(test)]
mod tests {
    use std::thread;
    use std::time::{Duration, Instant};
    use reqwest::Url;
    use reqwest;

    #[test]
    fn test_main_server() {
        thread::spawn(|| {
            let _ = crate::main();
        });

        thread::sleep(Duration::from_millis(500));

        let mut url = Url::parse("http://localhost:8080/").unwrap();
        url.query_pairs_mut().append_pair("teams", "[\"Bayern München\"]");
        url.query_pairs_mut().append_pair("tournaments", "[]");
        url.query_pairs_mut().append_pair("games", "[]");
        url.query_pairs_mut().append_pair("live", "1");
        url.query_pairs_mut().append_pair("highlights", "0");
        url.query_pairs_mut().append_pair("only_monthly_billing", "0");
        url.query_pairs_mut().append_pair("all_games", "0");
        
        let start = Instant::now();
        let resp = reqwest::blocking::get(url)
            .expect("Failed to send request");
        let duration = start.elapsed();
        println!("Response time for bayern munich query: {:?}", duration);
        assert!(resp.status().is_success());

        let mut url = Url::parse("http://localhost:8080/").unwrap();
        url.query_pairs_mut().append_pair("teams", "[\"Hatayspor\", \"Deutschland\", \"Bayern München\", \"Real Madrid\"]");
        url.query_pairs_mut().append_pair("tournaments", "[]");
        url.query_pairs_mut().append_pair("games", "[]");
        url.query_pairs_mut().append_pair("live", "1");
        url.query_pairs_mut().append_pair("highlights", "0");
        url.query_pairs_mut().append_pair("only_monthly_billing", "0");
        url.query_pairs_mut().append_pair("all_games", "0");
        let start = Instant::now();
        let resp = reqwest::blocking::get(url)
            .expect("Failed to send request");
        let duration = start.elapsed();
        println!("Response time for Hatayspor, Deutschland, Bayern München and Real Madrid query: {:?}", duration);
        assert!(resp.status().is_success());

        let mut url = Url::parse("http://localhost:8080/").unwrap();
        url.query_pairs_mut().append_pair("teams", "[\"Oxford United\", \"Los Angeles FC\", \"AS Rom\"]");
        url.query_pairs_mut().append_pair("tournaments", "[]");
        url.query_pairs_mut().append_pair("games", "[]");
        url.query_pairs_mut().append_pair("live", "1");
        url.query_pairs_mut().append_pair("highlights", "0");
        url.query_pairs_mut().append_pair("only_monthly_billing", "0");
        url.query_pairs_mut().append_pair("all_games", "0");
        let start = Instant::now();
        let resp = reqwest::blocking::get(url)
            .expect("Failed to send request");
        let duration = start.elapsed();
        println!("Response time for Oxford United, Los Angeles FC, AS Rom query: {:?}", duration);
        assert!(resp.status().is_success());


        let mut url = Url::parse("http://localhost:8080/").unwrap();
        url.query_pairs_mut().append_pair("teams", "[]");
        url.query_pairs_mut().append_pair("tournaments", "[]");
        url.query_pairs_mut().append_pair("games", "[]");
        url.query_pairs_mut().append_pair("live", "1");
        url.query_pairs_mut().append_pair("highlights", "0");
        url.query_pairs_mut().append_pair("only_monthly_billing", "0");
        url.query_pairs_mut().append_pair("all_games", "1");
        let start = Instant::now();
        let client = reqwest::blocking::Client::builder()
            .timeout(Duration::from_secs(100))
            .build()
            .expect("Failed to build client");
        let resp = client.get(url)
            .send()
            .expect("Failed to send request");
        let duration = start.elapsed();
        println!("Response time for all games: {:?}", duration);
        assert!(resp.status().is_success());
    }
}
