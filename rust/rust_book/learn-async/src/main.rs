use std::pin::{Pin, pin};
use std::{process::Output, time::Duration};
use trpl::{Either, Html};
use trpl::{ReceiverStream, Stream, StreamExt};

async fn page_title(url: &str) -> (&str, Option<String>) {
    let response = trpl::get(url).await;
    let response_text = response.text().await;
    let title = Html::parse(&response_text)
        .select_first("title")
        .map(|title_element| title_element.inner_html());
    (url, title)
}

fn extract_page_titles(args: Vec<String>) {
    trpl::run(async {
        let title_1 = page_title(&args[1]);
        let title_2 = page_title(&args[2]);

        let (url, maybe_title) = match trpl::race(title_1, title_2).await {
            Either::Left(left) => left,
            Either::Right(right) => right,
        };

        println!("{url} returned first");
        match maybe_title {
            Some(title) => println!("Its page title is: '{title}'"),
            None => println!("Its title could not be parsed"),
        }
    });
}

fn use_join() {
    trpl::run(async {
        let fut1 = async {
            for i in 1..10 {
                println!("hi number {i} from the first task!");
                trpl::sleep(Duration::from_millis(500)).await;
            }
        };

        let fut2 = async {
            for i in 1..5 {
                println!("hi number {i} from the second task!");
                trpl::sleep(Duration::from_millis(500)).await;
            }
        };

        trpl::join(fut1, fut2).await;
    });
}

fn use_channels() {
    trpl::run(async {
        let (tx, mut rx) = trpl::channel();

        // The move keyword here moves ownership of tx to the async block
        // ensures that when there is no more messages tx is drop
        // causing the rx_fut to exit as there would be no more messages to read from
        let tx_fut = async move {
            let vals = vec![
                String::from("hi"),
                String::from("from"),
                String::from("the"),
                String::from("future"),
            ];

            for val in vals {
                tx.send(val).unwrap();
                trpl::sleep(Duration::from_millis(500)).await;
            }
        };

        let rx_fut = async {
            while let Some(value) = rx.recv().await {
                println!("received '{value}'");
            }
        };

        trpl::join(tx_fut, rx_fut).await;
    });
}

fn use_pin() {
    trpl::run(async {
        let (tx, mut rx) = trpl::channel();

        // The move keyword here moves ownership of tx to the async block
        // ensures that when there is no more messages tx is drop
        // causing the rx_fut to exit as there would be no more messages to read from
        let tx_fut = pin!(async move {
            let vals = vec![
                String::from("<pin> hi"),
                String::from("<pin> from"),
                String::from("<pin> the"),
                String::from("<pin> future"),
            ];

            for val in vals {
                tx.send(val).unwrap();
                trpl::sleep(Duration::from_millis(500)).await;
            }
        });

        let rx_fut = pin!(async {
            while let Some(value) = rx.recv().await {
                println!("received '{value}'");
            }
        });

        let futures: Vec<Pin<&mut dyn Future<Output = ()>>> = vec![tx_fut, rx_fut];

        trpl::join_all(futures).await;
    });
}

fn pass_control_to_runtime_using_yield() {
    trpl::run(async {
        let a = async {
            println!("'a' started.");
            println!("a {}", 30);
            trpl::yield_now().await;
            println!("a {}", 10);
            trpl::yield_now().await;
            println!("a {}", 20);
            trpl::yield_now().await;
            println!("'a' finished.");
        };

        let b = async {
            println!("'b' started.");
            println!("b {}", 75);
            trpl::yield_now().await;
            println!("b {}", 10);
            trpl::yield_now().await;
            println!("b {}", 15);
            trpl::yield_now().await;
            println!("b {}", 350);
            trpl::yield_now().await;
            println!("'b' finished.");
        };

        trpl::race(a, b).await;
    });
}

async fn timeout<F: Future>(future_to_try: F, max_time: Duration) -> Result<F::Output, Duration> {
    let timeout_future = async { trpl::sleep(max_time) };

    match trpl::race(future_to_try, timeout_future).await {
        Either::Left(left) => Ok(left),
        Either::Right(_) => Err(max_time),
    }
}

fn use_async_as_building_blocks() {
    trpl::run(async {
        let slow = async {
            trpl::sleep(Duration::from_secs(5)).await;
            "I finished!"
        };

        match timeout(slow, Duration::from_secs(2)).await {
            Ok(message) => println!("Succeeded with '{message}'"),
            Err(duration) => {
                println!("Failed after {} seconds", duration.as_secs())
            }
        }
    });
}

fn get_messages() -> impl Stream<Item = String> {
    let (tx, rx) = trpl::channel();

    trpl::spawn_task(async move {
        let messages = ["a", "b", "c", "d", "e", "f", "g", "h", "i", "j"];
        for (index, message) in messages.into_iter().enumerate() {
            let time_to_sleep = if index % 2 == 0 { 100 } else { 300 };
            trpl::sleep(Duration::from_millis(time_to_sleep)).await;
            if let Err(send_error) = tx.send(format!("Message: '{message}'")) {
                eprintln!("Cannot send message '{message}: {send_error}'");
                break;
            }
        }
    });
    ReceiverStream::new(rx)
}

fn get_intervals() -> impl Stream<Item = u32> {
    let (tx, rx) = trpl::channel();

    trpl::spawn_task(async move {
        let mut count = 0;
        loop {
            trpl::sleep(Duration::from_millis(1)).await;
            count += 1;
            if let Err(send_error) = tx.send(count) {
                eprintln!("Cannot send interval '{count}: {send_error}'");
                break;
            }
        }
    });
    ReceiverStream::new(rx)
}

fn composing_streams() {
    trpl::run(async {
        let messages = pin!(get_messages().timeout(Duration::from_millis(200)));
        let intervals = get_intervals()
            .map(|count| format!("Interval: {count}"))
            .throttle(Duration::from_millis(100))
            .timeout(Duration::from_secs(10));
        let merged = messages.merge(intervals).take(20);
        let mut stream = pin!(merged);

        while let Some(result) = stream.next().await {
            match result {
                Ok(message) => println!("{message}"),
                Err(reason) => eprintln!("Problem: {reason:?}"),
            }
        }
    });
}

fn main() {
    // let args: Vec<String> = std::env::args().collect();
    // extract_page_titles(args);
    // use_join();
    // use_channels();
    // use_pin();
    // pass_control_to_runtime_using_yield();
    // use_async_as_building_blocks();
    trpl::run(async {
        let values = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
        let iter = values.iter().map(|n| n * 2);
        let stream = trpl::stream_from_iter(iter);

        let mut filtered = stream.filter(|value| value % 3 == 0 || value % 5 == 0);

        while let Some(value) = filtered.next().await {
            println!("The value was: {value}");
        }
    });

    composing_streams();
}
