use chrome_controller::ChromiumBrowser;
use core::panic;
use json::object;

#[tokio::main]
async fn main()
{
    let mut browser = ChromiumBrowser::new(String::from("http://localhost:8080/json"));
    match browser.connect().await
    {
        Ok(()) => println!("Connected to chromium!"),
        Err(error) => panic!("Couldn't connect to chromium: {error}"),
    }

    let mut open_chrome = object! {
        id: 3,
        method: "Page.navigate",
        params: object! {
            url: "https://veracode.com"
        }
    };

    match browser.run_command(&mut open_chrome).await
    {
        Ok(()) => println!("We are golden"),
        Err(error) => println!("Couldn't run command: {error}"),
    };
}
