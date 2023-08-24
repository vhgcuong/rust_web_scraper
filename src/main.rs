use reqwest;
use csv;
use indicatif::{ProgressBar, ProgressState, ProgressStyle};
use std::{fmt::Write, thread, time::Duration};
use reqwest::Error;

struct PokemonProduct {
    url: String,
    image: String,
    name: String,
    price: String,
}

fn get_pokemon_product_from_web(pokemon_products: &mut Vec<PokemonProduct>) -> Result<(), Error>{
    // pagination page to start from
    let first_page = "https://scrapeme.live/shop/page/1/";

    // define the supporting data structures
    let mut pages_to_scrape: Vec<String> = vec![first_page.to_owned()];
    let mut pages_discovered: std::collections::HashSet<String> = std::collections::HashSet::new();

    // current iteration
    let mut i = 1;
    // max number of iterations allowed
    let max_iterations = 48; //48

    //
    let bar = ProgressBar::new(max_iterations);
    bar.set_style(ProgressStyle::with_template("{spinner:.green} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {bytes}/{total_bytes} ({eta})")
        .unwrap()
        .with_key("eta", |state: &ProgressState, w: &mut dyn Write| write!(w, "{:.1}s", state.eta().as_secs_f64()).unwrap())
        .progress_chars("#>-"));

    while !pages_to_scrape.is_empty() && i <= max_iterations {
        //
        thread::sleep(Duration::from_millis(5));
        bar.inc(1);

        // get the first element from the queue
        let page_to_scrape = pages_to_scrape.remove(0);

        // retrieve and parse the HTML document
        let response = reqwest::blocking::get(page_to_scrape);
        let html_content = response.unwrap().text().unwrap();
        let document = scraper::Html::parse_document(&html_content);

        // scraping logic...
        // define the CSS selector to get all product
        // on the page
        let html_product_selector = scraper::Selector::parse("li.product").unwrap();
        // apply the CSS selector to get all products
        let html_products = document.select(&html_product_selector);

        // iterate over each HTML product to extract data
        // from it
        for html_product in html_products {
            // scraping logic to retrieve the info
            // of interest
            let url = html_product
                .select(&scraper::Selector::parse("a").unwrap())
                .next()
                .and_then(|a| a.value().attr("href"))
                .map(str::to_owned).expect("href not found");
            let image = html_product
                .select(&scraper::Selector::parse("img").unwrap())
                .next()
                .and_then(|img| img.value().attr("src"))
                .map(str::to_owned).expect("src not found");
            let name = html_product
                .select(&scraper::Selector::parse("h2").unwrap())
                .next()
                .map(|h2| h2.text().collect::<String>()).expect("h2 not found");
            let price = html_product
                .select(&scraper::Selector::parse(".price").unwrap())
                .next()
                .map(|price| price.text().collect::<String>()).expect("price not found");

            // instanciate a new Pokemon product
            // with the scraped data and add it to the list
            let pokemon_product = PokemonProduct {
                url,
                image,
                name,
                price,
            };

            pokemon_products.push(pokemon_product);
        }

        // get all pagination link elements
        let html_pagination_link_selector = scraper::Selector::parse("a.page-numbers").unwrap();
        let html_pagination_links = document.select(&html_pagination_link_selector);

        // iterate over them to find new pages to scrape
        for html_pagination_link in html_pagination_links {
            // get the pagination link URL
            let pagination_url = html_pagination_link
                .value()
                .attr("href")
                .unwrap()
                .to_owned();

            // if the page discovered is new
            if !pages_discovered.contains(&pagination_url) {
                pages_discovered.insert(pagination_url.clone());

                // if the page discovered should be scraped
                if !pages_to_scrape.contains(&pagination_url) {
                    pages_to_scrape.push(pagination_url.clone());
                }
            }
        }

        // increment the iteration counter
        i += 1;
    }

    bar.finish_with_message("done");
    Ok(())
}

// Do a request for the given URL, with a minimum time between requests
// to avoid overloading the server.
pub fn do_throttled_request(url: &str) -> Result<String, Error> {
    // See the real code for the throttling - it's omitted here for clarity
    let response = reqwest::blocking::get(url)?;
    response.text()
}

fn export_csv(pokemon_products: &Vec<PokemonProduct>) {
    // create the CSV output file
    let path = std::path::Path::new("products.csv");
    let mut writer = csv::Writer::from_path(path).unwrap();

    let _ = writer.flush();

    // append the header to the CSV
    writer
        .write_record(&["url", "image", "name", "price"])
        .unwrap();

    //
    let bar = ProgressBar::new(pokemon_products.len() as u64);
    bar.set_style(ProgressStyle::with_template("{spinner:.green} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {bytes}/{total_bytes} ({eta})")
        .unwrap()
        .with_key("eta", |state: &ProgressState, w: &mut dyn Write| write!(w, "{:.1}s", state.eta().as_secs_f64()).unwrap())
        .progress_chars("#>-"));

    // populate the output file
    for product in pokemon_products {

        // Bar
        thread::sleep(Duration::from_millis(5));
        bar.inc(1);

        let url = product.url.clone();
        let image = product.image.clone();
        let name = product.name.clone();
        let price = product.price.clone();
        writer.write_record(&[url, image, name, price]).unwrap();
    }

    //
    bar.finish_with_message("done");

    // free up the resources
    writer.flush().unwrap();
}

fn screenshot_data() {
    let browser = headless_chrome::Browser::default().unwrap();
    let tab = browser.new_tab().unwrap();
    tab.navigate_to("https://scrapeme.live/shop/").unwrap();

    let screenshot_data = tab
        .capture_screenshot(headless_chrome::protocol::cdp::Page::CaptureScreenshotFormatOption::Png, None, None, true)
        .unwrap();

    // write the screenshot data to the output file
    std::fs::write("screenshot.png", &screenshot_data).unwrap();
}

fn main() {

    let mut pokemon_products: Vec<PokemonProduct> = Vec::new();

    let _ = get_pokemon_product_from_web(&mut pokemon_products);

    export_csv(&pokemon_products);

    // screenshot_data();
}
