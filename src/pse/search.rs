use std::collections::BTreeMap;

use links::{BASE_URL, COMPANY_DIR};
use scraper::{Html, Selector};

use super::links;

pub async fn get_listed_companies() -> Result<(), Box<dyn std::error::Error>> {
    const TITLE: [&str; 5] = [
        "Company Name",
        "Stock Symbol",
        "Sector",
        "Subsector",
        "Listing Date",
    ];

    let mut page_count = 1;
    let mut listed_companies: Vec<BTreeMap<_, _>> = vec![];

    // As the number of listed companies grow overtime, we cannot set a static number to loop over;
    // we have to dynamically retrieve the number of paginated pages which list the companies,
    // iterate over the page count, and then scrape the companies. We can get page count from
    // `companyDirectory/search.ax`
    let url = format!("{BASE_URL}{COMPANY_DIR}search.ax?");
    let text = unescape(reqwest::get(&url).await?.text().await?).await;

    let document = Html::parse_document(&text);
    let selector = Selector::parse("span.count").unwrap();

    // Before slicing, an example result using the loops below is: ["[", "1", " ", "/", "6", ...]
    // The only element we need in the vector is the one at index 0, i.e, "6", the total number of
    // paginated pages, which we will use to loop over later.
    for element in document.select(&selector) {
        for text in element.text() {
            let parts: Vec<_> = text.chars().collect();
            page_count += parts[4].to_string().parse::<i32>().unwrap();

            // Only the first text contains the values we need, so we can stop here.
            break;
        }
        break;
    }

    let mut companies: Vec<Vec<String>> = vec![];

    // Retrieve the documents to scrape.
    for page_number in 1..=page_count {
        let text = unescape(
            reqwest::get(format!("{url}pageNo={page_number}"))
                .await?
                .text()
                .await?,
        )
        .await;

        // Extract table rows containing company cinformation from page.
        // This is what those table rows may look like:
        // <tr>
        //   <td><a>2GO Group, Inc.</a></td>
        //   <td><a>2GO</a></td>
        //   <td>Services</td>
        //   <td>Transportation Services</td>
        //   <td>May 15, 1995</td>
        // </tr>
        let document = Html::parse_document(&text);
        let tr_selector = Selector::parse("tr").unwrap();

        for tr in document.select(&tr_selector) {
            let mut company_info: Vec<String> = vec![];

            for td in tr.children() {
                // Extract texts inside the table data cells. Some of the texts we need may be
                // inside children. In that case we just iterate over them.
                for cell in td.children() {
                    if let Some(data) = cell.value().as_text() {
                        company_info.push(data.to_string());
                    } else {
                        for names in cell.children() {
                            if let Some(name) = names.value().as_text() {
                                company_info.push(name.to_string());
                            }
                        }
                    }
                }
            }
            if company_info != TITLE {
                companies.push(company_info);
            }
        }
    }

    for company in companies {
        let mut tdata = company.iter().cloned();
        let mut thead = TITLE.iter();
        let mut company_row: BTreeMap<_, _> = BTreeMap::new();

        loop {
            match (thead.next(), tdata.next()) {
                (Some(head), Some(cell)) => company_row.insert(head, cell),
                (Some(head), None) => company_row.insert(head, String::from("")),
                (None, Some(_cell)) => break,
                (None, None) => break,
            };
        }

        listed_companies.push(company_row);
    }

    for listed_company in listed_companies {
        println!("{:?}", listed_company)
    }

    Ok(())
}

pub async fn unescape(text: String) -> String {
    text.replace("\n", "")
        .replace("\t", "")
        .replace("\r", "")
        .replace("\"", "")
}
