use chrome_controller::ChromiumBrowser;
use serde_json::json;

fn main()
{
    let mut browser = ChromiumBrowser::connect(&String::from("http://localhost:8080/json")).unwrap();

    let mut open_chrome = json!({
        "id": 3, 
        "method": "Page.navigate", 
        "params": {
            "url": "https://google.com", 
        }
    });

    browser.run_command(&mut open_chrome).unwrap();
}
