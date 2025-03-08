use std::{io, time::Duration};

use fantoccini::{Client, ClientBuilder};
use tokio::task::JoinHandle;

const TESTRUN_SETUP_TIMEOUT: Duration = Duration::from_secs(5);
const DEFAULT_WAIT_TIMEOUT: Duration = Duration::from_secs(3);
const WEBDRIVER_ADDRESS: &str = "http://localhost:4444";

async fn init_webdriver_client() -> Client {
    let mut chrome_args = Vec::new();
    chrome_args.extend(["--headless", "--disable-gpu", "--disable-dev-shm-usage"]);

    let mut caps = serde_json::map::Map::new();
    caps.insert(
        "goog:chromeOptions".to_string(),
        serde_json::json!({
            "args": chrome_args,
        }),
    );
    ClientBuilder::native()
        .capabilities(caps)
        .connect(WEBDRIVER_ADDRESS)
        .await
        .expect("web driver to be available")
}

type ServerTaskHandle = JoinHandle<Result<(), io::Error>>;

async fn init() -> (String, ServerTaskHandle) {
    let (tx, rx) = tokio::sync::oneshot::channel();
    let handle = tokio::spawn(async move {
        let app = poc_rust_htmx::build_app();
        let addr = std::net::SocketAddr::from(([127, 0, 0, 1], 0));
        let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
        let assigned_addr = listener.local_addr().unwrap();
        tx.send(assigned_addr).unwrap();
        axum::serve(listener, app.into_make_service()).await
    });
    let assigned_addr = tokio::time::timeout(TESTRUN_SETUP_TIMEOUT, rx)
        .await
        .expect("test setup to not have timed out")
        .expect("socket address to have been received from the channel");
    let app_addr = format!("http://localhost:{}", assigned_addr.port());
    (app_addr, handle)
}

mod tests {
    use std::time::Duration;

    use fantoccini::Locator;

    use crate::{DEFAULT_WAIT_TIMEOUT, init, init_webdriver_client};

    #[tokio::test]
    async fn test_todo() {
        let (addr, _) = init().await;
        let client = init_webdriver_client().await;
        let c = client.clone();
        let res = tokio::spawn(async move {
            c.goto(&addr).await.unwrap();

            // Navigate to /todos using home page link
            let todo_nav = c
                .find(Locator::XPath("//a[contains(text(), 'Todos')]"))
                .await
                .unwrap();
            todo_nav.click().await.unwrap();
            let todos_url = format!("{addr}/todos");
            assert_eq!(c.current_url().await.unwrap().as_ref(), todos_url);

            // Add a new todo
            let input = c.find(Locator::Id("todo")).await.unwrap();
            input.send_keys("a new todo item").await.unwrap();

            let add_button = c
                .find(Locator::XPath("//button[text()='Add']"))
                .await
                .unwrap();
            add_button.click().await.unwrap();

            c.wait()
                .at_most(DEFAULT_WAIT_TIMEOUT)
                .for_element(Locator::XPath("//span[text()='a new todo item']"))
                .await
                .unwrap();

            // Delete the todo
            c.find(Locator::XPath("//button[text()='X']"))
                .await
                .unwrap()
                .click()
                .await
                .unwrap();
            c.wait()
                .at_most(Duration::from_millis(10))
                .for_element(Locator::XPath("//span[text()='a new todo item']"))
                .await
                .unwrap_err();
        })
        .await;
        client.close().await.unwrap();
        if let Err(e) = res {
            std::panic::resume_unwind(Box::new(e));
        }
    }
}
